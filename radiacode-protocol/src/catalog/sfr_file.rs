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

pub fn parse_sfr_file(text: &str) -> Vec<SfrCatalogEntry> {
    let mut entries = Vec::new();
    let mut current: Option<SfrCatalogEntry> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') && line.contains("VSFR_") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(SfrCatalogEntry {
                name: line.trim_start_matches('[').trim_end_matches(']').to_string(),
                address: 0,
                size: 0,
                value_kind: SfrValueKind::Unknown,
                signed: false,
            });
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        if let Some(value) = line.strip_prefix("Addr=") {
            entry.address = parse_hex_address(value);
        } else if let Some(value) = line.strip_prefix("Size=") {
            entry.size = value.parse().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("Type=") {
            entry.value_kind = match value {
                "1" => SfrValueKind::U8,
                "2" => SfrValueKind::U16,
                "4" => SfrValueKind::U32,
                "5" => SfrValueKind::F32,
                _ => SfrValueKind::Unknown,
            };
        } else if let Some(value) = line.strip_prefix("Signed=") {
            entry.signed = value == "1";
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    entries
}

fn parse_hex_address(value: &str) -> u32 {
    let trimmed = value.trim().trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(trimmed, 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{parse_sfr_file, SfrValueKind};

    #[test]
    fn parses_sfr_entry() {
        let text = "[VSFR_LEDS_ON]\nAddr=0x00000545\nSize=1\nType=1\nSigned=0\n";
        let entries = parse_sfr_file(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].address, 0x545);
        assert_eq!(entries[0].value_kind, SfrValueKind::U8);
    }
}
