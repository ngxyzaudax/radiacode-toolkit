pub fn pause_enabled(connected: bool, recording: bool, capture_paused: bool) -> bool {
    connected && recording && !capture_paused
}

pub fn reset_enabled(recording: bool) -> bool {
    !recording
}

#[cfg(test)]
mod tests {
    use super::{pause_enabled, reset_enabled};

    #[test]
    fn pause_only_while_recording_and_not_paused() {
        assert!(pause_enabled(true, true, false));
        assert!(!pause_enabled(true, false, false));
        assert!(!pause_enabled(true, true, true));
        assert!(!pause_enabled(false, true, false));
    }

    #[test]
    fn reset_disabled_while_recording() {
        assert!(reset_enabled(false));
        assert!(!reset_enabled(true));
    }
}
