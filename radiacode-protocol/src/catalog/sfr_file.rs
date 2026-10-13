#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfrValueKind {
    U8,
    U16,
    U32,
    I32,
    F32,
    Bool,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfrCatalogEntry {
    pub name: String,
    pub address: u32,
    pub size: u8,
    pub value_kind: SfrValueKind,
    pub signed: bool,
}

impl SfrCatalogEntry {
    fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            address: 0,
            size: 0,
            value_kind: SfrValueKind::Unknown,
            signed: false,
        }
    }
}

pub fn parse_sfr_file(text: &str) -> Vec<SfrCatalogEntry> {
    let mut entries = Vec::new();
    let mut current: Option<SfrCatalogEntry> = None;
    for line in text.lines().map(str::trim) {
        if let Some(section) = section_name(line) {
            entries.extend(current.take());
            current = section
                .contains("VSFR_")
                .then(|| SfrCatalogEntry::named(section));
            continue;
        }
        if let Some(entry) = current.as_mut() {
            apply_field(entry, line);
        }
    }
    entries.extend(current);
    entries
}

fn section_name(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']')
}

fn apply_field(entry: &mut SfrCatalogEntry, line: &str) {
    if let Some(value) = line.strip_prefix("Addr=") {
        entry.address = parse_hex_address(value);
    } else if let Some(value) = line.strip_prefix("Size=") {
        entry.size = value.parse().unwrap_or(0);
    } else if let Some(value) = line.strip_prefix("Type=") {
        entry.value_kind = parse_value_kind(value);
    } else if let Some(value) = line.strip_prefix("Signed=") {
        entry.signed = value == "1";
    }
}

fn parse_value_kind(value: &str) -> SfrValueKind {
    match value {
        "1" => SfrValueKind::U8,
        "2" => SfrValueKind::U16,
        "4" => SfrValueKind::U32,
        "5" => SfrValueKind::F32,
        _ => SfrValueKind::Unknown,
    }
}

fn parse_hex_address(value: &str) -> u32 {
    let trimmed = value
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    u32::from_str_radix(trimmed, 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{SfrValueKind, parse_sfr_file};

    #[test]
    fn parses_sfr_entry() {
        let sample = "[VSFR_DISP_BRT]\nAddr=0x00000511\nSize=1\nType=1\nSigned=0\n";
        let entries = parse_sfr_file(sample);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "VSFR_DISP_BRT");
        assert_eq!(entries[0].address, 0x511);
        assert_eq!(entries[0].size, 1);
        assert_eq!(entries[0].value_kind, SfrValueKind::U8);
        assert!(!entries[0].signed);
    }

    #[test]
    fn non_vsfr_section_terminates_previous_entry() {
        let sample = "[VSFR_A]\nAddr=0x10\n[GENERAL]\nAddr=0x20\n[VSFR_B]\nAddr=0x30\n";
        let entries = parse_sfr_file(sample);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].address, 0x10);
        assert_eq!(entries[1].address, 0x30);
    }
}
