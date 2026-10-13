use crossbeam_channel::Sender;
use radiacode_core::{DeviceEndpoint, DeviceStatus, Error, RadiaCode};
use tracing::{error, info, warn};

use super::device_info::load_device_info_with_retry;
use super::epoch::SessionEpoch;
use super::link::{reconnect_endpoint, should_reconnect};
use super::messages::WorkerEvent;
use super::ops_spectrum::fetch_spectrum_with_retries;
use super::session::WorkerSession;
use crate::model::SpectrumView;

pub async fn handle_device_error(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    device: RadiaCode,
    error: Error,
    epoch: &SessionEpoch,
) -> Option<RadiaCode> {
    if !should_reconnect(&error, session.session_endpoint.as_ref()) {
        error!(%error, "device operation failed");
        let _ = events.send(WorkerEvent::Error(error.to_string()));
        return Some(device);
    }
    warn!(%error, ?session.session_endpoint, "connection lost during device operation");
    drop(device);
    if !epoch.active() {
        warn!("skipping reconnect: session ended");
        return None;
    }
    reconnect_and_restore(events, session, epoch).await
}

async fn reconnect_and_restore(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) -> Option<RadiaCode> {
    let endpoint = session.session_endpoint.clone()?;
    let Some(restore) = session.session_restore.clone() else {
        warn!(?endpoint, "reconnect skipped: no cached session");
        let _ = events.send(WorkerEvent::Disconnected);
        return None;
    };
    info!(?endpoint, "attempting reconnect");
    let _ = events.send(WorkerEvent::Reconnecting);
    match reconnect_endpoint(&endpoint, epoch, &restore).await {
        Ok(device) => restore_after_link_up(events, session, device, &endpoint, epoch).await,
        Err(error) => report_reconnect_failure(events, &endpoint, error, epoch),
    }
}

fn report_reconnect_failure(
    events: &Sender<WorkerEvent>,
    endpoint: &DeviceEndpoint,
    error: Error,
    epoch: &SessionEpoch,
) -> Option<RadiaCode> {
    if !epoch.active() {
        warn!(?endpoint, "reconnect aborted: session ended");
        return None;
    }
    error!(?endpoint, %error, "reconnect failed");
    let _ = events.send(WorkerEvent::Error(error.to_string()));
    let _ = events.send(WorkerEvent::Disconnected);
    None
}

async fn restore_after_link_up(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    mut device: RadiaCode,
    endpoint: &DeviceEndpoint,
    epoch: &SessionEpoch,
) -> Option<RadiaCode> {
    if !epoch.active() {
        warn!(?endpoint, "reconnect aborted after link up: session ended");
        let _ = device.disconnect().await;
        return None;
    }
    session.link_status = DeviceStatus::default();
    session.data_buf_cursor.reset();
    match load_device_info_with_retry(&mut device, endpoint, None, events).await {
        Ok(_) if !epoch.active() => {
            let _ = device.disconnect().await;
            None
        }
        Ok(info) => {
            session.link_status = DeviceStatus::from(&info);
            info!(?endpoint, serial = %info.serial, "reconnected");
            let _ = events.send(WorkerEvent::Connected(info));
            Some(refresh_spectrum_after_reconnect(events, device, endpoint, epoch).await)
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

async fn refresh_spectrum_after_reconnect(
    events: &Sender<WorkerEvent>,
    mut device: RadiaCode,
    endpoint: &DeviceEndpoint,
    epoch: &SessionEpoch,
) -> RadiaCode {
    match fetch_spectrum_with_retries(&mut device).await {
        Ok(spectrum) if epoch.active() => {
            let _ = events.send(WorkerEvent::Spectrum(SpectrumView::from_spectrum(spectrum)));
        }
        Ok(_) => {}
        Err(error) => {
            error!(?endpoint, %error, "spectrum fetch failed after reconnect");
            let _ = events.send(WorkerEvent::Error(error.to_string()));
        }
    }
    device
}
