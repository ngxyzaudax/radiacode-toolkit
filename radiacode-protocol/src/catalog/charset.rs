pub fn decode_cp1251(bytes: &[u8]) -> String {
    encoding_rs::WINDOWS_1251.decode(bytes).0.into_owned()
}

#[cfg(test)]
mod tests {
    use super::decode_cp1251;

    #[test]
    fn decodes_cyrillic_unit_label() {
        let bytes = [0xEC, 0xEA, 0xC2, 0x2F, 0xF7];
        let text = decode_cp1251(&bytes);
        assert_eq!(text, "мкВ/ч");
    }
}
