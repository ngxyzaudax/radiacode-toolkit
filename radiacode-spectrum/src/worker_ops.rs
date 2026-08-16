use crossbeam_channel::Sender;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use radiacode_bluetooth::{BleError, scan_radiacode_devices};
use radiacode_core::{
    AlarmLimits, DeviceConfig, DeviceEndpoint, DeviceStatus, DiscoveredDevice, Error, RadiaCode,
    SessionRestore, Spectrum, merge_discovered, merge_status,
};
use radiacode_usb::scan_usb_devices;
use tracing::{debug, error, info, warn};

use crate::model::{DeviceInfo, SpectrumView};
use crate::worker::{WorkerEvent, WorkerSession};

const CONNECT_COOLDOWN: Duration = Duration::from_millis(500);
const TRANSIENT_RETRIES: usize = 2;
const ALARM_REFRESH_POLLS: u64 = 120;

pub async fn handle_scan(events: &Sender<WorkerEvent>) {
    info!("scanning for radiacode devices over usb and bluetooth");
    let (usb_devices, ble_devices) = tokio::join!(scan_usb_async(), scan_bluetooth_async());
    let usb_devices = usb_devices.unwrap_or_else(|error| {
        warn!(%error, "usb scan failed");
        Vec::new()
    });
    let ble_devices = ble_devices.unwrap_or_else(|error| {
        warn!(%error, "bluetooth scan failed");
        Vec::new()
    });
    if usb_devices.is_empty() && ble_devices.is_empty() {
        let _ = events.send(WorkerEvent::ScanFinished(Vec::new()));
        return;
    }
    let devices = merge_discovered(usb_devices, ble_devices);
    info!(count = devices.len(), "scan finished");
    let _ = events.send(WorkerEvent::ScanFinished(devices));
}

async fn scan_usb_async() -> Result<Vec<DiscoveredDevice>, String> {
    tokio::task::spawn_blocking(scan_usb_devices)
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

async fn scan_bluetooth_async() -> Result<Vec<DiscoveredDevice>, String> {
    scan_radiacode_devices(Duration::from_secs(5))
        .await
        .map_err(|error: BleError| error.to_string())
}

pub async fn handle_connect(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
    epoch: &SessionEpoch,
) {
    info!(?endpoint, "connecting");
    if let Some(previous) = session.device.take() {
        debug!(?endpoint, "disconnecting previous session before connect");
        let _ = previous.disconnect().await;
        tokio::time::sleep(CONNECT_COOLDOWN).await;
    }
    session.prepare_connect();
    if !epoch.active() {
        warn!(?endpoint, "connect aborted: session ended");
        return;
    }
    match connect_endpoint(endpoint, None).await {
        Ok(mut device) => {
            if !epoch.active() {
                warn!(?endpoint, "connect aborted after link up: session ended");
                let _ = device.disconnect().await;
                return;
            }
            match load_device_info(&mut device, endpoint, hint_rssi, events).await {
                Ok(info) => {
                    if !epoch.active() {
                        warn!(
                            ?endpoint,
                            "connect aborted after metadata load: session ended"
                        );
                        let _ = device.disconnect().await;
                        return;
                    }
                    session.link_status = DeviceStatus::from(&info);
                    session.session_restore = device.session_restore();
                    info!(
                        ?endpoint,
                        serial = %info.serial,
                        model = %info.model,
                        "connected"
                    );
                    let _ = events.send(WorkerEvent::Connected(info));
                    let _ = events.send(WorkerEvent::DeviceStatus(session.link_status));
                    session.device = Some(device);
                    session.session_endpoint = Some(endpoint.clone());
                }
                Err(error) => {
                    error!(?endpoint, %error, "failed to load device info");
                    let _ = device.disconnect().await;
                    let _ = events.send(WorkerEvent::Error(error.to_string()));
                    let _ = events.send(WorkerEvent::Disconnected);
                }
            }
        }
        Err(error) => {
            error!(?endpoint, %error, "connect failed");
            if error.is_usb_permission_denied() {
                let _ = events.send(WorkerEvent::UsbPermissionRequired {
                    endpoint: endpoint.clone(),
                });
            } else {
                let _ = events.send(WorkerEvent::Error(error.to_string()));
            }
            let _ = events.send(WorkerEvent::Disconnected);
        }
    }
}

async fn load_device_info_with_retry(
    device: &mut RadiaCode,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
    events: &Sender<WorkerEvent>,
) -> radiacode_core::Result<DeviceInfo> {
    match load_device_info(device, endpoint, hint_rssi, events).await {
        Ok(info) => Ok(info),
        Err(error) if error.is_transient() => {
            warn!(%error, "transient device info load failed, retrying once");
            tokio::time::sleep(Duration::from_millis(500)).await;
            load_device_info(device, endpoint, hint_rssi, events).await
        }
        Err(error) => Err(error),
    }
}

async fn load_device_info(
    device: &mut RadiaCode,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
    events: &Sender<WorkerEvent>,
) -> radiacode_core::Result<DeviceInfo> {
    debug!(?endpoint, "loading device metadata");
    let metadata = device.metadata().await?;
    let refresh_rssi = matches!(endpoint, DeviceEndpoint::Bluetooth { .. });
    let mut status = device.device_status(refresh_rssi).await.unwrap_or_default();
    if status.rssi_dbm.is_none() && refresh_rssi {
        status.rssi_dbm = device.sample_rssi_dbm().await.or(hint_rssi);
    }
    if let Ok(limits) = device.alarm_limits().await {
        let _ = events.send(WorkerEvent::AlarmLimits(limits));
    }
    Ok(DeviceInfo::from_metadata(metadata, endpoint, status))
}

pub async fn handle_disconnect(events: &Sender<WorkerEvent>, session: &mut WorkerSession) {
    info!("disconnect requested");
    if let Some(device) = session.device.take()
        && let Err(error) = device.disconnect().await
    {
        error!(%error, "disconnect failed");
        let _ = events.send(WorkerEvent::Error(error.to_string()));
    }
    let _ = events.send(WorkerEvent::Disconnected);
}

pub async fn handle_spectrum(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("spectrum fetch skipped: no active device");
        return;
    };
    session.device = match fetch_spectrum_with_retries(&mut device).await {
        Ok(spectrum) => {
            if !epoch.active() {
                Some(device)
            } else {
                debug!(
                    channels = spectrum.counts.len(),
                    duration_secs = spectrum.duration.as_secs(),
                    "spectrum fetched"
                );
                let _ = events.send(WorkerEvent::Spectrum(SpectrumView::from_spectrum(spectrum)));
                Some(device)
            }
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

pub async fn handle_dose_reset(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("dose reset skipped: no active device");
        return;
    };
    info!("resetting accumulated dose");
    session.device = match device.dose_reset().await {
        Ok(()) => {
            if epoch.active() {
                let _ = events.send(WorkerEvent::DoseResetComplete);
            }
            Some(device)
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

pub async fn handle_reset(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("spectrum reset skipped: no active device");
        return;
    };
    info!("resetting spectrum");
    session.device = match device.spectrum_reset().await {
        Ok(()) => {
            session.device = Some(device);
            handle_spectrum(events, session, epoch).await;
            return;
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

async fn fetch_spectrum_with_retries(device: &mut RadiaCode) -> radiacode_core::Result<Spectrum> {
    let mut last_error: Option<Error> = None;
    for attempt in 0..=TRANSIENT_RETRIES {
        if attempt > 0 {
            debug!(attempt, "retrying spectrum fetch after transient error");
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        match device.spectrum().await {
            Ok(spectrum) => return Ok(spectrum),
            Err(error) if error.is_transient() => {
                warn!(attempt, %error, "transient spectrum error");
                last_error = Some(error);
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or(Error::timeout()))
}

async fn handle_device_error(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    device: RadiaCode,
    error: Error,
    epoch: &SessionEpoch,
) -> Option<RadiaCode> {
    if should_reconnect(&error, session.session_endpoint.as_ref()) {
        warn!(%error, ?session.session_endpoint, "connection lost during device operation");
        drop(device);
        if epoch.active() {
            return reconnect_and_restore(events, session, epoch).await;
        }
        warn!("skipping reconnect: session ended");
        return None;
    }
    error!(%error, "device operation failed");
    let _ = events.send(WorkerEvent::Error(error.to_string()));
    Some(device)
}

fn should_reconnect(error: &Error, session_endpoint: Option<&DeviceEndpoint>) -> bool {
    session_endpoint.is_some() && (is_connection_lost(error) || error.is_timeout())
}

fn is_connection_lost(error: &Error) -> bool {
    error.is_connection_lost()
        || radiacode_bluetooth::is_connection_lost(error)
        || radiacode_usb::is_connection_lost(error)
}

async fn reconnect_and_restore(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) -> Option<RadiaCode> {
    let endpoint = session.session_endpoint.as_ref()?;
    if !epoch.active() {
        warn!(?endpoint, "reconnect skipped: session ended");
        return None;
    }
    let Some(restore) = session.session_restore.as_ref() else {
        warn!(?endpoint, "reconnect skipped: no cached session");
        let _ = events.send(WorkerEvent::Disconnected);
        return None;
    };
    info!(?endpoint, "attempting reconnect");
    let _ = events.send(WorkerEvent::Reconnecting);
    match reconnect_endpoint(endpoint, epoch, restore).await {
        Ok(mut device) => {
            if !epoch.active() {
                warn!(?endpoint, "reconnect aborted after link up: session ended");
                let _ = device.disconnect().await;
                return None;
            }
            session.link_status = DeviceStatus::default();
            session.data_buf_cursor.reset();
            match load_device_info_with_retry(&mut device, endpoint, None, events).await {
                Ok(info) => {
                    if !epoch.active() {
                        let _ = device.disconnect().await;
                        return None;
                    }
                    session.link_status = DeviceStatus::from(&info);
                    info!(?endpoint, serial = %info.serial, "reconnected");
                    let _ = events.send(WorkerEvent::Connected(info));
                    match fetch_spectrum_with_retries(&mut device).await {
                        Ok(spectrum) => {
                            if epoch.active() {
                                let _ = events.send(WorkerEvent::Spectrum(
                                    SpectrumView::from_spectrum(spectrum),
                                ));
                            }
                            Some(device)
                        }
                        Err(error) => {
                            error!(?endpoint, %error, "spectrum fetch failed after reconnect");
                            let _ = events.send(WorkerEvent::Error(error.to_string()));
                            Some(device)
                        }
                    }
                }
                Err(error) => {
                    error!(?endpoint, %error, "reconnect session restore failed");
                    let _ = device.disconnect().await;
                    let _ = events.send(WorkerEvent::Error(error.to_string()));
                    let _ = events.send(WorkerEvent::Disconnected);
                    None
                }
            }
        }
        Err(error) => {
            if epoch.active() {
                error!(?endpoint, %error, "reconnect failed");
                let _ = events.send(WorkerEvent::Error(error.to_string()));
                let _ = events.send(WorkerEvent::Disconnected);
            } else {
                warn!(?endpoint, "reconnect aborted: session ended");
            }
            None
        }
    }
}

pub async fn handle_monitor(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("monitor fetch skipped: no active device");
        return;
    };
    let limits = match ensure_alarm_limits(
        &mut device,
        &mut session.alarm_limits,
        events,
        session.monitor_polls,
    )
    .await
    {
        Ok(limits) => limits,
        Err(error) => {
            session.device = handle_device_error(events, session, device, error, epoch).await;
            return;
        }
    };
    let refresh_rssi = false;
    session.device = match device
        .poll_monitor(&limits, &mut session.data_buf_cursor, refresh_rssi)
        .await
    {
        Ok((sample, fresh)) => {
            merge_status(&mut session.link_status, fresh);
            if !epoch.active() {
                Some(device)
            } else {
                let has_data = !sample.rates.is_empty() || sample.accumulated.is_some();
                if has_data {
                    session.monitor_polls = session.monitor_polls.saturating_add(1);
                    let _ = events.send(WorkerEvent::MonitorSample(sample));
                } else {
                    debug!("monitor data not yet available in databuf");
                }
                let _ = events.send(WorkerEvent::DeviceStatus(session.link_status));
                let _ = events.send(WorkerEvent::MonitorPollComplete);
                Some(device)
            }
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

pub async fn handle_fetch_device_config(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("device config fetch skipped: no active device");
        return;
    };
    session.device = match device.load_device_config().await {
        Ok(config) => {
            if epoch.active() {
                session.alarm_limits = Some(config.alarms);
                let _ = events.send(WorkerEvent::AlarmLimits(config.alarms));
                let _ = events.send(WorkerEvent::DeviceConfig(config));
            }
            Some(device)
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

pub async fn handle_apply_device_config(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    config: DeviceConfig,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("device config apply skipped: no active device");
        return;
    };
    let apply_error = match device.apply_device_config(&config).await {
        Ok(()) => None,
        Err(error) if should_reconnect(&error, session.session_endpoint.as_ref()) => {
            session.device = handle_device_error(events, session, device, error, epoch).await;
            return;
        }
        Err(error) => {
            error!(%error, "device config apply failed; reloading device state");
            Some(error)
        }
    };
    session.device = match device.load_device_config().await {
        Ok(loaded) => {
            if epoch.active() {
                session.alarm_limits = Some(loaded.alarms);
                let _ = events.send(WorkerEvent::AlarmLimits(loaded.alarms));
                let _ = events.send(WorkerEvent::DeviceConfig(loaded));
                if let Some(error) = apply_error {
                    let _ = events.send(WorkerEvent::Error(error.to_string()));
                }
            }
            Some(device)
        }
        Err(error) => {
            if apply_error.is_none() && epoch.active() {
                session.alarm_limits = Some(config.alarms);
                let _ = events.send(WorkerEvent::AlarmLimits(config.alarms));
                let _ = events.send(WorkerEvent::DeviceConfig(config));
            }
            handle_device_error(events, session, device, error, epoch).await
        }
    };
}

pub async fn handle_sync_device_clock(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("clock sync skipped: no active device");
        return;
    };
    session.device = match device.sync_device_clock().await {
        Ok(()) => {
            if epoch.active() {
                info!("device clock synchronized");
            }
            Some(device)
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

async fn ensure_alarm_limits(
    device: &mut RadiaCode,
    cache: &mut Option<AlarmLimits>,
    events: &Sender<WorkerEvent>,
    monitor_polls: u64,
) -> radiacode_core::Result<AlarmLimits> {
    let refresh = cache.is_none() || monitor_polls.is_multiple_of(ALARM_REFRESH_POLLS);
    if refresh {
        let limits = device.alarm_limits().await?;
        *cache = Some(limits);
        let _ = events.send(WorkerEvent::AlarmLimits(limits));
    }
    Ok(cache.expect("alarm limits cached"))
}

async fn connect_endpoint(
    endpoint: &DeviceEndpoint,
    restore: Option<&SessionRestore>,
) -> radiacode_core::Result<RadiaCode> {
    match endpoint {
        DeviceEndpoint::Bluetooth { address } => {
            if let Some(restore) = restore {
                radiacode_bluetooth::reconnect_session(address, restore).await
            } else {
                radiacode_bluetooth::connect(address).await
            }
        }
        DeviceEndpoint::Usb { serial } => {
            if let Some(restore) = restore {
                radiacode_usb::reconnect_session(serial, restore).await
            } else {
                radiacode_usb::connect(serial).await
            }
        }
    }
}

async fn reconnect_endpoint(
    endpoint: &DeviceEndpoint,
    epoch: &SessionEpoch,
    restore: &SessionRestore,
) -> radiacode_core::Result<RadiaCode> {
    if !epoch.active() {
        return Err(Error::connection_closed());
    }
    connect_endpoint(endpoint, Some(restore)).await
}

#[derive(Clone)]
pub struct SessionEpoch {
    pub live: Arc<AtomicU64>,
    pub started: u64,
}

impl SessionEpoch {
    pub fn active(&self) -> bool {
        self.live.load(Ordering::SeqCst) == self.started
    }
}
