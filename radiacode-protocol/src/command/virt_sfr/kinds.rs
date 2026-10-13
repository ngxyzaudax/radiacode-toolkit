use super::VirtSfr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfrValueKind {
    U8,
    U16,
    I16,
    U32,
    I32,
    F32,
    Bool,
}

pub fn catalog() -> &'static [VirtSfr] {
    &[
        VirtSfr::DeviceCtrl,
        VirtSfr::DeviceTime,
        VirtSfr::DispBrt,
        VirtSfr::DispOffTime,
        VirtSfr::DispDir,
        VirtSfr::SoundCtrl,
        VirtSfr::SoundOn,
        VirtSfr::VibroCtrl,
        VirtSfr::VibroOn,
        VirtSfr::LedsOn,
        VirtSfr::AlarmMode,
        VirtSfr::DrLev1UrH,
        VirtSfr::DrLev2UrH,
        VirtSfr::DsUnits,
        VirtSfr::CpsFilter,
        VirtSfr::RawFilter,
        VirtSfr::DoseReset,
        VirtSfr::CrLev1Cp10s,
        VirtSfr::CrLev2Cp10s,
        VirtSfr::CrUnits,
        VirtSfr::DsLev1Ur,
        VirtSfr::DsLev2Ur,
        VirtSfr::Cps,
        VirtSfr::DrUrH,
        VirtSfr::DsUr,
        VirtSfr::TempDegC,
        VirtSfr::VBiasMv,
        VirtSfr::SysStatus,
        VirtSfr::SysMcuTemp,
    ]
}

pub fn catalog_registers() -> Vec<VirtSfr> {
    catalog().to_vec()
}

pub fn static_value_kind(register: VirtSfr) -> Option<SfrValueKind> {
    match register {
        VirtSfr::DispBrt
        | VirtSfr::SoundVol
        | VirtSfr::SoundOn
        | VirtSfr::VibroOn
        | VirtSfr::Led0Brt
        | VirtSfr::Led1Brt
        | VirtSfr::Led2Brt
        | VirtSfr::Led3Brt
        | VirtSfr::LedsOn
        | VirtSfr::CpsFilter
        | VirtSfr::RawFilter
        | VirtSfr::DsUnits
        | VirtSfr::CrUnits
        | VirtSfr::UseNanoSvH
        | VirtSfr::DispCtrl
        | VirtSfr::DispContr
        | VirtSfr::DispDir
        | VirtSfr::BleTxPwr
        | VirtSfr::PlaySignal
        | VirtSfr::MsCtrl
        | VirtSfr::MsMode
        | VirtSfr::MsSubMode
        | VirtSfr::DpotRdac
        | VirtSfr::DpotRdacEeprom
        | VirtSfr::DpotToler => Some(SfrValueKind::U8),
        VirtSfr::DispOn
        | VirtSfr::DispBacklightOn
        | VirtSfr::DeviceLang
        | VirtSfr::DeviceOn
        | VirtSfr::MsRun
        | VirtSfr::DoseReset
        | VirtSfr::CalibMode => Some(SfrValueKind::Bool),
        VirtSfr::SysRxSize | VirtSfr::SysTxSize | VirtSfr::Opt | VirtSfr::VBiasMv => {
            Some(SfrValueKind::U16)
        }
        VirtSfr::AccX | VirtSfr::AccY | VirtSfr::AccZ | VirtSfr::CompLev => Some(SfrValueKind::I16),
        VirtSfr::DispOffTime
        | VirtSfr::DrLev1UrH
        | VirtSfr::DrLev2UrH
        | VirtSfr::DsLev1HundredUr
        | VirtSfr::DsLev2HundredUr
        | VirtSfr::CrLev1Cp10s
        | VirtSfr::CrLev2Cp10s
        | VirtSfr::DsLev1Ur
        | VirtSfr::DsLev2Ur
        | VirtSfr::DeviceTime
        | VirtSfr::Cps
        | VirtSfr::DrUrH
        | VirtSfr::DsUr
        | VirtSfr::SysStatus
        | VirtSfr::SysMcuId0
        | VirtSfr::SysMcuId1
        | VirtSfr::SysMcuId2
        | VirtSfr::SysDeviceId
        | VirtSfr::SysSignature
        | VirtSfr::SysBootVersion
        | VirtSfr::SysTargetVersion => Some(SfrValueKind::U32),
        VirtSfr::ChnToKeVA0
        | VirtSfr::ChnToKeVA1
        | VirtSfr::ChnToKeVA2
        | VirtSfr::TempDegC
        | VirtSfr::RawTempDegC
        | VirtSfr::TempUpDegC
        | VirtSfr::TempDnDegC
        | VirtSfr::SysMcuTemp => Some(SfrValueKind::F32),
        VirtSfr::SysMcuVref => Some(SfrValueKind::I32),
        VirtSfr::DeviceCtrl
        | VirtSfr::SoundCtrl
        | VirtSfr::VibroCtrl
        | VirtSfr::AlarmMode
        | VirtSfr::LedsCtrl
        | VirtSfr::SoundButton => Some(SfrValueKind::U32),
    }
}
