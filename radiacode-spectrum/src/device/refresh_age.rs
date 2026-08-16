use std::time::{Duration, Instant};

pub fn refresh_age_label(instant: Option<Instant>) -> String {
    let Some(instant) = instant else {
        return "never".into();
    };
    let elapsed = instant.elapsed();
    if elapsed < Duration::from_secs(2) {
        "just now".into()
    } else if elapsed < Duration::from_secs(60) {
        format!("{}s ago", elapsed.as_secs())
    } else {
        format!("{}m ago", elapsed.as_secs() / 60)
    }
}
