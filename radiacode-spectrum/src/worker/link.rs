use radiacode_core::{DeviceEndpoint, Error, RadiaCode, SessionRestore};

use super::epoch::SessionEpoch;

pub async fn connect_endpoint(
    endpoint: &DeviceEndpoint,
    restore: Option<&SessionRestore>,
) -> radiacode_core::Result<RadiaCode> {
    match (endpoint, restore) {
        (DeviceEndpoint::Bluetooth { address }, Some(restore)) => {
            radiacode_bluetooth::reconnect_session(address, restore).await
        }
        (DeviceEndpoint::Bluetooth { address }, None) => {
            radiacode_bluetooth::connect(address).await
        }
        (DeviceEndpoint::Usb { serial }, Some(restore)) => {
            radiacode_usb::reconnect_session(serial, restore).await
        }
        (DeviceEndpoint::Usb { serial }, None) => radiacode_usb::connect(serial).await,
    }
}

pub async fn reconnect_endpoint(
    endpoint: &DeviceEndpoint,
    epoch: &SessionEpoch,
    restore: &SessionRestore,
) -> radiacode_core::Result<RadiaCode> {
    if !epoch.active() {
        return Err(Error::connection_closed());
    }
    connect_endpoint(endpoint, Some(restore)).await
}

pub fn is_connection_lost(error: &Error) -> bool {
    error.is_connection_lost()
        || radiacode_bluetooth::is_connection_lost(error)
        || radiacode_usb::is_connection_lost(error)
}

pub fn should_reconnect(error: &Error, session_endpoint: Option<&DeviceEndpoint>) -> bool {
    session_endpoint.is_some() && (is_connection_lost(error) || error.is_timeout())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    use super::{reconnect_endpoint, should_reconnect};
    use crate::worker::epoch::{SessionEpoch, end_session};
    use radiacode_core::test_support::VirtualDevice;
    use radiacode_core::test_support::open_device;
    use radiacode_core::{DeviceEndpoint, Error};

    fn endpoint() -> DeviceEndpoint {
        DeviceEndpoint::Usb {
            serial: "RC-1".to_string(),
        }
    }

    #[test]
    fn reconnect_requires_session_endpoint() {
        assert!(!should_reconnect(&Error::connection_closed(), None));
        assert!(should_reconnect(
            &Error::connection_closed(),
            Some(&endpoint())
        ));
    }

    #[test]
    fn timeouts_trigger_reconnect_but_protocol_errors_do_not() {
        assert!(should_reconnect(&Error::timeout(), Some(&endpoint())));
        assert!(!should_reconnect(
            &Error::MonitorDataPending,
            Some(&endpoint())
        ));
        assert!(!should_reconnect(&Error::DeviceNotFound, Some(&endpoint())));
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
