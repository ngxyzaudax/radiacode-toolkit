use crate::buffer::BytesBuffer;

use crate::error::Error;
use crate::fw_version::decode_fw_version;
use crate::spectrum::decode_spectrum;

fn ascii_field(value: &str) -> Vec<u8> {
    let mut bytes = vec![value.len() as u8];
    bytes.extend_from_slice(value.as_bytes());
    bytes
}

fn fw_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&4u16.to_le_bytes());
    payload.extend_from_slice(&5u16.to_le_bytes());
    payload.extend(ascii_field("boot-date"));
    payload.extend_from_slice(&8u16.to_le_bytes());
    payload.extend_from_slice(&4u16.to_le_bytes());
    payload.extend(ascii_field("target-date"));
    payload
}

fn spectrum_header(duration: u32) -> Vec<u8> {
    let mut payload = duration.to_le_bytes().to_vec();
    payload.extend([0.0f32, 1.0, 0.0].iter().flat_map(|v| v.to_le_bytes()));
    payload
}

#[test]
fn fw_version_accepts_retcode_prefixed_payload() {
    let mut payload = 1u32.to_le_bytes().to_vec();
    payload.extend(fw_payload());
    let versions = decode_fw_version(BytesBuffer::new(payload)).unwrap();
    assert_eq!(versions.target.major, 4);
}

#[test]
fn fw_version_tolerates_trailing_nul_padding_on_full_length_payload() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&4u16.to_le_bytes());
    payload.extend_from_slice(&5u16.to_le_bytes());
    payload.extend(ascii_field("Jan 15 2024 12:00:00"));
    payload.extend_from_slice(&8u16.to_le_bytes());
    payload.extend_from_slice(&4u16.to_le_bytes());
    payload.extend(ascii_field("Feb 20 2024 08:30:00 build"));
    payload.extend(std::iter::repeat_n(0u8, 10));
    let versions = decode_fw_version(BytesBuffer::new(payload)).unwrap();
    assert_eq!(versions.target.minor, 8);
    assert_eq!(versions.target.date, "Feb 20 2024 08:30:00 build");
}

#[test]
fn fw_version_rejects_trailing_garbage_and_short_payload() {
    let mut payload = fw_payload();
    payload.push(7);
    assert!(matches!(
        decode_fw_version(BytesBuffer::new(payload)),
        Err(Error::ProtocolMismatch { .. })
    ));
    assert!(matches!(
        decode_fw_version(BytesBuffer::new(vec![1, 2])),
        Err(Error::BufferUnderrun { .. })
    ));
}

#[test]
fn spectrum_v1_decodes_every_vlen_branch() {
    let mut payload = spectrum_header(7);
    payload.extend_from_slice(&((1u16 << 4) | 1).to_le_bytes());
    payload.push(10);
    payload.extend_from_slice(&((1u16 << 4) | 2).to_le_bytes());
    payload.push((-3i8) as u8);
    payload.extend_from_slice(&((1u16 << 4) | 3).to_le_bytes());
    payload.extend_from_slice(&300i16.to_le_bytes());
    payload.extend_from_slice(&((1u16 << 4) | 4).to_le_bytes());
    payload.extend_from_slice(&[0x01, 0x00, 0x00]);
    payload.extend_from_slice(&((1u16 << 4) | 5).to_le_bytes());
    payload.extend_from_slice(&(-8i32).to_le_bytes());
    payload.extend_from_slice(&(2u16 << 4).to_le_bytes());
    let spectrum = decode_spectrum(&mut BytesBuffer::new(payload), 1).unwrap();
    assert_eq!(spectrum.counts, vec![10, 7, 307, 308, 300, 0, 0]);
    assert_eq!(spectrum.duration.as_secs(), 7);
}

#[test]
fn spectrum_v1_rejects_negative_counts_and_bad_vlen() {
    let mut negative = spectrum_header(1);
    negative.extend_from_slice(&((1u16 << 4) | 2).to_le_bytes());
    negative.push((-1i8) as u8);
    assert!(matches!(
        decode_spectrum(&mut BytesBuffer::new(negative), 1),
        Err(Error::ProtocolMismatch { .. })
    ));
    let mut bad_vlen = spectrum_header(1);
    bad_vlen.extend_from_slice(&((1u16 << 4) | 9).to_le_bytes());
    assert!(matches!(
        decode_spectrum(&mut BytesBuffer::new(bad_vlen), 1),
        Err(Error::ProtocolMismatch { .. })
    ));
}

#[test]
fn spectrum_rejects_unknown_format_and_short_header() {
    assert!(matches!(
        decode_spectrum(&mut BytesBuffer::new(spectrum_header(1)), 2),
        Err(Error::ProtocolMismatch { .. })
    ));
    assert!(matches!(
        decode_spectrum(&mut BytesBuffer::new(vec![1, 2, 3]), 0),
        Err(Error::BufferUnderrun { .. })
    ));
}
