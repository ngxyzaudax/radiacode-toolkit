use super::{ALL_REGISTERS, SfrValueKind, VirtSfr, catalog_registers, static_value_kind};
use crate::catalog::parse_sfr_file;
use crate::command::virt_sfr::extract::{extract_vsfr_f32, extract_vsfr_u32, resolve_value_kind};

#[test]
fn every_register_round_trips_through_its_address() {
    for register in ALL_REGISTERS {
        let address = u32::from(register);
        assert_eq!(VirtSfr::try_from(address), Ok(register));
    }
}

#[test]
fn addresses_are_unique() {
    let mut addresses: Vec<u32> = ALL_REGISTERS.iter().map(|r| u32::from(*r)).collect();
    addresses.sort_unstable();
    addresses.dedup();
    assert_eq!(addresses.len(), ALL_REGISTERS.len());
}

#[test]
fn unknown_address_is_rejected() {
    assert_eq!(VirtSfr::try_from(0x1234_5678), Err(()));
}

#[test]
fn every_register_has_a_static_kind() {
    for register in ALL_REGISTERS {
        assert!(
            static_value_kind(register).is_some(),
            "{register:?} lacks static kind"
        );
        assert_eq!(register.static_value_kind(), static_value_kind(register));
    }
}

#[test]
fn catalog_registers_are_a_subset_of_all_registers() {
    for register in catalog_registers() {
        assert!(ALL_REGISTERS.contains(&register));
    }
    assert_eq!(VirtSfr::catalog().len(), catalog_registers().len());
}

#[test]
fn static_kinds_match_known_registers() {
    assert_eq!(static_value_kind(VirtSfr::DispBrt), Some(SfrValueKind::U8));
    assert_eq!(
        static_value_kind(VirtSfr::DeviceOn),
        Some(SfrValueKind::Bool)
    );
    assert_eq!(static_value_kind(VirtSfr::VBiasMv), Some(SfrValueKind::U16));
    assert_eq!(static_value_kind(VirtSfr::AccX), Some(SfrValueKind::I16));
    assert_eq!(
        static_value_kind(VirtSfr::DeviceTime),
        Some(SfrValueKind::U32)
    );
    assert_eq!(
        static_value_kind(VirtSfr::TempDegC),
        Some(SfrValueKind::F32)
    );
    assert_eq!(
        static_value_kind(VirtSfr::SysMcuVref),
        Some(SfrValueKind::I32)
    );
    assert_eq!(
        static_value_kind(VirtSfr::DeviceCtrl),
        Some(SfrValueKind::U32)
    );
}

#[test]
fn extract_sub_word_prefers_low_bits_unless_only_high_bits_are_set() {
    assert_eq!(extract_vsfr_u32(0x0000_0007, SfrValueKind::U8), 7);
    assert_eq!(extract_vsfr_u32(0x0700_0000, SfrValueKind::U8), 7);
    assert_eq!(extract_vsfr_u32(0x0700_0003, SfrValueKind::U8), 3);
    assert_eq!(extract_vsfr_u32(0x1234_0000, SfrValueKind::U16), 0x1234);
    assert_eq!(extract_vsfr_u32(0x0000_FFFF, SfrValueKind::I16), u32::MAX);
    assert_eq!(
        extract_vsfr_u32(0xDEAD_BEEF, SfrValueKind::U32),
        0xDEAD_BEEF
    );
    assert_eq!(extract_vsfr_u32(0x0000_0001, SfrValueKind::Bool), 1);
}

#[test]
fn extract_f32_reinterprets_bits() {
    let raw = 2.5f32.to_bits();
    assert_eq!(extract_vsfr_f32(raw, SfrValueKind::F32), 2.5);
    assert_eq!(extract_vsfr_f32(raw, SfrValueKind::U32), 2.5);
}

#[test]
fn resolve_value_kind_prefers_device_catalog_entries() {
    let sfr = "[VSFR_DISP_BRT]\nAddr=0x00000511\nSize=2\nType=2\nSigned=1\n\
               [VSFR_X]\nAddr=0x00008024\nSize=4\nType=5\n\
               [VSFR_Y]\nAddr=0x00008021\nSize=4\nType=4\nSigned=1\n\
               [VSFR_Z]\nAddr=0x00000522\nSize=1\nType=9\n\
               [VSFR_W]\nAddr=0x00000531\nSize=3\nType=4\n";
    let entries = parse_sfr_file(sfr);
    assert_eq!(
        resolve_value_kind(VirtSfr::DispBrt, &entries),
        Some(SfrValueKind::I16)
    );
    assert_eq!(
        resolve_value_kind(VirtSfr::TempDegC, &entries),
        Some(SfrValueKind::F32)
    );
    assert_eq!(
        resolve_value_kind(VirtSfr::DrUrH, &entries),
        Some(SfrValueKind::I32)
    );
    assert_eq!(
        resolve_value_kind(VirtSfr::SoundOn, &entries),
        Some(SfrValueKind::Bool)
    );
    assert_eq!(
        resolve_value_kind(VirtSfr::VibroOn, &entries),
        Some(SfrValueKind::U32)
    );
    assert_eq!(
        resolve_value_kind(VirtSfr::DeviceOn, &entries),
        Some(SfrValueKind::Bool)
    );
}
