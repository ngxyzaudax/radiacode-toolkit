use radiacode_protocol::{
    SfrCatalogEntry, VirtSfr, VirtSfrValueKind, extract_vsfr_u32, resolve_value_kind,
};

pub fn decode_batch_words(
    ids: &[VirtSfr],
    raw_words: &[u32],
    entries: &[SfrCatalogEntry],
) -> Vec<u32> {
    ids.iter()
        .zip(raw_words.iter())
        .map(|(id, raw)| {
            let kind = resolve_value_kind(*id, entries).unwrap_or(VirtSfrValueKind::U32);
            extract_vsfr_u32(*raw, kind)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use radiacode_protocol::VirtSfr;

    use super::decode_batch_words;

    #[test]
    fn decodes_u8_register_from_high_byte() {
        let ids = [VirtSfr::DispBrt];
        let raw = [0x0700_0000];
        let values = decode_batch_words(&ids, &raw, &[]);
        assert_eq!(values, vec![7]);
    }
}
