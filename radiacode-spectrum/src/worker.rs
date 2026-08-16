use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, unbounded};
use radiacode_core::{
    AlarmLimits, DataBufCursor, DeviceConfig, DeviceEndpoint, DeviceStatus, DiscoveredDevice,
    MonitorPollSample, RadiaCode, SessionRestore,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::{self, MissedTickBehavior};
use tracing::{debug, info};

use crate::model::{DeviceInfo, SpectrumView};
use crate::spectrogram::capture::{SpectrogramCapture, spawn_capture_router};
use crate::worker_ops::{
    SessionEpoch, handle_apply_device_config, handle_connect, handle_disconnect, handle_dose_reset,
    handle_fetch_device_config, handle_monitor, handle_reset, handle_scan, handle_spectrum,
    handle_sync_device_clock,
};

#[derive(Debug, Clone)]
pub enum WorkerCommand {
    Scan,
    Connect {
        endpoint: DeviceEndpoint,
        hint_rssi: Option<i16>,
    },
    Disconnect,
    FetchSpectrum,
    ResetSpectrum,
    ResetDose,
    FetchMonitor,
    SetCaptureInterval(f64),
    SetMonitorPollInterval(u64),
    FetchDeviceConfig,
    ApplyDeviceConfig(DeviceConfig),
    SyncDeviceClock,
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    ScanFinished(Vec<DiscoveredDevice>),
    Connected(DeviceInfo),
    Disconnected,
    Reconnecting,
    UsbPermissionRequired { endpoint: DeviceEndpoint },
    Spectrum(SpectrumView),
    DeviceStatus(DeviceStatus),
    MonitorSample(MonitorPollSample),
    DoseResetComplete,
    MonitorPollComplete,
    AlarmLimits(AlarmLimits),
    DeviceConfig(DeviceConfig),
    Error(String),
    Busy(bool),
}

pub struct WorkerHandle {
    pub commands: UnboundedSender<WorkerCommand>,
    pub events: Receiver<WorkerEvent>,
    session_epoch: Arc<AtomicU64>,
}

impl WorkerHandle {
    pub fn end_session(&self) {
        self.session_epoch.fetch_add(1, Ordering::SeqCst);
    }
}

pub struct WorkerSession {
    pub device: Option<RadiaCode>,
    pub session_endpoint: Option<DeviceEndpoint>,
    pub alarm_limits: Option<AlarmLimits>,
    pub monitor_polls: u64,
    pub data_buf_cursor: DataBufCursor,
    pub link_status: DeviceStatus,
    pub session_restore: Option<SessionRestore>,
}

impl WorkerSession {
    pub fn new() -> Self {
        Self {
            device: None,
            session_endpoint: None,
            alarm_limits: None,
            monitor_polls: 0,
            data_buf_cursor: DataBufCursor::default(),
            link_status: DeviceStatus::default(),
            session_restore: None,
        }
    }

    pub fn reset(&mut self) {
        self.session_endpoint = None;
        self.alarm_limits = None;
        self.monitor_polls = 0;
        self.data_buf_cursor.reset();
        self.link_status = DeviceStatus::default();
        self.session_restore = None;
        self.device = None;
    }

    pub fn prepare_connect(&mut self) {
        self.alarm_limits = None;
        self.monitor_polls = 0;
        self.data_buf_cursor.reset();
        self.link_status = DeviceStatus::default();
        self.session_restore = None;
    }

    pub fn prepare_disconnect(&mut self) {
        self.session_endpoint = None;
        self.alarm_limits = None;
        self.monitor_polls = 0;
        self.data_buf_cursor.reset();
        self.link_status = DeviceStatus::default();
        self.session_restore = None;
    }

    pub fn clear_if_device_lost(&mut self) {
        if self.device.is_none() {
            self.reset();
        }
    }
}

pub fn spawn_worker(capture: Arc<Mutex<SpectrogramCapture>>) -> WorkerHandle {
    let (commands, command_rx) = unbounded_channel();
    let (worker_event_tx, worker_event_rx) = unbounded();
    let (ui_event_tx, ui_event_rx) = unbounded();
    spawn_capture_router(worker_event_rx, ui_event_tx, capture);
    let session_epoch = Arc::new(AtomicU64::new(0));
    let worker_epoch = Arc::clone(&session_epoch);
    info!("spawning device worker thread");
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        runtime.block_on(worker_loop(command_rx, worker_event_tx, worker_epoch));
    });
    WorkerHandle {
        commands,
        events: ui_event_rx,
        session_epoch,
    }
}

const MONITOR_POLL_SECS: u64 = 1;

struct CoalescedBatch {
    priority: Option<WorkerCommand>,
    fetch_monitor: bool,
    fetch_spectrum: bool,
}

impl CoalescedBatch {
    fn absorb(&mut self, command: WorkerCommand) {
        match command {
            WorkerCommand::FetchMonitor => self.fetch_monitor = true,
            WorkerCommand::FetchSpectrum => self.fetch_spectrum = true,
            priority => self.priority = Some(priority),
        }
    }
}

async fn recv_batch(commands: &mut UnboundedReceiver<WorkerCommand>) -> Option<CoalescedBatch> {
    let first = commands.recv().await?;
    let mut batch = CoalescedBatch {
        priority: None,
        fetch_monitor: false,
        fetch_spectrum: false,
    };
    batch.absorb(first);
    while batch.priority.is_none() {
        match commands.try_recv() {
            Ok(next) => batch.absorb(next),
            Err(_) => break,
        }
    }
    Some(batch)
}

fn capture_duration(secs: f64) -> Duration {
    Duration::from_secs_f64(secs.clamp(1.0, 20.0))
}

fn monitor_poll_duration(secs: u64) -> Duration {
    Duration::from_secs(secs.clamp(1, 60))
}

async fn run_fetch_batch(
    batch: CoalescedBatch,
    events: &Sender<WorkerEvent>,
    session_epoch: &Arc<AtomicU64>,
    session: &mut WorkerSession,
) {
    if batch.fetch_monitor || batch.fetch_spectrum {
        debug!(
            fetch_monitor = batch.fetch_monitor,
            fetch_spectrum = batch.fetch_spectrum,
            session_endpoint = ?session.session_endpoint,
            "worker coalesced fetch batch"
        );
    }
    let _ = events.send(WorkerEvent::Busy(true));
    let epoch = SessionEpoch {
        live: Arc::clone(session_epoch),
        started: session_epoch.load(Ordering::SeqCst),
    };
    if batch.fetch_monitor {
        handle_monitor(events, session, &epoch).await;
        session.clear_if_device_lost();
    }
    if batch.fetch_spectrum && session.device.is_some() {
        handle_spectrum(events, session, &epoch).await;
        session.clear_if_device_lost();
    }
    let _ = events.send(WorkerEvent::Busy(false));
}

async fn worker_loop(
    mut commands: UnboundedReceiver<WorkerCommand>,
    events: Sender<WorkerEvent>,
    session_epoch: Arc<AtomicU64>,
) {
    let mut session = WorkerSession::new();
    let mut capture_interval_secs = 5.0;
    let mut monitor_poll_secs = MONITOR_POLL_SECS;
    let mut spectrum_tick = time::interval(capture_duration(capture_interval_secs));
    spectrum_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut monitor_tick = time::interval(monitor_poll_duration(monitor_poll_secs));
    monitor_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    debug!("device worker loop ready");
    loop {
        tokio::select! {
            batch = recv_batch(&mut commands) => {
                let Some(batch) = batch else {
                    break;
                };
                if let Some(command) = batch.priority {
                    match command {
                        WorkerCommand::SetCaptureInterval(secs) => {
                            capture_interval_secs = secs;
                            spectrum_tick = time::interval(capture_duration(capture_interval_secs));
                            spectrum_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
                            debug!(capture_interval_secs, "worker capture interval updated");
                            continue;
                        }
                        WorkerCommand::SetMonitorPollInterval(secs) => {
                            monitor_poll_secs = secs.clamp(1, 60);
                            monitor_tick = time::interval(monitor_poll_duration(monitor_poll_secs));
                            monitor_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
                            debug!(monitor_poll_secs, "worker monitor poll interval updated");
                            continue;
                        }
                        command => {
                            debug!(?command, session_endpoint = ?session.session_endpoint, "worker command");
                            run_command(
                                command,
                                &events,
                                &session_epoch,
                                &mut session,
                            )
                            .await;
                            if session.device.is_some() {
                                spectrum_tick.reset();
                                monitor_tick.reset();
                            }
                            continue;
                        }
                    }
                }
                run_fetch_batch(
                    batch,
                    &events,
                    &session_epoch,
                    &mut session,
                )
                .await;
            }
            _ = spectrum_tick.tick(), if session.device.is_some() => {
                debug!(capture_interval_secs, "worker background spectrum fetch");
                run_fetch_batch(
                    CoalescedBatch {
                        priority: None,
                        fetch_monitor: false,
                        fetch_spectrum: true,
                    },
                    &events,
                    &session_epoch,
                    &mut session,
                )
                .await;
            }
            _ = monitor_tick.tick(), if session.device.is_some() => {
                debug!("worker background monitor fetch");
                run_fetch_batch(
                    CoalescedBatch {
                        priority: None,
                        fetch_monitor: true,
                        fetch_spectrum: false,
                    },
                    &events,
                    &session_epoch,
                    &mut session,
                )
                .await;
            }
        }
    }
    info!("device worker loop ended");
}

async fn run_command(
    command: WorkerCommand,
    events: &Sender<WorkerEvent>,
    session_epoch: &Arc<AtomicU64>,
    session: &mut WorkerSession,
) {
    let _ = events.send(WorkerEvent::Busy(true));
    let epoch = SessionEpoch {
        live: Arc::clone(session_epoch),
        started: session_epoch.load(Ordering::SeqCst),
    };
    match command {
        WorkerCommand::Scan => handle_scan(events).await,
        WorkerCommand::Connect {
            endpoint,
            hint_rssi,
        } => {
            handle_connect(events, session, &endpoint, hint_rssi, &epoch).await;
        }
        WorkerCommand::Disconnect => {
            session_epoch.fetch_add(1, Ordering::SeqCst);
            session.prepare_disconnect();
            handle_disconnect(events, session).await;
        }
        WorkerCommand::FetchSpectrum => {
            handle_spectrum(events, session, &epoch).await;
            session.clear_if_device_lost();
        }
        WorkerCommand::ResetSpectrum => {
            handle_reset(events, session, &epoch).await;
            session.clear_if_device_lost();
        }
        WorkerCommand::ResetDose => {
            handle_dose_reset(events, session, &epoch).await;
            session.clear_if_device_lost();
        }
        WorkerCommand::FetchMonitor => {
            handle_monitor(events, session, &epoch).await;
            session.clear_if_device_lost();
        }
        WorkerCommand::FetchDeviceConfig => {
            handle_fetch_device_config(events, session, &epoch).await;
            session.clear_if_device_lost();
        }
        WorkerCommand::ApplyDeviceConfig(config) => {
            handle_apply_device_config(events, session, config, &epoch).await;
            session.clear_if_device_lost();
        }
        WorkerCommand::SyncDeviceClock => {
            handle_sync_device_clock(events, session, &epoch).await;
            session.clear_if_device_lost();
        }
        WorkerCommand::SetCaptureInterval(_) | WorkerCommand::SetMonitorPollInterval(_) => {
            unreachable!("interval commands are handled in the worker select loop")
        }
    }
    let _ = events.send(WorkerEvent::Busy(false));
}

#[cfg(test)]
mod tests {
    use super::WorkerSession;
    use radiacode_core::DeviceStatus;

    #[test]
    fn worker_session_reset_clears_link_status() {
        let mut session = WorkerSession::new();
        session.link_status = DeviceStatus {
            battery_percent: Some(80.0),
            temperature_c: Some(25.0),
            rssi_dbm: Some(-50),
        };
        session.monitor_polls = 5;
        session.reset();
        assert_eq!(session.link_status, DeviceStatus::default());
        assert_eq!(session.monitor_polls, 0);
        assert!(session.device.is_none());
        assert!(session.session_endpoint.is_none());
    }
}
