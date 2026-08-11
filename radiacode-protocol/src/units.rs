#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoseDisplayUnit {
    MicroSievertPerHour,
    MicroRoentgenPerHour,
}

impl DoseDisplayUnit {
    pub fn from_device_flag(flag: u32) -> Self {
        if flag != 0 {
            Self::MicroSievertPerHour
        } else {
            Self::MicroRoentgenPerHour
        }
    }

    pub fn to_device_flag(self) -> u32 {
        match self {
            Self::MicroSievertPerHour => 1,
            Self::MicroRoentgenPerHour => 0,
        }
    }

    pub fn is_sv(self) -> bool {
        matches!(self, Self::MicroSievertPerHour)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountDisplayUnit {
    Cps,
    Cpm,
}

impl CountDisplayUnit {
    pub fn from_device_flag(flag: u32) -> Self {
        if flag != 0 {
            Self::Cpm
        } else {
            Self::Cps
        }
    }

    pub fn to_device_flag(self) -> u32 {
        match self {
            Self::Cpm => 1,
            Self::Cps => 0,
        }
    }

    pub fn is_cpm(self) -> bool {
        matches!(self, Self::Cpm)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoseRateRh(f32);

impl DoseRateRh {
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn as_f32(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CountRateCps(f32);

impl CountRateCps {
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn as_f32(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoseRoentgen(f32);

impl DoseRoentgen {
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn as_f32(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawMicroRoentgenPerHour(u32);

impl RawMicroRoentgenPerHour {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawCountsPer10s(u32);

impl RawCountsPer10s {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}
