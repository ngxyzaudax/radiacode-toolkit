use std::time::{Duration, SystemTime};

use tracing::{debug, info, warn};

use radiacode_protocol::{Command, VirtSfr, VirtString, parse_sfr_file, validate_catalog};

use crate::error::{Error, Result};
use crate::session_restore::SessionRestore;

use super::RadiaCode;

const INIT_SETTLE: Duration = Duration::from_millis(250);
const BASE_TIME_OFFSET: Duration = Duration::from_secs(128);
const MIN_FIRMWARE: (u16, u16) = (4, 8);

impl RadiaCode {
    pub(super) async fn initialize(
        &mut self,
        ignore_firmware_check: bool,
        restore: Option<&SessionRestore>,
    ) -> Result<()> {
        debug!(
            reconnect = restore.is_some(),
            "initializing radiacode session"
        );
        self.handshake().await?;
        self.sync_clock_and_base_time().await?;
        let sfr_text = self.load_sfr_entries().await;
        self.settle_link().await;
        match restore {
            Some(restore) => {
                self.restore_session(restore);
                Ok(())
            }
            None => self.discover_session(ignore_firmware_check, sfr_text).await,
        }
    }

    async fn handshake(&mut self) -> Result<()> {
        self.execute_raw(Command::SetExchange, &[0x01, 0xff, 0x12, 0xff])
            .await?;
        self.settle_link().await;
        Ok(())
    }

    async fn settle_link(&mut self) {
        self.drain_transport().await;
        tokio::time::sleep(INIT_SETTLE).await;
        self.drain_transport().await;
    }

    async fn sync_clock_and_base_time(&mut self) -> Result<()> {
        crate::device_time::set_local_time_now(self).await?;
        self.write_vsfr(VirtSfr::DeviceTime, &0u32.to_le_bytes())
            .await?;
        self.base_time = SystemTime::now() + BASE_TIME_OFFSET;
        Ok(())
    }

    async fn load_sfr_entries(&mut self) -> Option<String> {
        let sfr_file = self.read_virt_string(VirtString::SfrFile).await.ok()?;
        let sfr_text = String::from_utf8_lossy(sfr_file.data()).into_owned();
        self.sfr_entries = parse_sfr_file(&sfr_text);
        Some(sfr_text)
    }

    fn restore_session(&mut self, restore: &SessionRestore) {
        self.cached_versions = Some(restore.versions.clone());
        self.spectrum_format_version = restore.spectrum_format_version;
        info!(
            major = restore.versions.target.major,
            minor = restore.versions.target.minor,
            spectrum_format_version = self.spectrum_format_version,
            "reconnect session restored from cache"
        );
    }

    async fn discover_session(
        &mut self,
        ignore_firmware_check: bool,
        sfr_text: Option<String>,
    ) -> Result<()> {
        let versions = crate::device_info::fw_version(self).await?;
        let (major, minor) = (versions.target.major, versions.target.minor);
        self.cached_versions = Some(versions);
        info!(major, minor, "device firmware");
        if !ignore_firmware_check && firmware_too_old(major, minor) {
            return Err(Error::IncompatibleFirmware { major, minor });
        }
        let configuration = crate::device_info::configuration(self).await?;
        self.spectrum_format_version = spectrum_format_version_from(&configuration);
        if let Some(text) = sfr_text {
            log_catalog_drift(&text);
        }
        info!(
            spectrum_format_version = self.spectrum_format_version,
            "device initialized"
        );
        Ok(())
    }
}

fn firmware_too_old(major: u16, minor: u16) -> bool {
    (major, minor) < MIN_FIRMWARE
}

fn spectrum_format_version_from(configuration: &str) -> u32 {
    let parsed = radiacode_protocol::parse_configuration_ini(configuration).spec_format_version;
    if parsed != 0 {
        return parsed;
    }
    super::vsfr::parse_spectrum_format_version(configuration)
}

fn log_catalog_drift(sfr_text: &str) {
    match validate_catalog(sfr_text) {
        Some(drifts) => drifts.iter().for_each(|drift| {
            warn!(
                register = ?drift.register,
                message = %drift.message,
                "protocol catalog drift from device SFR_FILE"
            );
        }),
        None => debug!("SFR_FILE validation skipped: no parseable entries"),
    }
}
