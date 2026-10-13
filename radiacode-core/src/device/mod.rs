mod session;
mod session_init;
mod vsfr;

#[cfg(test)]
#[path = "vsfr_tests.rs"]
mod vsfr_tests;

use std::time::SystemTime;

use radiacode_protocol::{DeviceTicks, Sequence, SfrCatalogEntry, Transport};

use crate::error::Result;
use crate::session_restore::SessionRestore;
use radiacode_protocol::DeviceVersions;

pub struct RadiaCode {
    pub(crate) transport: Box<dyn Transport>,
    pub(crate) sequence: Sequence,
    pub(crate) spectrum_format_version: u32,
    cached_versions: Option<DeviceVersions>,
    base_time: SystemTime,
    sfr_entries: Vec<SfrCatalogEntry>,
}

impl RadiaCode {
    pub async fn disconnect(self) -> Result<()> {
        self.transport.disconnect().await.map_err(Into::into)
    }

    pub async fn rssi_dbm(&self) -> Option<i16> {
        self.transport.link_rssi_dbm().await
    }

    pub async fn drain_transport(&mut self) {
        self.transport.drain_link().await;
    }

    pub async fn sample_rssi_dbm(&self) -> Option<i16> {
        self.transport.sample_link_rssi_dbm().await
    }

    pub(crate) fn cached_versions(&self) -> Option<&DeviceVersions> {
        self.cached_versions.as_ref()
    }

    pub fn session_restore(&self) -> Option<SessionRestore> {
        Some(SessionRestore {
            versions: self.cached_versions.as_ref()?.clone(),
            spectrum_format_version: self.spectrum_format_version,
        })
    }

    pub fn spectrum_format_version(&self) -> u32 {
        self.spectrum_format_version
    }

    pub fn base_time(&self) -> SystemTime {
        self.base_time
    }

    pub fn sfr_entries(&self) -> &[SfrCatalogEntry] {
        &self.sfr_entries
    }

    pub fn wall_clock(&self, ticks: DeviceTicks) -> SystemTime {
        ticks.wall_clock_time(self.base_time)
    }
}
