use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("response timed out")]
    Timeout,
    #[error("connection closed")]
    ConnectionClosed,
    #[error("protocol mismatch: expected {expected}, got {got}")]
    ProtocolMismatch { expected: String, got: String },
    #[error("unexpected device return code {0}")]
    UnexpectedReturnCode(u32),
    #[error("buffer underrun: need {need} bytes, have {have}")]
    BufferUnderrun { need: usize, have: usize },
    #[error("vsfr batch returned no readable registers")]
    VsfrBatchEmpty,
    #[error("transport unavailable: {0}")]
    TransportUnavailable(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::Timeout | Self::ProtocolMismatch { .. } | Self::BufferUnderrun { .. }
        )
    }

    pub fn is_connection_lost(&self) -> bool {
        matches!(self, Self::ConnectionClosed)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn timeout_is_transient_not_connection_lost() {
        assert!(Error::Timeout.is_transient());
        assert!(!Error::Timeout.is_connection_lost());
    }

    #[test]
    fn connection_closed_is_link_loss() {
        assert!(Error::ConnectionClosed.is_connection_lost());
        assert!(!Error::ConnectionClosed.is_transient());
    }
}
