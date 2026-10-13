pub fn ascii_field(value: &str) -> Vec<u8> {
    let mut bytes = vec![value.len() as u8];
    bytes.extend_from_slice(value.as_bytes());
    bytes
}

pub fn u32_words(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

pub fn fw_version_payload(boot: (u16, u16), target: (u16, u16)) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&boot.1.to_le_bytes());
    bytes.extend_from_slice(&boot.0.to_le_bytes());
    bytes.extend_from_slice(&ascii_field("Jan 01 2020"));
    bytes.extend_from_slice(&target.1.to_le_bytes());
    bytes.extend_from_slice(&target.0.to_le_bytes());
    bytes.extend_from_slice(&ascii_field("Feb 02 2024"));
    bytes
}

pub fn hw_serial_payload(groups: &[u32]) -> Vec<u8> {
    let mut bytes = ((groups.len() * 4) as u32).to_le_bytes().to_vec();
    bytes.extend_from_slice(&u32_words(groups));
    bytes
}

pub fn fw_signature_payload(signature: u32, filename: &str, id_string: &str) -> Vec<u8> {
    let mut bytes = signature.to_le_bytes().to_vec();
    bytes.extend_from_slice(&ascii_field(filename));
    bytes.extend_from_slice(&ascii_field(id_string));
    bytes
}

pub fn virt_string_payload(retcode: u32, data: &[u8]) -> Vec<u8> {
    let mut bytes = retcode.to_le_bytes().to_vec();
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    bytes
}

pub fn vsfr_read_payload(value: Option<u32>) -> Vec<u8> {
    match value {
        Some(value) => u32_words(&[1, value]),
        None => u32_words(&[0]),
    }
}

pub fn spectrum_v0_payload(duration_secs: u32, calib: [f32; 3], counts: &[u32]) -> Vec<u8> {
    let mut bytes = duration_secs.to_le_bytes().to_vec();
    bytes.extend(calib.iter().flat_map(|value| value.to_le_bytes()));
    bytes.extend_from_slice(&u32_words(counts));
    bytes
}

pub fn data_buf_header(seq: u8, entity: u8, group: u8, ticks: i32) -> Vec<u8> {
    let mut bytes = vec![seq, entity, group];
    bytes.extend_from_slice(&ticks.to_le_bytes());
    bytes
}

pub fn real_time_record(seq: u8, ticks: i32, count_rate_cps: f32, dose_rate_rh: f32) -> Vec<u8> {
    let mut bytes = data_buf_header(seq, 0, 0, ticks);
    bytes.extend_from_slice(&count_rate_cps.to_le_bytes());
    bytes.extend_from_slice(&dose_rate_rh.to_le_bytes());
    bytes.extend_from_slice(&50u16.to_le_bytes());
    bytes.extend_from_slice(&70u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.push(0);
    bytes
}

pub fn rare_record(seq: u8, ticks: i32, battery_percent: f32, temperature_c: f32) -> Vec<u8> {
    let mut bytes = data_buf_header(seq, 0, 3, ticks);
    bytes.extend_from_slice(&3600u32.to_le_bytes());
    bytes.extend_from_slice(&0.0025f32.to_le_bytes());
    bytes.extend_from_slice(&((temperature_c * 100.0 + 2000.0) as u16).to_le_bytes());
    bytes.extend_from_slice(&((battery_percent * 100.0) as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes
}
