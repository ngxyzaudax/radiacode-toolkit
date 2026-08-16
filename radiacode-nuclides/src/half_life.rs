const SECS_PER_NS: f64 = 1e-9;
const SECS_PER_US: f64 = 1e-6;
const SECS_PER_MS: f64 = 1e-3;
const SECS_PER_MIN: f64 = 60.0;
const SECS_PER_HOUR: f64 = 3600.0;
const SECS_PER_DAY: f64 = 86400.0;
const SECS_PER_YEAR: f64 = 31_557_600.0;
const SCI_NOTATION_YEAR_THRESHOLD: f64 = 10_000.0;

pub fn format_half_life(secs: Option<f64>) -> String {
    let Some(secs) = secs else {
        return "stable".to_string();
    };
    if !secs.is_finite() || secs <= 0.0 {
        return "stable".to_string();
    }
    format_positive_secs(secs)
}

fn format_positive_secs(secs: f64) -> String {
    let units: [(&str, f64); 8] = [
        ("y", SECS_PER_YEAR),
        ("d", SECS_PER_DAY),
        ("h", SECS_PER_HOUR),
        ("m", SECS_PER_MIN),
        ("s", 1.0),
        ("ms", SECS_PER_MS),
        ("us", SECS_PER_US),
        ("ns", SECS_PER_NS),
    ];
    for (label, unit_secs) in units {
        if secs >= unit_secs {
            let value = secs / unit_secs;
            if label == "y" && value >= SCI_NOTATION_YEAR_THRESHOLD {
                return format!("{value:.3e} y");
            }
            return format!("{} {label}", format_value(value));
        }
    }
    format!("{} ns", format_value(secs / SECS_PER_NS))
}

fn format_value(value: f64) -> String {
    if value >= 10.0 {
        format!("{value:.1}")
    } else if value >= 1.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_stable() {
        assert_eq!(format_half_life(None), "stable");
    }

    #[test]
    fn formats_am241_years() {
        let secs = 432.6 * SECS_PER_YEAR;
        assert_eq!(format_half_life(Some(secs)), "432.6 y");
    }

    #[test]
    fn formats_cs137_years() {
        let secs = 30.08 * SECS_PER_YEAR;
        assert_eq!(format_half_life(Some(secs)), "30.1 y");
    }

    #[test]
    fn formats_u238_scientific() {
        let secs = 4.468e9 * SECS_PER_YEAR;
        let text = format_half_life(Some(secs));
        assert!(text.contains("e") && text.ends_with(" y"));
    }
}
