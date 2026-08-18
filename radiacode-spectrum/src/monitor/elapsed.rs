use std::time::Duration;

const WALL_SLACK: Duration = Duration::from_millis(750);

pub fn resolve_elapsed(
    device_from_epoch: Duration,
    wall_from_start: Duration,
    last_elapsed: Option<Duration>,
) -> Duration {
    let capped = device_from_epoch.min(wall_from_start.saturating_add(WALL_SLACK));
    match last_elapsed {
        None => capped,
        Some(last) if capped > last => capped,
        Some(last) => last,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_elapsed;
    use std::time::Duration;

    #[test]
    fn caps_device_jump_to_wall_time() {
        let elapsed = resolve_elapsed(
            Duration::from_secs(180),
            Duration::from_secs(105),
            Some(Duration::from_secs(100)),
        );
        assert_eq!(elapsed, Duration::from_millis(105_750));
    }

    #[test]
    fn keeps_monotonic_when_device_rewinds() {
        let elapsed = resolve_elapsed(
            Duration::from_secs(1),
            Duration::from_secs(10),
            Some(Duration::from_secs(8)),
        );
        assert_eq!(elapsed, Duration::from_secs(8));
    }

    #[test]
    fn follows_device_when_within_wall() {
        let elapsed = resolve_elapsed(
            Duration::from_secs(12),
            Duration::from_secs(12),
            Some(Duration::from_secs(11)),
        );
        assert_eq!(elapsed, Duration::from_secs(12));
    }
}
