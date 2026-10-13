mod addresses;
mod extract;
mod kinds;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use addresses::ALL_REGISTERS;
pub use extract::{extract_vsfr_f32, extract_vsfr_u32, resolve_value_kind};
pub use kinds::{SfrValueKind, catalog_registers, static_value_kind};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtSfr {
    DeviceCtrl = 0x0500,
    DeviceLang = 0x0502,
    DeviceOn = 0x0503,
    DeviceTime = 0x0504,
    DispCtrl = 0x0510,
    DispBrt = 0x0511,
    DispContr = 0x0512,
    DispOffTime = 0x0513,
    DispOn = 0x0514,
    DispDir = 0x0515,
    DispBacklightOn = 0x0516,
    SoundCtrl = 0x0520,
    SoundVol = 0x0521,
    SoundOn = 0x0522,
    SoundButton = 0x0523,
    VibroCtrl = 0x0530,
    VibroOn = 0x0531,
    LedsCtrl = 0x0540,
    Led0Brt = 0x0541,
    Led1Brt = 0x0542,
    Led2Brt = 0x0543,
    Led3Brt = 0x0544,
    LedsOn = 0x0545,
    AlarmMode = 0x05E0,
    PlaySignal = 0x05E1,
    MsCtrl = 0x0600,
    MsMode = 0x0601,
    MsSubMode = 0x0602,
    MsRun = 0x0603,
    BleTxPwr = 0x0700,
    DrLev1UrH = 0x8000,
    DrLev2UrH = 0x8001,
    DsLev1HundredUr = 0x8002,
    DsLev2HundredUr = 0x8003,
    DsUnits = 0x8004,
    CpsFilter = 0x8005,
    RawFilter = 0x8006,
    DoseReset = 0x8007,
    CrLev1Cp10s = 0x8008,
    CrLev2Cp10s = 0x8009,
    UseNanoSvH = 0x800C,
    ChnToKeVA0 = 0x8010,
    ChnToKeVA1 = 0x8011,
    ChnToKeVA2 = 0x8012,
    CrUnits = 0x8013,
    DsLev1Ur = 0x8014,
    DsLev2Ur = 0x8015,
    Cps = 0x8020,
    DrUrH = 0x8021,
    DsUr = 0x8022,
    TempDegC = 0x8024,
    AccX = 0x8025,
    AccY = 0x8026,
    AccZ = 0x8027,
    Opt = 0x8028,
    RawTempDegC = 0x8033,
    TempUpDegC = 0x8034,
    TempDnDegC = 0x8035,
    VBiasMv = 0xC000,
    CompLev = 0xC001,
    CalibMode = 0xC002,
    DpotRdac = 0xC004,
    DpotRdacEeprom = 0xC005,
    DpotToler = 0xC006,
    SysMcuId0 = 0xFFFF0000,
    SysMcuId1 = 0xFFFF0001,
    SysMcuId2 = 0xFFFF0002,
    SysDeviceId = 0xFFFF0005,
    SysSignature = 0xFFFF0006,
    SysRxSize = 0xFFFF0007,
    SysTxSize = 0xFFFF0008,
    SysBootVersion = 0xFFFF0009,
    SysTargetVersion = 0xFFFF000A,
    SysStatus = 0xFFFF000B,
    SysMcuVref = 0xFFFF000C,
    SysMcuTemp = 0xFFFF000D,
}

impl VirtSfr {
    pub fn catalog() -> &'static [VirtSfr] {
        kinds::catalog()
    }

    pub fn static_value_kind(self) -> Option<SfrValueKind> {
        static_value_kind(self)
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
        addresses::try_from_address(value)
    }
}
