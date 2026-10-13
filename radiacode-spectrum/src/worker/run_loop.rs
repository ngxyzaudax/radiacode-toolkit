use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use crossbeam_channel::Sender;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, info, warn};

use super::batch::{CoalescedBatch, recv_batch};
use super::epoch::{SessionEpoch, end_session};
use super::messages::{WorkerCommand, WorkerEvent};
use super::ops_config::{
    handle_apply_device_config, handle_fetch_device_config, handle_sync_device_clock,
};
use super::ops_connect::{handle_connect, handle_disconnect};
use super::ops_monitor::handle_monitor;
use super::ops_scan::handle_scan;
use super::ops_spectrum::{handle_dose_reset, handle_reset, handle_spectrum};
use super::session::WorkerSession;
use super::timers::FetchTimers;

pub struct WorkerContext {
    pub events: Sender<WorkerEvent>,
    pub session_epoch: Arc<AtomicU64>,
    pub session: WorkerSession,
}

impl WorkerContext {
    fn epoch(&self) -> SessionEpoch {
        SessionEpoch::capture(&self.session_epoch)
    }

    fn set_busy(&self, busy: bool) {
        let _ = self.events.send(WorkerEvent::Busy(busy));
    }
}

pub async fn worker_loop(mut commands: UnboundedReceiver<WorkerCommand>, mut ctx: WorkerContext) {
    let mut timers = FetchTimers::new();
    debug!("device worker loop ready");
    loop {
        tokio::select! {
            batch = recv_batch(&mut commands) => {
                let Some(batch) = batch else { break };
                handle_batch(batch, &mut ctx, &mut timers).await;
            }
            _ = timers.monitor.tick(), if ctx.session.has_device() => {
                debug!("worker background monitor fetch");
                run_fetch_batch(CoalescedBatch::monitor_only(), &mut ctx).await;
            }
            _ = timers.spectrum.tick(), if ctx.session.has_device() => {
                debug!(capture_interval_secs = timers.capture_secs, "worker background spectrum fetch");
                run_fetch_batch(CoalescedBatch::monitor_and_spectrum(), &mut ctx).await;
            }
        }
    }
    info!("device worker loop ended");
}

async fn handle_batch(batch: CoalescedBatch, ctx: &mut WorkerContext, timers: &mut FetchTimers) {
    match batch.priority {
        Some(WorkerCommand::SetCaptureInterval(secs)) => timers.set_capture_interval(secs),
        Some(WorkerCommand::SetMonitorPollInterval(secs)) => timers.set_monitor_poll_interval(secs),
        Some(command) => {
            debug!(?command, session_endpoint = ?ctx.session.session_endpoint, "worker command");
            run_command(command, ctx).await;
            if ctx.session.has_device() {
                timers.reset();
            }
        }
        None => run_fetch_batch(batch, ctx).await,
    }
}

pub async fn run_fetch_batch(batch: CoalescedBatch, ctx: &mut WorkerContext) {
    if batch.has_fetch() {
        debug!(
            fetch_monitor = batch.fetch_monitor,
            fetch_spectrum = batch.fetch_spectrum,
            session_endpoint = ?ctx.session.session_endpoint,
            "worker coalesced fetch batch"
        );
    }
    ctx.set_busy(true);
    let epoch = ctx.epoch();
    if batch.fetch_monitor {
        handle_monitor(&ctx.events, &mut ctx.session, &epoch).await;
        ctx.session.clear_if_device_lost();
    }
    if batch.fetch_spectrum && ctx.session.has_device() {
        handle_spectrum(&ctx.events, &mut ctx.session, &epoch).await;
        ctx.session.clear_if_device_lost();
    }
    ctx.set_busy(false);
}

pub async fn run_command(command: WorkerCommand, ctx: &mut WorkerContext) {
    ctx.set_busy(true);
    let epoch = ctx.epoch();
    let events = &ctx.events;
    let session = &mut ctx.session;
    match command {
        WorkerCommand::Scan => handle_scan(events).await,
        WorkerCommand::Connect {
            endpoint,
            hint_rssi,
        } => handle_connect(events, session, &endpoint, hint_rssi, &epoch).await,
        WorkerCommand::Disconnect => {
            end_session(&ctx.session_epoch);
            session.clear_link_state();
            handle_disconnect(events, session).await;
        }
        WorkerCommand::FetchSpectrum => handle_spectrum(events, session, &epoch).await,
        WorkerCommand::ResetSpectrum => handle_reset(events, session, &epoch).await,
        WorkerCommand::ResetDose => handle_dose_reset(events, session, &epoch).await,
        WorkerCommand::FetchMonitor => handle_monitor(events, session, &epoch).await,
        WorkerCommand::FetchDeviceConfig => {
            handle_fetch_device_config(events, session, &epoch).await
        }
        WorkerCommand::ApplyDeviceConfig(config) => {
            handle_apply_device_config(events, session, config, &epoch).await
        }
        WorkerCommand::SyncDeviceClock => handle_sync_device_clock(events, session, &epoch).await,
        WorkerCommand::SetCaptureInterval(_) | WorkerCommand::SetMonitorPollInterval(_) => {
            warn!(?command, "interval command reached run_command; ignored")
        }
    }
    session.clear_if_device_lost();
    ctx.set_busy(false);
}
