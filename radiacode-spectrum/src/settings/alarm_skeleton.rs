use radiacode_core::{
    AlarmLimits, AlarmSignalMode, BacklightOffTime, CountDisplayUnit, DeviceConfig,
    DisplayDirection, DoseDisplayUnit, SignalFlags,
};

pub fn alarm_skeleton_config() -> DeviceConfig {
    DeviceConfig {
        alarms: AlarmLimits {
            l1_count_rate: 0.0,
            l2_count_rate: 0.0,
            l1_dose_rate: 0.0,
            l2_dose_rate: 0.0,
            l1_dose: 0.0,
            l2_dose: 0.0,
            dose_unit: DoseDisplayUnit::MicroSievertPerHour,
            count_unit: CountDisplayUnit::Cps,
        },
        alarm_mode: AlarmSignalMode::Once,
        brightness: 5,
        backlight_off: BacklightOffTime::Sec30,
        display_dir: DisplayDirection::Auto,
        sound_on: true,
        vibro_on: true,
        leds_on: false,
        leds_supported: true,
        leds_uses_device_ctrl: false,
        sound_ctrl: SignalFlags::from_raw(0),
        vibro_ctrl: SignalFlags::from_raw(0),
    }
}
