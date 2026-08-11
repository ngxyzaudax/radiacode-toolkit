use crate::units::{CountDisplayUnit, DoseDisplayUnit, RawCountsPer10s, RawMicroRoentgenPerHour};

pub fn dose_display_from_rh(dose_rate_rh: f32, unit: DoseDisplayUnit) -> f32 {
    match unit {
        DoseDisplayUnit::MicroSievertPerHour => dose_rate_rh * 10_000.0,
        DoseDisplayUnit::MicroRoentgenPerHour => dose_rate_rh * 1_000_000.0,
    }
}

pub fn count_display_from_cps(count_cps: f32, unit: CountDisplayUnit) -> f32 {
    match unit {
        CountDisplayUnit::Cpm => count_cps * 60.0,
        CountDisplayUnit::Cps => count_cps,
    }
}

pub fn encode_dose_alarm(display: f32, unit: DoseDisplayUnit) -> RawMicroRoentgenPerHour {
    let multiplier = match unit {
        DoseDisplayUnit::MicroSievertPerHour => 100.0,
        DoseDisplayUnit::MicroRoentgenPerHour => 1.0,
    };
    RawMicroRoentgenPerHour::new((display * multiplier).round().max(0.0) as u32)
}

pub fn decode_dose_alarm(raw: RawMicroRoentgenPerHour, unit: DoseDisplayUnit) -> f32 {
    let divisor = match unit {
        DoseDisplayUnit::MicroSievertPerHour => 100.0,
        DoseDisplayUnit::MicroRoentgenPerHour => 1.0,
    };
    raw.as_u32() as f32 / divisor
}

pub fn encode_dose_accum(display_micro: f32, unit: DoseDisplayUnit) -> u32 {
    let multiplier = match unit {
        DoseDisplayUnit::MicroSievertPerHour => 100.0,
        DoseDisplayUnit::MicroRoentgenPerHour => 1.0,
    };
    (display_micro * multiplier).round().max(0.0) as u32
}

pub fn decode_dose_accum(raw: u32, unit: DoseDisplayUnit) -> f32 {
    let divisor = match unit {
        DoseDisplayUnit::MicroSievertPerHour => 100.0,
        DoseDisplayUnit::MicroRoentgenPerHour => 1.0,
    };
    raw as f32 / divisor
}

pub fn encode_count_alarm(display: f32, unit: CountDisplayUnit) -> RawCountsPer10s {
    let multiplier = match unit {
        CountDisplayUnit::Cpm => 1.0 / 6.0,
        CountDisplayUnit::Cps => 10.0,
    };
    RawCountsPer10s::new((display * multiplier).round().max(0.0) as u32)
}

pub fn decode_count_alarm(raw: RawCountsPer10s, unit: CountDisplayUnit) -> f32 {
    let multiplier = match unit {
        CountDisplayUnit::Cpm => 60.0,
        CountDisplayUnit::Cps => 1.0,
    };
    raw.as_u32() as f32 / 10.0 * multiplier
}

pub fn dose_display_from_accum_r(dose_r: f32, unit: DoseDisplayUnit) -> f32 {
    dose_display_from_rh(dose_r, unit)
}

#[cfg(test)]
mod tests {
    use super::{
        count_display_from_cps, decode_count_alarm, decode_dose_alarm, dose_display_from_rh,
        encode_count_alarm, encode_dose_alarm,
    };
    use crate::units::{CountDisplayUnit, DoseDisplayUnit};

    #[test]
    fn dose_sv_round_trip() {
        let display = 1.25;
        let raw = encode_dose_alarm(display, DoseDisplayUnit::MicroSievertPerHour);
        assert_eq!(raw.as_u32(), 125);
        assert!((decode_dose_alarm(raw, DoseDisplayUnit::MicroSievertPerHour) - display).abs() < 0.01);
    }

    #[test]
    fn dose_r_round_trip() {
        let display = 125.0;
        let raw = encode_dose_alarm(display, DoseDisplayUnit::MicroRoentgenPerHour);
        assert_eq!(raw.as_u32(), 125);
        assert!(
            (decode_dose_alarm(raw, DoseDisplayUnit::MicroRoentgenPerHour) - display).abs() < 0.01
        );
    }

    #[test]
    fn count_cps_round_trip() {
        let display = 42.0;
        let raw = encode_count_alarm(display, CountDisplayUnit::Cps);
        assert_eq!(raw.as_u32(), 420);
        assert!((decode_count_alarm(raw, CountDisplayUnit::Cps) - display).abs() < 0.01);
    }

    #[test]
    fn count_cpm_round_trip() {
        let display = 600.0;
        let raw = encode_count_alarm(display, CountDisplayUnit::Cpm);
        assert_eq!(raw.as_u32(), 100);
        assert!((decode_count_alarm(raw, CountDisplayUnit::Cpm) - display).abs() < 0.01);
    }

    #[test]
    fn realtime_dose_conversions() {
        let rh = 0.000_125;
        assert!((dose_display_from_rh(rh, DoseDisplayUnit::MicroSievertPerHour) - 1.25).abs() < 0.001);
        assert!(
            (dose_display_from_rh(rh, DoseDisplayUnit::MicroRoentgenPerHour) - 125.0).abs() < 0.1
        );
    }

    #[test]
    fn count_display() {
        assert!((count_display_from_cps(10.0, CountDisplayUnit::Cps) - 10.0).abs() < 0.001);
        assert!((count_display_from_cps(10.0, CountDisplayUnit::Cpm) - 600.0).abs() < 0.001);
    }
}
