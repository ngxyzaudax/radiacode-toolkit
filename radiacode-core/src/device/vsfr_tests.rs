use radiacode_protocol::BytesBuffer;

use super::vsfr::trim_trailing_nul_if_needed;

fn normalize_virt_string_payload(
    buffer: &mut BytesBuffer,
    flen: usize,
) -> radiacode_protocol::Result<()> {
    trim_trailing_nul_if_needed(buffer, flen);
    let size = buffer.size();
    if size < flen {
        return Err(radiacode_protocol::Error::BufferUnderrun {
            need: flen,
            have: size,
        });
    }
    if size > flen {
        *buffer = BytesBuffer::new(buffer.data()[..flen].to_vec());
    }
    Ok(())
}

#[test]
fn truncates_overlong_virt_string_payload() {
    let payload = b"RC-110-006806-extra";
    let mut buffer = BytesBuffer::new(payload.to_vec());
    normalize_virt_string_payload(&mut buffer, 13).expect("normalize");
    assert_eq!(buffer.data(), b"RC-110-006806");
}

#[test]
fn rejects_short_virt_string_payload() {
    let mut buffer = BytesBuffer::new(b"short".to_vec());
    let error = normalize_virt_string_payload(&mut buffer, 13).expect_err("underrun");
    assert!(matches!(
        error,
        radiacode_protocol::Error::BufferUnderrun { need: 13, have: 5 }
    ));
}
