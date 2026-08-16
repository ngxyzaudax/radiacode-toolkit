#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfrValueKind {
    U8,
    U32,
    F32,
    Bool,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtSfr {
    DeviceCtrl = 0x0500,
    DeviceLang = 0x0502,
    DeviceOn = 0x0503,
    DeviceTime = 0x0504,
    DispBrt = 0x0511,
    DispOffTime = 0x0513,
    DispDir = 0x0515,
    SoundCtrl = 0x0520,
    SoundOn = 0x0522,
    VibroCtrl = 0x0530,
    VibroOn = 0x0531,
    LedsCtrl = 0x0540,
    LedsOn = 0x0545,
    AlarmMode = 0x05E0,
    DrLev1UrH = 0x8000,
    DrLev2UrH = 0x8001,
    DsUnits = 0x8004,
    CpsFilter = 0x8005,
    RawFilter = 0x8006,
    DoseReset = 0x8007,
    CrLev1Cp10s = 0x8008,
    CrLev2Cp10s = 0x8009,
    UseNanoSvH = 0x800C,
    CrUnits = 0x8013,
    DsLev1Ur = 0x8014,
    DsLev2Ur = 0x8015,
    Cps = 0x8020,
    DrUrH = 0x8021,
    DsUr = 0x8022,
    TempDegC = 0x8024,
    VBiasMv = 0xC000,
    SysStatus = 0xFFFF000B,
    SysMcuTemp = 0xFFFF000D,
}

impl VirtSfr {
    pub fn catalog() -> &'static [VirtSfr] {
        &[
            Self::DeviceCtrl,
            Self::DeviceTime,
            Self::DispBrt,
            Self::DispOffTime,
            Self::DispDir,
            Self::SoundCtrl,
            Self::SoundOn,
            Self::VibroCtrl,
            Self::VibroOn,
            Self::LedsOn,
            Self::AlarmMode,
            Self::DrLev1UrH,
            Self::DrLev2UrH,
            Self::DsUnits,
            Self::CpsFilter,
            Self::RawFilter,
            Self::DoseReset,
            Self::CrLev1Cp10s,
            Self::CrLev2Cp10s,
            Self::CrUnits,
            Self::DsLev1Ur,
            Self::DsLev2Ur,
            Self::Cps,
            Self::DrUrH,
            Self::DsUr,
            Self::TempDegC,
            Self::VBiasMv,
            Self::SysStatus,
            Self::SysMcuTemp,
        ]
    }

    pub fn static_value_kind(self) -> Option<SfrValueKind> {
        match self {
            Self::DispBrt
            | Self::SoundOn
            | Self::VibroOn
            | Self::LedsOn
            | Self::CpsFilter
            | Self::RawFilter
            | Self::DsUnits
            | Self::CrUnits
            | Self::UseNanoSvH => Some(SfrValueKind::U8),
            Self::DispOffTime
            | Self::DrLev1UrH
            | Self::DrLev2UrH
            | Self::CrLev1Cp10s
            | Self::CrLev2Cp10s
            | Self::DsLev1Ur
            | Self::DsLev2Ur
            | Self::DeviceTime
            | Self::Cps
            | Self::DrUrH
            | Self::DsUr
            | Self::SysStatus => Some(SfrValueKind::U32),
            Self::TempDegC | Self::SysMcuTemp => Some(SfrValueKind::F32),
            Self::DeviceCtrl
            | Self::DispDir
            | Self::SoundCtrl
            | Self::VibroCtrl
            | Self::AlarmMode
            | Self::VBiasMv => Some(SfrValueKind::U32),
            _ => None,
        }
    }
}

impl From<VirtSfr> for u32 {
    fn from(value: VirtSfr) -> Self {
        value as u32
    }
}

impl TryFrom<u32> for VirtSfr {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x0500 => Ok(Self::DeviceCtrl),
            0x0502 => Ok(Self::DeviceLang),
            0x0503 => Ok(Self::DeviceOn),
            0x0504 => Ok(Self::DeviceTime),
            0x0511 => Ok(Self::DispBrt),
            0x0513 => Ok(Self::DispOffTime),
            0x0515 => Ok(Self::DispDir),
            0x0520 => Ok(Self::SoundCtrl),
            0x0522 => Ok(Self::SoundOn),
            0x0530 => Ok(Self::VibroCtrl),
            0x0531 => Ok(Self::VibroOn),
            0x0540 => Ok(Self::LedsCtrl),
            0x0545 => Ok(Self::LedsOn),
            0x05E0 => Ok(Self::AlarmMode),
            0x8000 => Ok(Self::DrLev1UrH),
            0x8001 => Ok(Self::DrLev2UrH),
            0x8004 => Ok(Self::DsUnits),
            0x8005 => Ok(Self::CpsFilter),
            0x8006 => Ok(Self::RawFilter),
            0x8007 => Ok(Self::DoseReset),
            0x8008 => Ok(Self::CrLev1Cp10s),
            0x8009 => Ok(Self::CrLev2Cp10s),
            0x800C => Ok(Self::UseNanoSvH),
            0x8013 => Ok(Self::CrUnits),
            0x8014 => Ok(Self::DsLev1Ur),
            0x8015 => Ok(Self::DsLev2Ur),
            0x8020 => Ok(Self::Cps),
            0x8021 => Ok(Self::DrUrH),
            0x8022 => Ok(Self::DsUr),
            0x8024 => Ok(Self::TempDegC),
            0xC000 => Ok(Self::VBiasMv),
            0xFFFF000B => Ok(Self::SysStatus),
            0xFFFF000D => Ok(Self::SysMcuTemp),
            _ => Err(()),
        }
    }
}
