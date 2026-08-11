use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Protocol(#[from] radiacode_protocol::Error),
    #[error("device not found")]
    DeviceNotFound,
    #[error("incompatible firmware {major}.{minor}, >=4.8 required")]
    IncompatibleFirmware { major: u16, minor: u16 },
    #[error("live rates not yet available in device buffer")]
    MonitorDataPending,
    #[error("usb permission denied; install radiacode.rules udev rule and replug device")]
    UsbPermissionDenied,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn timeout() -> Self {
        Self::Protocol(radiacode_protocol::Error::Timeout)
    }

    pub fn connection_closed() -> Self {
        Self::Protocol(radiacode_protocol::Error::ConnectionClosed)
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Protocol(radiacode_protocol::Error::Timeout))
    }

    pub fn is_transient(&self) -> bool {
        match self {
            Self::Protocol(error) => error.is_transient(),
            _ => false,
        }
    }

    pub fn is_connection_lost(&self) -> bool {
        match self {
            Self::Protocol(error) => error.is_connection_lost(),
            _ => false,
        }
    }

    pub fn is_usb_permission_denied(&self) -> bool {
        matches!(self, Self::UsbPermissionDenied)
    }
}

pub fn protocol_error(error: radiacode_protocol::Error) -> Error {
    Error::Protocol(error)
}

#[cfg(test)]
mod tests {
    use super::Error;
    use radiacode_protocol::Error as ProtocolError;

    #[test]
    fn timeout_is_transient_not_connection_lost() {
        let error = Error::Protocol(ProtocolError::Timeout);
        assert!(error.is_transient());
        assert!(!error.is_connection_lost());
    }

    #[test]
    fn connection_closed_is_link_loss() {
        let error = Error::Protocol(ProtocolError::ConnectionClosed);
        assert!(error.is_connection_lost());
        assert!(!error.is_transient());
    }
}
