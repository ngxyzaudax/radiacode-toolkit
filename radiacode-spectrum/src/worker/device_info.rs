use std::time::Duration;

use crossbeam_channel::Sender;
use radiacode_lib::{DeviceEndpoint, DeviceStatus, RadiaCode, RssiStatus};
use tracing::{debug, warn};

use super::messages::WorkerEvent;
use crate::model::DeviceInfo;

const INFO_RETRY_DELAY: Duration = Duration::from_millis(500);

pub async fn load_device_info_with_retry(
    device: &mut RadiaCode,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
    events: &Sender<WorkerEvent>,
) -> radiacode_lib::Result<DeviceInfo> {
    match load_device_info(device, endpoint, hint_rssi, events).await {
        Err(error) if error.is_transient() => {
            warn!(%error, "transient device info load failed, retrying once");
            tokio::time::sleep(INFO_RETRY_DELAY).await;
            load_device_info(device, endpoint, hint_rssi, events).await
        }
        outcome => outcome,
    }
}

pub async fn load_device_info(
    device: &mut RadiaCode,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
    events: &Sender<WorkerEvent>,
) -> radiacode_lib::Result<DeviceInfo> {
    debug!(?endpoint, "loading device metadata");
    let metadata = device.metadata().await?;
    let status = load_link_status(device, endpoint, hint_rssi).await;
    publish_alarm_limits(device, events).await;
    Ok(DeviceInfo::from_metadata(metadata, endpoint, status))
}

async fn load_link_status(
    device: &mut RadiaCode,
    endpoint: &DeviceEndpoint,
    hint_rssi: Option<i16>,
) -> DeviceStatus {
    let refresh_rssi = matches!(endpoint, DeviceEndpoint::Bluetooth { .. });
    let mut status = device.device_status(refresh_rssi).await.unwrap_or_default();
    if refresh_rssi && status.rssi == RssiStatus::NotReported {
        status.rssi = advertised_rssi(device.sample_rssi().await, hint_rssi);
    }
    status
}

pub(super) fn advertised_rssi(sampled: RssiStatus, hint: Option<i16>) -> RssiStatus {
    match sampled {
        RssiStatus::NotReported => hint.map(RssiStatus::Measured).unwrap_or_default(),
        other => other,
    }
}

async fn publish_alarm_limits(device: &mut RadiaCode, events: &Sender<WorkerEvent>) {
    if let Ok(limits) = device.alarm_limits().await {
        let _ = events.send(WorkerEvent::AlarmLimits(limits));
    }
}
