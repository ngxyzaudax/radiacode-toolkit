use std::time::{Duration, SystemTime};

use tracing::{debug, info, warn};

use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{
    BytesBuffer, Command, Sequence, Transport, build_request, request_header, strip_echoed_header,
};

use crate::error::{Error, Result};
use crate::session_restore::SessionRestore;

use super::RadiaCode;

const CONNECT_ATTEMPTS: usize = 3;
const RECONNECT_ATTEMPTS: usize = 2;
const CONNECT_RETRY_DELAY: Duration = Duration::from_millis(1500);
const RECONNECT_RETRY_DELAY: Duration = Duration::from_millis(3000);

pub(crate) struct OpenFailure {
    pub error: Error,
    pub transport: Box<dyn Transport>,
}

struct RetryPolicy {
    attempts: usize,
    delay: Duration,
}

fn retry_policy(reconnect: bool) -> RetryPolicy {
    if reconnect {
        RetryPolicy {
            attempts: RECONNECT_ATTEMPTS,
            delay: RECONNECT_RETRY_DELAY,
        }
    } else {
        RetryPolicy {
            attempts: CONNECT_ATTEMPTS,
            delay: CONNECT_RETRY_DELAY,
        }
    }
}

impl RadiaCode {
    pub async fn open(
        transport: Box<dyn Transport>,
        ignore_firmware_check: bool,
        restore: Option<&SessionRestore>,
    ) -> Result<Self> {
        let reconnect = restore.is_some();
        let policy = retry_policy(reconnect);
        info!(ignore_firmware_check, reconnect, "radiacode open");
        let mut last_error: Option<Error> = None;
        let mut transport = transport;
        for attempt in 0..policy.attempts {
            if attempt > 0 {
                warn!(
                    attempt,
                    reconnect, "retrying radiacode open after transient failure"
                );
                tokio::time::sleep(policy.delay).await;
            }
            match Self::try_open_once(transport, ignore_firmware_check, restore).await {
                Ok(device) => return Ok(device),
                Err(failure) if failure.error.is_transient() => {
                    transport = failure.transport;
                    last_error = Some(failure.error);
                }
                Err(failure) => {
                    let _ = failure.transport.disconnect().await;
                    return Err(failure.error);
                }
            }
        }
        let _ = transport.disconnect().await;
        Err(last_error.unwrap_or(ProtocolError::ConnectionClosed.into()))
    }

    async fn try_open_once(
        transport: Box<dyn Transport>,
        ignore_firmware_check: bool,
        restore: Option<&SessionRestore>,
    ) -> std::result::Result<Self, OpenFailure> {
        let sequence = Sequence::session_start();
        debug!(
            seq_start = sequence.start_value(),
            "command sequence initialized"
        );
        let mut device = Self {
            transport,
            sequence,
            spectrum_format_version: restore
                .map(|session| session.spectrum_format_version)
                .unwrap_or(0),
            cached_versions: restore.map(|session| session.versions.clone()),
            base_time: SystemTime::now(),
            sfr_entries: Vec::new(),
        };
        if let Err(error) = device.initialize(ignore_firmware_check, restore).await {
            warn!(%error, "initialize failed, disconnecting partial session");
            let transport = device.transport;
            return Err(OpenFailure { error, transport });
        }
        Ok(device)
    }

    pub async fn execute_raw(&mut self, command: Command, args: &[u8]) -> Result<BytesBuffer> {
        let seq = self.sequence.advance();
        debug!(?command, seq, args_len = args.len(), "execute command");
        let request = build_request(command, seq, args);
        let response = self.transport.execute(&request).await?;
        strip_echoed_header(response, request_header(command, seq)).map_err(Into::into)
    }
}
