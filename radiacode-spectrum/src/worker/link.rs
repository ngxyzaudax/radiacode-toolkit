use radiacode_lib::bluetooth::{BluetoothTransport, retry_connect};
use radiacode_lib::{
    DeviceEndpoint, Error, OpenOptions, RadiaCode, Result, SessionRestore, connect,
};

use super::epoch::SessionEpoch;
use crate::ble_rssi::SamplingBluetoothTransport;

pub async fn connect_endpoint(
    endpoint: &DeviceEndpoint,
    restore: Option<&SessionRestore>,
) -> Result<RadiaCode> {
    let options = open_options(restore);
    match endpoint {
        DeviceEndpoint::Bluetooth { address } => {
            open_bluetooth(address, restore.is_some(), options).await
        }
        DeviceEndpoint::Usb { .. } => connect(endpoint, options).await,
    }
}

fn open_options(restore: Option<&SessionRestore>) -> OpenOptions {
    match restore {
        Some(restore) => OpenOptions::default().restore(restore.clone()),
        None => OpenOptions::default(),
    }
}

async fn open_bluetooth(address: &str, fresh: bool, options: OpenOptions) -> Result<RadiaCode> {
    let address = address.to_string();
    let transport = retry_connect(fresh, move |use_fresh| {
        let address = address.clone();
        async move { open_bluetooth_once(&address, use_fresh).await }
    })
    .await?;
    RadiaCode::open(
        Box::new(SamplingBluetoothTransport::new(transport)),
        options,
    )
    .await
}

async fn open_bluetooth_once(address: &str, fresh: bool) -> Result<BluetoothTransport> {
    match fresh {
        true => BluetoothTransport::connect_fresh_once(address).await,
        false => BluetoothTransport::connect_once(address).await,
    }
}

pub async fn reconnect_endpoint(
    endpoint: &DeviceEndpoint,
    epoch: &SessionEpoch,
    restore: &SessionRestore,
) -> Result<RadiaCode> {
    if !epoch.active() {
        return Err(Error::ConnectionClosed);
    }
    connect_endpoint(endpoint, Some(restore)).await
}

pub fn is_connection_lost(error: &Error) -> bool {
    error.is_connection_lost() || error.is_usb_permission_denied()
}

pub fn should_reconnect(error: &Error, session_endpoint: Option<&DeviceEndpoint>) -> bool {
    session_endpoint.is_some() && (is_connection_lost(error) || error.is_timeout())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    use super::{reconnect_endpoint, should_reconnect};
    use crate::test_device::{VirtualDevice, open_device};
    use crate::worker::epoch::{SessionEpoch, end_session};
    use radiacode_lib::{DeviceEndpoint, Error};

    fn endpoint() -> DeviceEndpoint {
        DeviceEndpoint::Usb {
            serial: "RC-1".to_string(),
        }
    }

    #[test]
    fn reconnect_requires_session_endpoint() {
        assert!(!should_reconnect(&Error::ConnectionClosed, None));
        assert!(should_reconnect(
            &Error::ConnectionClosed,
            Some(&endpoint())
        ));
    }

    #[test]
    fn timeouts_trigger_reconnect_but_protocol_errors_do_not() {
        assert!(should_reconnect(&Error::Timeout, Some(&endpoint())));
        assert!(!should_reconnect(
            &Error::MonitorDataPending,
            Some(&endpoint())
        ));
        assert!(!should_reconnect(&Error::DeviceNotFound, Some(&endpoint())));
    }

    #[tokio::test(start_paused = true)]
    async fn fast_reconnect_retries_device_not_found_then_scans_fresh() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        use radiacode_lib::bluetooth::retry_connect;

        let calls = AtomicUsize::new(0);
        let outcome = retry_connect(false, |fresh| {
            let index = calls.fetch_add(1, Ordering::SeqCst);
            async move {
                match (index, fresh) {
                    (0, false) => Err(Error::DeviceNotFound),
                    (1, true) => Ok("open"),
                    _ => Err(Error::InvalidArgument("extra attempt".into())),
                }
            }
        })
        .await;
        assert_eq!(outcome.expect("retried connect"), "open");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn reconnect_is_refused_once_session_ended() {
        let live = Arc::new(AtomicU64::new(0));
        let epoch = SessionEpoch::capture(&live);
        end_session(&live);
        let (device, _handle) = open_device(VirtualDevice::default()).await;
        let restore = device.session_restore().expect("session restore");
        let outcome = reconnect_endpoint(&endpoint(), &epoch, &restore).await;
        assert!(matches!(outcome, Err(error) if error.is_connection_lost()));
    }
}
