pub use radiacode_protocol::{
    CountDisplayUnit, DoseDisplayUnit, count_display_from_cps, decode_count_alarm,
    decode_dose_accum, decode_dose_alarm, dose_display_from_rh, encode_count_alarm,
    encode_dose_accum, encode_dose_alarm,
};

pub fn dose_unit_label(unit: DoseDisplayUnit) -> &'static str {
    match unit {
        DoseDisplayUnit::MicroSievertPerHour => "µSv/h",
        DoseDisplayUnit::MicroRoentgenPerHour => "µR/h",
    }
}

pub fn dose_accum_unit_label(unit: DoseDisplayUnit) -> &'static str {
    match unit {
        DoseDisplayUnit::MicroSievertPerHour => "µSv",
        DoseDisplayUnit::MicroRoentgenPerHour => "µR",
    }
}

pub fn count_unit_label(unit: CountDisplayUnit) -> &'static str {
    match unit {
        CountDisplayUnit::Cpm => "cpm",
        CountDisplayUnit::Cps => "cps",
    }
}
