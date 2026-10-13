use radiacode_protocol::{Command, CountDisplayUnit, DoseDisplayUnit, VirtSfr, VirtString};

use crate::device_config::{AlarmSignalMode, BacklightOffTime, DisplayDirection, SignalFlags};
use crate::test_support::fixtures::configured_alarm_device;
use crate::test_support::{LEDS_SFR_FILE, open_device};

#[test]
fn backlight_round_trip() {
    for value in BacklightOffTime::all() {
        assert_eq!(BacklightOffTime::from_raw(value.as_raw()), value);
        assert!(!value.label().is_empty());
    }
    assert_eq!(BacklightOffTime::from_raw(99), BacklightOffTime::Sec5);
}

#[test]
fn display_direction_and_alarm_mode_round_trip() {
    for direction in [
        DisplayDirection::Auto,
        DisplayDirection::Right,
        DisplayDirection::Left,
    ] {
        assert_eq!(DisplayDirection::from_raw(direction.as_raw()), direction);
        assert!(!direction.label().is_empty());
    }
    assert_eq!(DisplayDirection::from_raw(7), DisplayDirection::Auto);
    assert_eq!(AlarmSignalMode::from_raw(0), AlarmSignalMode::Continuous);
    assert_eq!(AlarmSignalMode::from_raw(5), AlarmSignalMode::Once);
    assert_eq!(AlarmSignalMode::Once.as_raw(), 1);
    assert_eq!(AlarmSignalMode::Continuous.label(), "Continuous");
}

#[test]
fn signal_flags_round_trip_sound_ctrl() {
    let flags = SignalFlags::from_raw(0x1f9d);
    assert!(flags.buttons);
    assert!(!flags.clicks);
    assert!(flags.dose_rate_alarm1);
    assert!(flags.connection);
    assert!(flags.power);
    assert!(flags.count_rate_alarm1);
    assert!(flags.count_rate_alarm2);
    assert!(flags.count_rate_out_of_scale);
    assert_eq!(flags.as_raw(), 0x1f9d);
}

#[test]
fn signal_flags_preserve_reserved_bits_and_strip_vibro_events() {
    let flags = SignalFlags::from_raw(0xFFFF_FFFF);
    assert_eq!(flags.reserved, 0xFFFF_E000);
    assert_eq!(flags.as_raw(), 0xFFFF_FFFF);
    let vibro = flags.vibro_events();
    assert!(!vibro.clicks && !vibro.connection && !vibro.power);
    assert_eq!(vibro.as_raw(), !((1 << 1) | (1 << 8) | (1 << 9)));
}

#[tokio::test(start_paused = true)]
async fn load_device_config_decodes_all_registers_via_device_ctrl() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    let config = device.load_device_config().await.expect("config");
    assert_eq!(
        config.alarms.dose_unit,
        DoseDisplayUnit::MicroSievertPerHour
    );
    assert_eq!(config.alarms.count_unit, CountDisplayUnit::Cps);
    assert_eq!(config.alarms.l1_count_rate, 30.0);
    assert_eq!(config.alarm_mode, AlarmSignalMode::Once);
    assert_eq!(config.brightness, 9);
    assert_eq!(config.backlight_off, BacklightOffTime::Sec30);
    assert_eq!(config.display_dir, DisplayDirection::Left);
    assert!(config.sound_on && !config.vibro_on);
    assert!(!config.vibro_ctrl.clicks);
    assert!(config.leds_supported && config.leds_uses_device_ctrl && !config.leds_on);
    let sfr_reads = handle
        .device()
        .requests_for(Command::RdVirtString)
        .into_iter()
        .filter(|request| request.args == u32::from(VirtString::SfrFile).to_le_bytes())
        .count();
    assert_eq!(sfr_reads, 1);
}

#[tokio::test(start_paused = true)]
async fn load_device_config_uses_leds_on_register_when_catalog_lists_it() {
    let device = configured_alarm_device()
        .with_string(VirtString::SfrFile, LEDS_SFR_FILE.as_bytes())
        .with_register(VirtSfr::LedsOn, 1);
    let (mut device, _handle) = open_device(device).await;
    let config = device.load_device_config().await.expect("config");
    assert!(config.leds_on && config.leds_supported && !config.leds_uses_device_ctrl);
}
