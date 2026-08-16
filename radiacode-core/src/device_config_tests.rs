use crate::device_config::{BacklightOffTime, SignalFlags};

#[test]
fn backlight_round_trip() {
    for value in BacklightOffTime::all() {
        assert_eq!(BacklightOffTime::from_raw(value.as_raw()), value);
    }
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
fn device_ctrl_light_bit_matches_android_toggle() {
    const DEVICE_CTRL_LIGHT: u32 = 1 << 3;
    assert_eq!(0x3D & DEVICE_CTRL_LIGHT, DEVICE_CTRL_LIGHT);
    assert_eq!(0x35 & DEVICE_CTRL_LIGHT, 0);
    assert_eq!(0x3D ^ 0x35, DEVICE_CTRL_LIGHT);
}
