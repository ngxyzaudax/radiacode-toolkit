use crossbeam_channel::Sender;
use radiacode_core::{DeviceConfig, Error, RadiaCode};
use tracing::{error, info, warn};

use super::epoch::SessionEpoch;
use super::link::should_reconnect;
use super::messages::WorkerEvent;
use super::ops_reconnect::handle_device_error;
use super::session::WorkerSession;

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
            publish_device_config(events, session, config, epoch);
            Some(device)
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

fn publish_device_config(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    config: DeviceConfig,
    epoch: &SessionEpoch,
) {
    if !epoch.active() {
        return;
    }
    session.alarm_limits = Some(config.alarms);
    let _ = events.send(WorkerEvent::AlarmLimits(config.alarms));
    let _ = events.send(WorkerEvent::DeviceConfig(config));
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
    session.device = reload_after_apply(events, session, device, config, apply_error, epoch).await;
}

async fn reload_after_apply(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    mut device: RadiaCode,
    requested: DeviceConfig,
    apply_error: Option<Error>,
    epoch: &SessionEpoch,
) -> Option<RadiaCode> {
    match device.load_device_config().await {
        Ok(loaded) => {
            publish_device_config(events, session, loaded, epoch);
            if let Some(error) = apply_error.filter(|_| epoch.active()) {
                let _ = events.send(WorkerEvent::Error(error.to_string()));
            }
            Some(device)
        }
        Err(error) => {
            if apply_error.is_none() {
                publish_device_config(events, session, requested, epoch);
            }
            handle_device_error(events, session, device, error, epoch).await
        }
    }
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
