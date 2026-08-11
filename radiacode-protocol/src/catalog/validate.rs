use crate::command::{SfrValueKind as StaticValueKind, VirtSfr};

use super::sfr_file::{SfrValueKind, parse_sfr_file};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogDrift {
    pub register: VirtSfr,
    pub message: String,
}

pub fn validate_catalog(sfr_text: &str) -> Vec<CatalogDrift> {
    let entries = parse_sfr_file(sfr_text);
    let mut drifts = Vec::new();
    for register in catalog_registers() {
        let Some(entry) = entries
            .iter()
            .find(|entry| entry.address == u32::from(register))
        else {
            drifts.push(CatalogDrift {
                register,
                message: "missing from device SFR_FILE".into(),
            });
            continue;
        };
        if let Some(expected_kind) = register.static_value_kind() {
            let device_kind = entry.value_kind;
            if !kinds_compatible(expected_kind, device_kind) {
                drifts.push(CatalogDrift {
                    register,
                    message: format!("device type {device_kind:?} != static {expected_kind:?}"),
                });
            }
        }
    }
    drifts
}

fn catalog_registers() -> Vec<VirtSfr> {
    VirtSfr::catalog().to_vec()
}

fn kinds_compatible(static_kind: StaticValueKind, device_kind: SfrValueKind) -> bool {
    matches!(
        (static_kind, device_kind),
        (StaticValueKind::U32, SfrValueKind::U32)
            | (StaticValueKind::F32, SfrValueKind::F32)
            | (StaticValueKind::U8, SfrValueKind::U8)
            | (StaticValueKind::Bool, SfrValueKind::U8)
            | (_, SfrValueKind::Unknown)
    )
}
