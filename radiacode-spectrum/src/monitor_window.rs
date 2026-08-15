pub const MONITOR_WINDOW_PRESET_MINUTES: [u32; 10] = [1, 2, 3, 5, 10, 15, 20, 30, 45, 60];

pub fn snap_window_minutes(minutes: u32) -> u32 {
    MONITOR_WINDOW_PRESET_MINUTES
        .iter()
        .copied()
        .min_by_key(|preset| minutes.abs_diff(*preset))
        .unwrap_or(MONITOR_WINDOW_PRESET_MINUTES[0])
}

pub fn window_preset_index(minutes: u32) -> usize {
    let snapped = snap_window_minutes(minutes);
    MONITOR_WINDOW_PRESET_MINUTES
        .iter()
        .position(|preset| *preset == snapped)
        .unwrap_or(0)
}

pub fn window_preset_minutes(index: usize) -> u32 {
    MONITOR_WINDOW_PRESET_MINUTES
        .get(index)
        .copied()
        .unwrap_or(MONITOR_WINDOW_PRESET_MINUTES[0])
}

pub fn window_preset_count() -> usize {
    MONITOR_WINDOW_PRESET_MINUTES.len()
}

#[cfg(test)]
mod tests {
    use super::{snap_window_minutes, window_preset_index, window_preset_minutes};

    #[test]
    fn snaps_unknown_values_to_nearest_preset() {
        assert_eq!(snap_window_minutes(4), 3);
        assert_eq!(snap_window_minutes(60), 60);
    }

    #[test]
    fn preset_index_round_trips() {
        for index in 0..super::window_preset_count() {
            let minutes = window_preset_minutes(index);
            assert_eq!(window_preset_index(minutes), index);
        }
    }
}
