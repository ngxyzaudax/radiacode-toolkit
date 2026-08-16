use std::time::Duration;

use tracing::{debug, info, warn};

use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{
    BytesBuffer, Command, Sequence, Transport, VirtSfr, VirtString, build_request, request_header,
    strip_echoed_header,
};

use crate::error::{Error, Result};
use crate::session_restore::SessionRestore;

use super::RadiaCode;

const CONNECT_ATTEMPTS: usize = 3;
const RECONNECT_ATTEMPTS: usize = 2;
const CONNECT_RETRY_DELAY: Duration = Duration::from_millis(1500);
const RECONNECT_RETRY_DELAY: Duration = Duration::from_millis(3000);
const INIT_SETTLE: Duration = Duration::from_millis(250);

pub(crate) struct OpenFailure {
    pub error: Error,
    pub transport: Box<dyn Transport>,
}

impl RadiaCode {
    pub async fn open(
        transport: Box<dyn Transport>,
        ignore_firmware_check: bool,
        restore: Option<&SessionRestore>,
    ) -> Result<Self> {
        let reconnect = restore.is_some();
        let attempts = if reconnect {
            RECONNECT_ATTEMPTS
        } else {
            CONNECT_ATTEMPTS
        };
        let retry_delay = if reconnect {
            RECONNECT_RETRY_DELAY
        } else {
            CONNECT_RETRY_DELAY
        };
        info!(ignore_firmware_check, reconnect, "radiacode open");
        let mut last_error: Option<Error> = None;
        let mut transport = transport;
        for attempt in 0..attempts {
            if attempt > 0 {
                warn!(
                    attempt,
                    reconnect, "retrying radiacode open after transient failure"
                );
                tokio::time::sleep(retry_delay).await;
            }
            match Self::try_open_once(transport, ignore_firmware_check, restore).await {
                Ok(device) => return Ok(device),
                Err(OpenFailure {
                    error,
                    transport: recovered,
                }) if error.is_transient() => {
                    transport = recovered;
                    last_error = Some(error);
                }
                Err(OpenFailure {
                    error,
                    transport: recovered,
                }) => {
                    let _ = recovered.disconnect().await;
                    return Err(error);
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
        };
        if let Err(error) = device.initialize(ignore_firmware_check, restore).await {
            warn!(%error, "initialize failed, disconnecting partial session");
            let transport = device.transport;
            return Err(OpenFailure { error, transport });
        }
        Ok(device)
    }

    async fn initialize(
        &mut self,
        ignore_firmware_check: bool,
        restore: Option<&SessionRestore>,
    ) -> Result<()> {
        debug!(
            reconnect = restore.is_some(),
            "initializing radiacode session"
        );
        self.execute_raw(Command::SetExchange, &[0x01, 0xff, 0x12, 0xff])
            .await?;
        self.drain_transport().await;
        tokio::time::sleep(INIT_SETTLE).await;
        self.drain_transport().await;
        crate::device_time::set_local_time_now(self).await?;
        self.write_vsfr(VirtSfr::DeviceTime, &0u32.to_le_bytes())
            .await?;
        self.drain_transport().await;
        tokio::time::sleep(INIT_SETTLE).await;
        self.drain_transport().await;

        if let Some(restore) = restore {
            self.cached_versions = Some(restore.versions.clone());
            self.spectrum_format_version = restore.spectrum_format_version;
            let major = restore.versions.target.major;
            let minor = restore.versions.target.minor;
            info!(
                major,
                minor,
                spectrum_format_version = self.spectrum_format_version,
                "reconnect session restored from cache"
            );
            return Ok(());
        }

        let versions = crate::device_info::fw_version(self).await?;
        self.cached_versions = Some(versions.clone());
        let major = versions.target.major;
        let minor = versions.target.minor;
        info!(major, minor, "device firmware");
        if !ignore_firmware_check && (major < 4 || (major == 4 && minor < 8)) {
            return Err(Error::IncompatibleFirmware { major, minor });
        }

        let configuration = crate::device_info::configuration(self).await?;
        self.spectrum_format_version =
            radiacode_protocol::parse_configuration_ini(&configuration).spec_format_version;
        if self.spectrum_format_version == 0 {
            self.spectrum_format_version =
                super::vsfr::parse_spectrum_format_version(&configuration);
        }
        if let Ok(sfr_file) = self.read_virt_string(VirtString::SfrFile).await {
            let sfr_text = String::from_utf8_lossy(sfr_file.data());
            match radiacode_protocol::validate_catalog(&sfr_text) {
                Some(drifts) => {
                    for drift in drifts {
                        warn!(
                            register = ?drift.register,
                            message = %drift.message,
                            "protocol catalog drift from device SFR_FILE"
                        );
                    }
                }
                None => debug!("SFR_FILE validation skipped: no parseable entries"),
            }
        }
        info!(
            spectrum_format_version = self.spectrum_format_version,
            "device initialized"
        );
        Ok(())
    }

    pub async fn execute_raw(&mut self, command: Command, args: &[u8]) -> Result<BytesBuffer> {
        let seq = self.sequence.advance();
        debug!(?command, seq, args_len = args.len(), "execute command");
        let request = build_request(command, seq, args);
        let response = self.transport.execute(&request).await?;
        strip_echoed_header(response, request_header(command, seq)).map_err(Into::into)
    }
}
