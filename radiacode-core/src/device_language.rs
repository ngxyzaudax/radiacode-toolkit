#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceLanguage {
    Russian,
    English,
}

impl DeviceLanguage {
    pub fn to_device_flag(self) -> u32 {
        match self {
            Self::Russian => 0,
            Self::English => 1,
        }
    }
}
