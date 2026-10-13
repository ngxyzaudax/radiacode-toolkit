use radiacode_core::SessionRestore;
use radiacode_core::{AlarmLimits, DataBufCursor, DeviceEndpoint, DeviceStatus, RadiaCode};

use crate::model::DeviceInfo;

#[derive(Default)]
pub struct WorkerSession {
    pub device: Option<RadiaCode>,
    pub session_endpoint: Option<DeviceEndpoint>,
    pub alarm_limits: Option<AlarmLimits>,
    pub monitor_polls: u64,
    pub data_buf_cursor: DataBufCursor,
    pub link_status: DeviceStatus,
    pub session_restore: Option<SessionRestore>,
}

impl WorkerSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_link_state(&mut self) {
        self.session_endpoint = None;
        self.alarm_limits = None;
        self.monitor_polls = 0;
        self.data_buf_cursor.reset();
        self.link_status = DeviceStatus::default();
        self.session_restore = None;
    }

    pub fn reset(&mut self) {
        self.clear_link_state();
        self.device = None;
    }

    pub fn clear_if_device_lost(&mut self) {
        if self.device.is_none() {
            self.reset();
        }
    }

    pub fn adopt_device(
        &mut self,
        device: RadiaCode,
        endpoint: &DeviceEndpoint,
        info: &DeviceInfo,
    ) {
        self.link_status = DeviceStatus::from(info);
        self.session_restore = device.session_restore();
        self.device = Some(device);
        self.session_endpoint = Some(endpoint.clone());
    }

    pub fn has_device(&self) -> bool {
        self.device.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::WorkerSession;
    use radiacode_core::DeviceStatus;

    fn populated_session() -> WorkerSession {
        let mut session = WorkerSession::new();
        session.link_status = DeviceStatus {
            battery_percent: Some(80.0),
            temperature_c: Some(25.0),
            rssi_dbm: Some(-50),
        };
        session.monitor_polls = 5;
        session
    }

    #[test]
    fn reset_clears_link_status_and_counters() {
        let mut session = populated_session();
        session.reset();
        assert_eq!(session.link_status, DeviceStatus::default());
        assert_eq!(session.monitor_polls, 0);
        assert!(session.device.is_none());
        assert!(session.session_endpoint.is_none());
        assert!(!session.has_device());
    }

    #[test]
    fn clear_if_device_lost_without_device_resets_everything() {
        let mut session = populated_session();
        session.clear_if_device_lost();
        assert_eq!(session.monitor_polls, 0);
        assert_eq!(session.link_status, DeviceStatus::default());
    }
}
