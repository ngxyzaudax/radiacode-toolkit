mod opcodes;
mod virt_sfr;

pub use opcodes::{Command, VirtString};
pub use virt_sfr::{
    SfrValueKind, VirtSfr, catalog_registers, extract_vsfr_f32, extract_vsfr_u32,
    resolve_value_kind,
};
