use crate::catalog::SfrCatalogEntry;
use crate::command::VirtSfr;

const LEDS_ON_ENTRY_NAME: &str = "VSFR_LEDS_ON";

pub fn sfr_supports_leds_on(text: &str) -> bool {
    text.lines()
        .any(|line| line.contains(LEDS_ON_ENTRY_NAME) || line.contains("Addr=0x00000545"))
}

pub fn sfr_entries_support_leds_on(entries: &[SfrCatalogEntry]) -> bool {
    entries.iter().any(|entry| {
        entry.address == u32::from(VirtSfr::LedsOn) || entry.name == LEDS_ON_ENTRY_NAME
    })
}

#[cfg(test)]
mod tests {
    use super::{sfr_entries_support_leds_on, sfr_supports_leds_on};
    use crate::catalog::parse_sfr_file;

    #[test]
    fn detects_leds_on_entry() {
        let sample = "[VSFR_LEDS_ON]\nAddr=0x00000545\nVirtual=1\nSize=1\n";
        assert!(sfr_supports_leds_on(sample));
        assert!(sfr_entries_support_leds_on(&parse_sfr_file(sample)));
    }

    #[test]
    fn absent_when_not_listed() {
        let sample = "[VSFR_VIBRO_ON]\nAddr=0x00000531\n";
        assert!(!sfr_supports_leds_on(sample));
        assert!(!sfr_entries_support_leds_on(&parse_sfr_file(sample)));
    }

    #[test]
    fn entry_detection_matches_by_address_alone() {
        let sample = "[VSFR_UNNAMED]\nAddr=0x00000545\n";
        assert!(sfr_entries_support_leds_on(&parse_sfr_file(sample)));
    }
}
