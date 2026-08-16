use crate::types::AlarmLimits;

const CTRL_BUTTONS: u32 = 1 << 0;
const CTRL_CLICKS: u32 = 1 << 1;
const CTRL_DR_ALARM1: u32 = 1 << 2;
const CTRL_DR_ALARM2: u32 = 1 << 3;
const CTRL_DR_OOS: u32 = 1 << 4;
const CTRL_DOSE_ALARM1: u32 = 1 << 5;
const CTRL_DOSE_ALARM2: u32 = 1 << 6;
const CTRL_DOSE_OOS: u32 = 1 << 7;
const CTRL_CONNECTION: u32 = 1 << 8;
const CTRL_POWER: u32 = 1 << 9;
const CTRL_CR_ALARM1: u32 = 1 << 10;
const CTRL_CR_ALARM2: u32 = 1 << 11;
const CTRL_CR_OOS: u32 = 1 << 12;
const CTRL_KNOWN_MASK: u32 = 0x1FFF;
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
    Sec5,
    Sec10,
    Sec15,
    Sec30,
    Min2,
    Min5,
}

impl BacklightOffTime {
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            1 => Self::Sec10,
            2 => Self::Sec15,
            3 => Self::Sec30,
            4 => Self::Min2,
            5 => Self::Min5,
            _ => Self::Sec5,
        }
    }

    pub fn as_raw(self) -> u32 {
        match self {
            Self::Sec5 => 0,
            Self::Sec10 => 1,
            Self::Sec15 => 2,
            Self::Sec30 => 3,
            Self::Min2 => 4,
            Self::Min5 => 5,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalFlags {
    pub buttons: bool,
    pub clicks: bool,
    pub dose_rate_alarm1: bool,
    pub dose_rate_alarm2: bool,
    pub dose_rate_out_of_scale: bool,
    pub dose_alarm1: bool,
    pub dose_alarm2: bool,
    pub dose_out_of_scale: bool,
    pub connection: bool,
    pub power: bool,
    pub count_rate_alarm1: bool,
    pub count_rate_alarm2: bool,
    pub count_rate_out_of_scale: bool,
    pub reserved: u32,
}

impl SignalFlags {
    pub fn from_raw(raw: u32) -> Self {
        Self {
            buttons: raw & CTRL_BUTTONS != 0,
            clicks: raw & CTRL_CLICKS != 0,
            dose_rate_alarm1: raw & CTRL_DR_ALARM1 != 0,
            dose_rate_alarm2: raw & CTRL_DR_ALARM2 != 0,
            dose_rate_out_of_scale: raw & CTRL_DR_OOS != 0,
            dose_alarm1: raw & CTRL_DOSE_ALARM1 != 0,
            dose_alarm2: raw & CTRL_DOSE_ALARM2 != 0,
            dose_out_of_scale: raw & CTRL_DOSE_OOS != 0,
            connection: raw & CTRL_CONNECTION != 0,
            power: raw & CTRL_POWER != 0,
            count_rate_alarm1: raw & CTRL_CR_ALARM1 != 0,
            count_rate_alarm2: raw & CTRL_CR_ALARM2 != 0,
            count_rate_out_of_scale: raw & CTRL_CR_OOS != 0,
            reserved: raw & !CTRL_KNOWN_MASK,
        }
    }

    pub fn as_raw(self) -> u32 {
        let mut raw = self.reserved;
        if self.buttons {
            raw |= CTRL_BUTTONS;
        }
        if self.clicks {
            raw |= CTRL_CLICKS;
        }
        if self.dose_rate_alarm1 {
            raw |= CTRL_DR_ALARM1;
        }
        if self.dose_rate_alarm2 {
            raw |= CTRL_DR_ALARM2;
        }
        if self.dose_rate_out_of_scale {
            raw |= CTRL_DR_OOS;
        }
        if self.dose_alarm1 {
            raw |= CTRL_DOSE_ALARM1;
        }
        if self.dose_alarm2 {
            raw |= CTRL_DOSE_ALARM2;
        }
        if self.dose_out_of_scale {
            raw |= CTRL_DOSE_OOS;
        }
        if self.connection {
            raw |= CTRL_CONNECTION;
        }
        if self.power {
            raw |= CTRL_POWER;
        }
        if self.count_rate_alarm1 {
            raw |= CTRL_CR_ALARM1;
        }
        if self.count_rate_alarm2 {
            raw |= CTRL_CR_ALARM2;
        }
        if self.count_rate_out_of_scale {
            raw |= CTRL_CR_OOS;
        }
        raw
    }

    pub fn without_clicks(self) -> Self {
        Self {
            clicks: false,
            ..self
        }
    }

    pub fn vibro_events(self) -> Self {
        Self {
            clicks: false,
            connection: false,
            power: false,
            ..self
        }
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
