use crate::types::AlarmLimits;

use super::signal_flags::SignalFlags;

pub(crate) const DEVICE_CTRL_LIGHT: u32 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmSignalMode {
    Once,
    Continuous,
}

impl AlarmSignalMode {
    pub fn from_raw(raw: u32) -> Self {
        if raw == 0 {
            Self::Continuous
        } else {
            Self::Once
        }
    }

    pub fn as_raw(self) -> u32 {
        match self {
            Self::Continuous => 0,
            Self::Once => 1,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Once => "Once",
            Self::Continuous => "Continuous",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayDirection {
    Auto = 0,
    Right = 1,
    Left = 2,
}

impl DisplayDirection {
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            1 => Self::Right,
            2 => Self::Left,
            _ => Self::Auto,
        }
    }

    pub fn as_raw(self) -> u32 {
        self as u32
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Right => "Right",
            Self::Left => "Left",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacklightOffTime {
    Sec5 = 0,
    Sec10 = 1,
    Sec15 = 2,
    Sec30 = 3,
    Min2 = 4,
    Min5 = 5,
}

impl BacklightOffTime {
    pub fn from_raw(raw: u32) -> Self {
        Self::all()
            .into_iter()
            .find(|value| value.as_raw() == raw)
            .unwrap_or(Self::Sec5)
    }

    pub fn as_raw(self) -> u32 {
        self as u32
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sec5 => "5 s",
            Self::Sec10 => "10 s",
            Self::Sec15 => "15 s",
            Self::Sec30 => "30 s",
            Self::Min2 => "2 min",
            Self::Min5 => "5 min",
        }
    }

    pub fn all() -> [Self; 6] {
        [
            Self::Sec5,
            Self::Sec10,
            Self::Sec15,
            Self::Sec30,
            Self::Min2,
            Self::Min5,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceConfig {
    pub alarms: AlarmLimits,
    pub alarm_mode: AlarmSignalMode,
    pub brightness: u8,
    pub backlight_off: BacklightOffTime,
    pub display_dir: DisplayDirection,
    pub sound_on: bool,
    pub vibro_on: bool,
    pub leds_on: bool,
    pub leds_supported: bool,
    pub leds_uses_device_ctrl: bool,
    pub sound_ctrl: SignalFlags,
    pub vibro_ctrl: SignalFlags,
}
