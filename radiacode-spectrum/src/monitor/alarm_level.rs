use crate::monitor::state::AlarmLevel;

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
