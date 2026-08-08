use crate::monitor::AlarmLevel;

pub fn format_session_duration(secs: u32) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let minutes = (secs % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

pub fn alarm_level(value: Option<f32>, limits: Option<(f32, f32)>) -> AlarmLevel {
    let Some(value) = value else {
        return AlarmLevel::Normal;
    };
    let Some((l1, l2)) = limits else {
        return AlarmLevel::Normal;
    };
    if value >= l2 {
        AlarmLevel::Danger
    } else if value >= l1 {
        AlarmLevel::Warning
    } else {
        AlarmLevel::Normal
    }
}
