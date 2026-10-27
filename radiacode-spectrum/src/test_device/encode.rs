pub fn ascii_field(value: &str) -> Vec<u8> {
    std::iter::once(u8::try_from(value.len()).unwrap())
        .chain(value.bytes())
        .collect()
}

pub fn u32_words(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

pub fn concat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

pub fn fw_version_payload(boot: (u16, u16), target: (u16, u16)) -> Vec<u8> {
    concat(&[
        &boot.1.to_le_bytes(),
        &boot.0.to_le_bytes(),
        &ascii_field("Jan 01 2020"),
        &target.1.to_le_bytes(),
        &target.0.to_le_bytes(),
        &ascii_field("Feb 02 2024"),
    ])
}

pub fn hw_serial_payload(groups: &[u32]) -> Vec<u8> {
    let length = u32::try_from(groups.len() * 4).unwrap();
    concat(&[&length.to_le_bytes(), &u32_words(groups)])
}

pub fn fw_signature_payload(signature: u32, filename: &str, id_string: &str) -> Vec<u8> {
    concat(&[
        &signature.to_le_bytes(),
        &ascii_field(filename),
        &ascii_field(id_string),
    ])
}

pub fn virt_string_payload(retcode: u32, data: &[u8]) -> Vec<u8> {
    concat(&[
        &u32_words(&[retcode, u32::try_from(data.len()).unwrap()]),
        data,
    ])
}

pub fn vsfr_read_payload(value: Option<u32>) -> Vec<u8> {
    value.map_or_else(|| u32_words(&[0]), |value| u32_words(&[1, value]))
}

pub fn spectrum_v0_payload(duration_secs: u32, calibration: [f32; 3], counts: &[u32]) -> Vec<u8> {
    let coefficients: Vec<u8> = calibration
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    concat(&[
        &duration_secs.to_le_bytes(),
        &coefficients,
        &u32_words(counts),
    ])
}

pub fn calibration_payload(coefficients: [f32; 3]) -> Vec<u8> {
    coefficients
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

pub fn data_buf_header(seq: u8, entity: u8, group: u8, ticks: i32) -> Vec<u8> {
    concat(&[&[seq, entity, group], &ticks.to_le_bytes()])
}

pub fn real_time_record(seq: u8, ticks: i32, count_rate_cps: f32, dose_rate_rh: f32) -> Vec<u8> {
    concat(&[
        &data_buf_header(seq, 0, 0, ticks),
        &count_rate_cps.to_le_bytes(),
        &dose_rate_rh.to_le_bytes(),
        &50u16.to_le_bytes(),
        &70u16.to_le_bytes(),
        &0u16.to_le_bytes(),
        &[0],
    ])
}

pub fn rare_record(seq: u8, ticks: i32, battery_percent: f32, temperature_c: f32) -> Vec<u8> {
    let temperature_raw = (temperature_c * 100.0 + 2000.0) as u16;
    let battery_raw = (battery_percent * 100.0) as u16;
    concat(&[
        &data_buf_header(seq, 0, 3, ticks),
        &3600u32.to_le_bytes(),
        &0.0025f32.to_le_bytes(),
        &temperature_raw.to_le_bytes(),
        &battery_raw.to_le_bytes(),
        &0u16.to_le_bytes(),
    ])
}
