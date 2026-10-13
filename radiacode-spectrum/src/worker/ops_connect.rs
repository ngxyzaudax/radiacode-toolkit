use std::time::Duration;

use crossbeam_channel::Sender;
use radiacode_core::{DeviceEndpoint, Error, RadiaCode};
use tracing::{debug, error, info, warn};

use super::device_info::load_device_info;
use super::epoch::SessionEpoch;
use super::link::connect_endpoint;
use super::messages::WorkerEvent;
use super::session::WorkerSession;
use crate::model::DeviceInfo;

const CONNECT_COOLDOWN: Duration = Duration::from_millis(500);

pub async fn handle_connect(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
    epoch: &SessionEpoch,
) {
    info!(?endpoint, "connecting");
    release_previous_device(session, endpoint).await;
    session.clear_link_state();
    if !epoch.active() {
        warn!(?endpoint, "connect aborted: session ended");
        return;
    }
    match connect_endpoint(endpoint, None).await {
        Ok(device) => finish_connect(events, session, device, endpoint, hint_rssi, epoch).await,
        Err(error) => report_connect_failure(events, endpoint, error),
    }
}

async fn release_previous_device(session: &mut WorkerSession, endpoint: &DeviceEndpoint) {
    if let Some(previous) = session.device.take() {
        debug!(?endpoint, "disconnecting previous session before connect");
        let _ = previous.disconnect().await;
        tokio::time::sleep(CONNECT_COOLDOWN).await;
    }
}

async fn finish_connect(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    mut device: RadiaCode,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
    epoch: &SessionEpoch,
) {
    if !epoch.active() {
        abort_connect(
            device,
            endpoint,
            "connect aborted after link up: session ended",
        )
        .await;
        return;
    }
    match load_device_info(&mut device, endpoint, hint_rssi, events).await {
        Ok(_) if !epoch.active() => {
            abort_connect(
                device,
                endpoint,
                "connect aborted after metadata load: session ended",
            )
            .await;
        }
        Ok(info) => announce_connected(events, session, device, endpoint, info),
        Err(error) => reject_connected_device(events, device, endpoint, error).await,
    }
}

async fn abort_connect(device: RadiaCode, endpoint: &DeviceEndpoint, reason: &str) {
    warn!(?endpoint, reason);
    let _ = device.disconnect().await;
}

fn announce_connected(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    device: RadiaCode,
    endpoint: &DeviceEndpoint,
    info: DeviceInfo,
) {
    session.adopt_device(device, endpoint, &info);
    info!(?endpoint, serial = %info.serial, model = %info.model, "connected");
    let _ = events.send(WorkerEvent::Connected(info));
    let _ = events.send(WorkerEvent::DeviceStatus(session.link_status));
}

async fn reject_connected_device(
    events: &Sender<WorkerEvent>,
    device: RadiaCode,
    endpoint: &DeviceEndpoint,
    error: Error,
) {
    error!(?endpoint, %error, "failed to load device info");
    let _ = device.disconnect().await;
    let _ = events.send(WorkerEvent::Error(error.to_string()));
    let _ = events.send(WorkerEvent::Disconnected);
}

fn report_connect_failure(events: &Sender<WorkerEvent>, endpoint: &DeviceEndpoint, error: Error) {
    error!(?endpoint, %error, "connect failed");
    let _ = events.send(connect_failure_event(endpoint, &error));
    let _ = events.send(WorkerEvent::Disconnected);
}

pub(super) fn connect_failure_event(endpoint: &DeviceEndpoint, error: &Error) -> WorkerEvent {
    if error.is_usb_permission_denied() {
        WorkerEvent::UsbPermissionRequired {
            endpoint: endpoint.clone(),
        }
    } else {
        WorkerEvent::Error(error.to_string())
    }
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
