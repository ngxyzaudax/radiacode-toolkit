pub fn format_hms(secs: f64) -> String {
    let total = secs.max(0.0).round() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::format_hms;

    #[test]
    fn zero_is_midnight() {
        assert_eq!(format_hms(0.0), "00:00:00");
    }

    #[test]
    fn sub_minute() {
        assert_eq!(format_hms(45.4), "00:00:45");
    }

    #[test]
    fn multi_hour() {
        assert_eq!(format_hms(3661.0), "01:01:01");
    }

    #[test]
    fn negative_clamps_to_zero() {
        assert_eq!(format_hms(-10.0), "00:00:00");
    }
}
