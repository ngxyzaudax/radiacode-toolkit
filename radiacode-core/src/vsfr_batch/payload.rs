use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{BytesBuffer, VirtSfr, extract_vsfr_u32, resolve_value_kind};

use crate::device::RadiaCode;
use crate::error::Result;

pub(super) const MAX_BATCH_SIZE: usize = 32;

pub(super) fn validate_batch_size(count: usize) -> Result<()> {
    if count == 0 {
        return Err(batch_size_error("at least one VSFR", "empty batch"));
    }
    if count > MAX_BATCH_SIZE {
        return Err(batch_size_error(
            "at most 32 VSFRs",
            &format!("{count} VSFRs"),
        ));
    }
    Ok(())
}

fn batch_size_error(expected: &str, got: &str) -> crate::error::Error {
    ProtocolError::ProtocolMismatch {
        expected: expected.into(),
        got: got.into(),
    }
    .into()
}

pub(super) fn all_valid_flags(count: usize) -> u32 {
    match count {
        0 => 0,
        MAX_BATCH_SIZE.. => u32::MAX,
        bits => (1u32 << bits) - 1,
    }
}

pub(super) fn encode_batch_ids(ids: &[VirtSfr]) -> Vec<u8> {
    let mut args = (ids.len() as u32).to_le_bytes().to_vec();
    for id in ids {
        args.extend_from_slice(&u32::from(*id).to_le_bytes());
    }
    args
}

pub(super) fn take_vsfr_values(response: &mut BytesBuffer, count: usize) -> Result<Vec<u32>> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(response.take_u32_le()?);
    }
    ensure_empty_payload(response)?;
    Ok(values)
}

pub(super) fn ensure_empty_payload(response: &mut BytesBuffer) -> Result<()> {
    if response.size() != 0 {
        return Err(ProtocolError::ProtocolMismatch {
            expected: "empty payload".into(),
            got: format!("{} trailing bytes", response.size()),
        }
        .into());
    }
    Ok(())
}

pub fn extract_vsfr_value(device: &RadiaCode, register: VirtSfr, raw: u32) -> u32 {
    let kind = resolve_value_kind(register, device.sfr_entries())
        .unwrap_or(radiacode_protocol::VirtSfrValueKind::U32);
    extract_vsfr_u32(raw, kind)
}

#[cfg(test)]
mod tests {
    use radiacode_protocol::BytesBuffer;

    use super::{MAX_BATCH_SIZE, all_valid_flags, take_vsfr_values, validate_batch_size};

    #[test]
    fn take_vsfr_values_reads_count_values() {
        let mut response = BytesBuffer::new(vec![1, 0, 0, 0, 2, 0, 0, 0]);
        let values = take_vsfr_values(&mut response, 2).expect("values");
        assert_eq!(values, vec![1, 2]);
    }

    #[test]
    fn take_vsfr_values_rejects_trailing_bytes() {
        let mut response = BytesBuffer::new(vec![1, 0, 0, 0, 9]);
        assert!(take_vsfr_values(&mut response, 1).is_err());
    }

    #[test]
    fn all_valid_flags_never_overflows() {
        assert_eq!(all_valid_flags(1), 0b1);
        assert_eq!(all_valid_flags(16), 0xFFFF);
        assert_eq!(all_valid_flags(31), 0x7FFF_FFFF);
        assert_eq!(all_valid_flags(MAX_BATCH_SIZE), u32::MAX);
    }

    #[test]
    fn batch_size_bounds() {
        assert!(validate_batch_size(0).is_err());
        assert!(validate_batch_size(1).is_ok());
        assert!(validate_batch_size(MAX_BATCH_SIZE).is_ok());
        assert!(validate_batch_size(MAX_BATCH_SIZE + 1).is_err());
    }
}
