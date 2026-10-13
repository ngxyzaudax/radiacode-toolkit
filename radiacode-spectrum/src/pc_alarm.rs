use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tracing::debug;

const MIN_BEEP_INTERVAL: Duration = Duration::from_secs(2);
const FALLBACK_SOUND_PATH: &str = "/usr/share/sounds/freedesktop/stereo/message.oga";

static LAST_BEEP: Mutex<Option<Instant>> = Mutex::new(None);

pub fn beep_allowed(last_beep: Option<Instant>, now: Instant) -> bool {
    last_beep.is_none_or(|instant| now.duration_since(instant) >= MIN_BEEP_INTERVAL)
}

fn claim_beep_slot(now: Instant) -> bool {
    let mut last = LAST_BEEP
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if !beep_allowed(*last, now) {
        return false;
    }
    *last = Some(now);
    true
}

pub fn maybe_beep_alarm(enabled: bool) {
    if !enabled || !claim_beep_slot(Instant::now()) {
        return;
    }
    play_alarm_sound();
}

fn play_alarm_sound() {
    if Command::new("canberra-gtk-play")
        .args(["-i", "message"])
        .spawn()
        .is_ok()
    {
        debug!("pc alarm beep via canberra");
        return;
    }
    let _ = Command::new("paplay").arg(FALLBACK_SOUND_PATH).spawn();
    debug!("pc alarm beep attempted");
}

#[cfg(test)]
mod tests {
    use super::beep_allowed;
    use std::time::{Duration, Instant};

    #[test]
    fn first_beep_is_always_allowed() {
        assert!(beep_allowed(None, Instant::now()));
    }

    #[test]
    fn beeps_are_rate_limited_to_two_seconds() {
        let start = Instant::now();
        assert!(!beep_allowed(
            Some(start),
            start + Duration::from_millis(1999)
        ));
        assert!(beep_allowed(Some(start), start + Duration::from_secs(2)));
    }
}
