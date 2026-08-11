#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusFlags(u16);

impl StatusFlags {
    pub fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u16 {
        self.0
    }

    pub fn charging(self) -> bool {
        self.0 & 0x2000 != 0
    }

    pub fn charge_complete(self) -> bool {
        self.0 & 0x4000 != 0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventId {
    PowerOff = 0,
    PowerOn = 1,
    LowBatteryShutdown = 2,
    ChangeDeviceParams = 3,
    DoseReset = 4,
    UserEvent = 5,
    BatteryEmptyAlarm = 6,
    ChargeStart = 7,
    ChargeStop = 8,
    DoseRateAlarm1 = 9,
    DoseRateAlarm2 = 10,
    DoseRateOffscale = 11,
    DoseAlarm1 = 12,
    DoseAlarm2 = 13,
    DoseOffscale = 14,
    TemperatureTooLow = 15,
    TemperatureTooHigh = 16,
    TextMessage = 17,
    MemorySnapshot = 18,
    SpectrumReset = 19,
    CountRateAlarm1 = 20,
    CountRateAlarm2 = 21,
    CountRateOffscale = 22,
    Unknown(u8),
}

impl EventId {
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::PowerOff,
            1 => Self::PowerOn,
            2 => Self::LowBatteryShutdown,
            3 => Self::ChangeDeviceParams,
            4 => Self::DoseReset,
            5 => Self::UserEvent,
            6 => Self::BatteryEmptyAlarm,
            7 => Self::ChargeStart,
            8 => Self::ChargeStop,
            9 => Self::DoseRateAlarm1,
            10 => Self::DoseRateAlarm2,
            11 => Self::DoseRateOffscale,
            12 => Self::DoseAlarm1,
            13 => Self::DoseAlarm2,
            14 => Self::DoseOffscale,
            15 => Self::TemperatureTooLow,
            16 => Self::TemperatureTooHigh,
            17 => Self::TextMessage,
            18 => Self::MemorySnapshot,
            19 => Self::SpectrumReset,
            20 => Self::CountRateAlarm1,
            21 => Self::CountRateAlarm2,
            22 => Self::CountRateOffscale,
            other => Self::Unknown(other),
        }
    }
}
