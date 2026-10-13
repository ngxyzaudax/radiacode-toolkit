use super::{SfrValueKind, VirtSfr};
use crate::catalog::{SfrCatalogEntry, SfrValueKind as DeviceSfrKind};

pub fn extract_vsfr_u32(raw: u32, kind: SfrValueKind) -> u32 {
    match kind {
        SfrValueKind::U8 | SfrValueKind::Bool => pick_subword(raw, 8),
        SfrValueKind::U16 => pick_subword(raw, 16),
        SfrValueKind::I16 => pick_subword(raw, 16) as i16 as u32,
        SfrValueKind::U32 | SfrValueKind::I32 | SfrValueKind::F32 => raw,
    }
}

fn pick_subword(raw: u32, width: u32) -> u32 {
    let mask = (1u32 << width) - 1;
    let low = raw & mask;
    let high = raw >> (32 - width);
    if high != 0 && low == 0 { high } else { low }
}

pub fn extract_vsfr_f32(raw: u32, kind: SfrValueKind) -> f32 {
    match kind {
        SfrValueKind::F32 => f32::from_bits(raw),
        other => f32::from_bits(extract_vsfr_u32(raw, other)),
    }
}

pub fn resolve_value_kind(register: VirtSfr, entries: &[SfrCatalogEntry]) -> Option<SfrValueKind> {
    entries
        .iter()
        .find(|entry| entry.address == u32::from(register))
        .map(device_entry_kind)
        .or_else(|| register.static_value_kind())
}

fn device_entry_kind(entry: &SfrCatalogEntry) -> SfrValueKind {
    match (entry.size, entry.value_kind) {
        (1, DeviceSfrKind::U8) => SfrValueKind::U8,
        (2, DeviceSfrKind::U16) if entry.signed => SfrValueKind::I16,
        (2, DeviceSfrKind::U16) => SfrValueKind::U16,
        (4, DeviceSfrKind::F32) => SfrValueKind::F32,
        (4, DeviceSfrKind::U32) if entry.signed => SfrValueKind::I32,
        (4, DeviceSfrKind::U32) | (4, DeviceSfrKind::I32) => SfrValueKind::U32,
        (1, _) => SfrValueKind::Bool,
        _ => SfrValueKind::U32,
    }
}

#[cfg(test)]
mod tests {
    use super::{SfrValueKind, extract_vsfr_u32};

    #[test]
    fn extracts_u8_from_high_byte() {
        assert_eq!(extract_vsfr_u32(0x0500_0000, SfrValueKind::U8), 5);
    }

    #[test]
    fn extracts_u8_from_low_byte() {
        assert_eq!(extract_vsfr_u32(1, SfrValueKind::U8), 1);
    }

    #[test]
    fn extracts_u16_from_high_halfword() {
        assert_eq!(extract_vsfr_u32(0x0001_0000, SfrValueKind::U16), 1);
    }

    #[test]
    fn extracts_u16_from_low_halfword() {
        assert_eq!(extract_vsfr_u32(1, SfrValueKind::U16), 1);
    }

    #[test]
    fn passes_through_u32() {
        assert_eq!(
            extract_vsfr_u32(0x1234_5678, SfrValueKind::U32),
            0x1234_5678
        );
    }
}
