use crate::buffer::BytesBuffer;
use crate::decode_spectrum;

#[test]
fn decode_spectrum_format_zero() {
    let duration = 10u32.to_le_bytes();
    let a0 = 0.0f32.to_le_bytes();
    let a1 = 1.0f32.to_le_bytes();
    let a2 = 0.0f32.to_le_bytes();
    let counts = [5u32, 10u32, 0u32].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>();
    let mut payload = Vec::new();
    payload.extend_from_slice(&duration);
    payload.extend_from_slice(&a0);
    payload.extend_from_slice(&a1);
    payload.extend_from_slice(&a2);
    payload.extend_from_slice(&counts);
    let mut buffer = BytesBuffer::new(payload);
    let spectrum = decode_spectrum(&mut buffer, 0).expect("spectrum");
    assert_eq!(spectrum.counts, vec![5, 10, 0]);
    assert_eq!(spectrum.duration.as_secs(), 10);
}

#[test]
fn decode_spectrum_format_one_zero_runs() {
    let duration = 1u32.to_le_bytes();
    let coeffs = [0.0f32, 1.0f32, 0.0f32]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    let packed = ((3u16) << 4) | 0u16;
    let mut payload = Vec::new();
    payload.extend_from_slice(&duration);
    payload.extend_from_slice(&coeffs);
    payload.extend_from_slice(&packed.to_le_bytes());
    let mut buffer = BytesBuffer::new(payload);
    let spectrum = decode_spectrum(&mut buffer, 1).expect("spectrum");
    assert_eq!(spectrum.counts, vec![0, 0, 0]);
}
