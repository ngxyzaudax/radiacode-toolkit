use radiacode_protocol::{Command, CountDisplayUnit, VirtSfr, VirtString};

use crate::device_config::{DeviceConfig, DisplayDirection};

use crate::test_support::fixtures::configured_alarm_device;
use crate::test_support::{LEDS_SFR_FILE, open_device};
use crate::types::{AlarmLimits, AlarmLimitsUpdate};

const DEVICE_CTRL_LIGHT: u32 = 1 << 3;

#[tokio::test(start_paused = true)]
async fn apply_device_config_writes_registers_and_light_bit() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
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
    let (mut device, handle) = open_device(configured_alarm_device()).await;
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
    let device = configured_alarm_device()
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
    let (mut device, handle) = open_device(configured_alarm_device()).await;
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
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    device
        .set_alarm_limits(&AlarmLimitsUpdate::default())
        .await
        .expect("noop");
    assert_eq!(handle.device().request_count(Command::WrVirtSfrBatch), 0);
}

#[tokio::test(start_paused = true)]
async fn sync_device_clock_sets_time_and_resets_device_time() {
    let (mut device, handle) = open_device(configured_alarm_device()).await;
    handle
        .device()
        .registers
        .insert(u32::from(VirtSfr::DeviceTime), 99);
    device.sync_device_clock().await.expect("sync");
    assert_eq!(handle.device().request_count(Command::SetTime), 2);
    assert_eq!(handle.device().register(VirtSfr::DeviceTime), Some(0));
}
