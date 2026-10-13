use radiacode_core::{Error, RadiaCode, SessionRestore};
use tracing::info;

use crate::usb_error::map_usb_error;

use super::UsbTransport;

pub async fn connect(serial: &str) -> radiacode_core::Result<RadiaCode> {
    let transport = open_transport(serial).await?;
    RadiaCode::open(Box::new(transport), false, None).await
}

pub async fn reconnect_session(
    serial: &str,
    restore: &SessionRestore,
) -> radiacode_core::Result<RadiaCode> {
    info!(%serial, "radiacode usb reconnect with cached session");
    let transport = open_transport(serial).await?;
    RadiaCode::open(Box::new(transport), false, Some(restore)).await
}

async fn open_transport(serial: &str) -> radiacode_core::Result<UsbTransport> {
    let serial = serial.to_string();
    tokio::task::spawn_blocking(move || UsbTransport::connect(&serial).map_err(map_usb_error))
        .await
        .map_err(|error| {
            Error::from(radiacode_protocol::Error::TransportUnavailable(
                error.to_string(),
            ))
        })?
}
