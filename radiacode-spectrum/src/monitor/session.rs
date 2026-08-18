use std::time::Instant;

use super::state::MonitorState;

pub fn on_connect(state: &mut MonitorState) {
    state.history.clear();
    state.latest = None;
    state.device_epoch_ticks = None;
    state.session_started = Some(Instant::now());
    state.decode_warnings = 0;
    state.rejected_records = 0;
    state.resync_count = 0;
    state.seq_gaps = 0;
    state.lost_records = 0;
    state.status = "Loading monitor data…".into();
}

pub fn on_disconnect(state: &mut MonitorState) {
    state.history.clear();
    state.latest = None;
    state.limits = None;
    state.device_epoch_ticks = None;
    state.session_started = None;
    state.decode_warnings = 0;
    state.rejected_records = 0;
    state.resync_count = 0;
    state.seq_gaps = 0;
    state.lost_records = 0;
    state.status = "Connect a device to start monitoring.".into();
}

pub fn on_reconnecting(state: &mut MonitorState) {
    state.decode_warnings = 0;
    state.rejected_records = 0;
    state.resync_count = 0;
    state.seq_gaps = 0;
    state.lost_records = 0;
}
