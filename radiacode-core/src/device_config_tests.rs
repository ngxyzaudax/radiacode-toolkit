use radiacode_protocol::{Command, CountDisplayUnit, DoseDisplayUnit, VirtSfr, VirtString};

use crate::device_config::{
    AlarmSignalMode, BacklightOffTime, DeviceConfig, DisplayDirection, SignalFlags,
};
use crate::test_support::{LEDS_SFR_FILE, VirtualDevice, open_device};
use crate::types::{AlarmLimits, AlarmLimitsUpdate};

const DEVICE_CTRL_LIGHT: u32 = 1 << 3;

fn configured_device() -> VirtualDevice {
    VirtualDevice::default()
        .with_register(VirtSfr::CrLev1Cp10s, 300)
        .with_register(VirtSfr::CrLev2Cp10s, 600)
        .with_register(VirtSfr::DrLev1UrH, 100)
        .with_register(VirtSfr::DrLev2UrH, 200)
        .with_register(VirtSfr::DsLev1Ur, 1000)
        .with_register(VirtSfr::DsLev2Ur, 2000)
        .with_register(VirtSfr::DsUnits, 1)
        .with_register(VirtSfr::CrUnits, 0)
        .with_register(VirtSfr::AlarmMode, 1)
        .with_register(VirtSfr::DispBrt, 12)
        .with_register(VirtSfr::DispOffTime, 3)
        .with_register(VirtSfr::DispDir, 2)
        .with_register(VirtSfr::SoundOn, 1)
        .with_register(VirtSfr::SoundCtrl, 0x1f9d)
        .with_register(VirtSfr::VibroOn, 0)
        .with_register(VirtSfr::VibroCtrl, 0x1fff)
        .with_register(VirtSfr::DeviceCtrl, 0x35)
}

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
    let (mut device, handle) = open_device(configured_device()).await;
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
    let device = configured_device()
        .with_string(VirtString::SfrFile, LEDS_SFR_FILE.as_bytes())
        .with_register(VirtSfr::LedsOn, 1);
    let (mut device, _handle) = open_device(device).await;
    let config = device.load_device_config().await.expect("config");
    assert!(config.leds_on && config.leds_supported && !config.leds_uses_device_ctrl);
}

#[tokio::test(start_paused = true)]
async fn apply_device_config_writes_registers_and_light_bit() {
    let (mut device, handle) = open_device(configured_device()).await;
    let mut config = device.load_device_config().await.expect("config");
    config.brightness = 3;
    config.leds_on = true;
    config.display_dir = DisplayDirection::Right;
    device.apply_device_config(&config).await.expect("apply");
    let state = handle.device();
    assert_eq!(state.register(VirtSfr::DispBrt), Some(3));
    assert_eq!(state.register(VirtSfr::DispDir), Some(1));
    assert_eq!(
        state.register(VirtSfr::DeviceCtrl),
        Some(0x35 | DEVICE_CTRL_LIGHT)
    );
    assert_eq!(state.register(VirtSfr::LedsOn), None);
}

#[tokio::test(start_paused = true)]
async fn apply_device_config_skips_device_ctrl_write_when_unchanged() {
    let (mut device, handle) = open_device(configured_device()).await;
    let config = device.load_device_config().await.expect("config");
    let writes_before = handle.device().request_count(Command::WrVirtSfr);
    device.apply_device_config(&config).await.expect("apply");
    assert_eq!(
        handle.device().request_count(Command::WrVirtSfr),
        writes_before
    );
}

#[tokio::test(start_paused = true)]
async fn apply_device_config_writes_leds_on_register_when_supported() {
    let device = configured_device()
        .with_string(VirtString::SfrFile, LEDS_SFR_FILE.as_bytes())
        .with_register(VirtSfr::LedsOn, 0);
    let (mut device, handle) = open_device(device).await;
    let config = DeviceConfig {
        leds_on: true,
        ..device.load_device_config().await.expect("config")
    };
    device.apply_device_config(&config).await.expect("apply");
    assert_eq!(handle.device().register(VirtSfr::LedsOn), Some(1));
    assert_eq!(handle.device().register(VirtSfr::DeviceCtrl), Some(0x35));
}

#[tokio::test(start_paused = true)]
async fn alarm_limits_round_trip_through_update() {
    let (mut device, handle) = open_device(configured_device()).await;
    let limits: AlarmLimits = device.alarm_limits().await.expect("limits");
    assert_eq!(limits.l2_count_rate, 60.0);
    device
        .set_alarm_limits(&AlarmLimitsUpdate {
            l1_count_rate: Some(45.0),
            count_unit: Some(CountDisplayUnit::Cps),
            ..AlarmLimitsUpdate::default()
        })
        .await
        .expect("update");
    assert_eq!(handle.device().register(VirtSfr::CrLev1Cp10s), Some(450));
    assert_eq!(handle.device().register(VirtSfr::CrLev2Cp10s), Some(600));
    let reloaded = device.alarm_limits().await.expect("limits");
    assert_eq!(reloaded.l1_count_rate, 45.0);
}

#[tokio::test(start_paused = true)]
async fn empty_alarm_update_writes_nothing() {
    let (mut device, handle) = open_device(configured_device()).await;
    device
        .set_alarm_limits(&AlarmLimitsUpdate::default())
        .await
        .expect("noop");
    assert_eq!(handle.device().request_count(Command::WrVirtSfrBatch), 0);
}

#[tokio::test(start_paused = true)]
async fn sync_device_clock_sets_time_and_resets_device_time() {
    let (mut device, handle) = open_device(configured_device()).await;
    handle
        .device()
        .registers
        .insert(u32::from(VirtSfr::DeviceTime), 99);
    device.sync_device_clock().await.expect("sync");
    assert_eq!(handle.device().request_count(Command::SetTime), 2);
    assert_eq!(handle.device().register(VirtSfr::DeviceTime), Some(0));
}
