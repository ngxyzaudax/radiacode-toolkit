use crate::buffer::BytesBuffer;
use crate::error::{Error, Result};
use crate::types::DeviceVersions;

pub fn decode_fw_version(response: BytesBuffer) -> Result<DeviceVersions> {
    let candidates = decode_fw_version_candidates(response.clone());
    for candidate in candidates {
        if let Ok(versions) = decode_fw_version_body(candidate) {
            return Ok(versions);
        }
    }
    decode_fw_version_body(response)
}

fn decode_fw_version_candidates(mut response: BytesBuffer) -> Vec<BytesBuffer> {
    let mut candidates = vec![response.clone()];
    while response.size() > 51 && response.data().last() == Some(&0) {
        let trimmed = response.data()[..response.size() - 1].to_vec();
        response = BytesBuffer::new(trimmed);
        candidates.push(response.clone());
    }
    if response.size() >= 4 {
        let mut skip_retcode = response.clone();
        if skip_retcode.take_u32_le().ok() == Some(1) {
            candidates.push(skip_retcode);
        }
    }
    candidates
}

fn decode_fw_version_body(mut response: BytesBuffer) -> Result<DeviceVersions> {
    use crate::types::FirmwareVersion;

    let boot_minor = response.take_u16_le()?;
    let boot_major = response.take_u16_le()?;
    let boot_date = response.take_length_prefixed_ascii()?;
    let target_minor = response.take_u16_le()?;
    let target_major = response.take_u16_le()?;
    let target_date = response
        .take_length_prefixed_ascii()?
        .trim_end_matches('\0')
        .to_string();
    if response.size() != 0 {
        return Err(Error::ProtocolMismatch {
            expected: "empty fw_version tail".into(),
            got: format!("{} trailing bytes", response.size()),
        });
    }
    Ok(DeviceVersions {
        boot: FirmwareVersion {
            major: boot_major,
            minor: boot_minor,
            date: boot_date,
        },
        target: FirmwareVersion {
            major: target_major,
            minor: target_minor,
            date: target_date,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::decode_fw_version;
    use crate::buffer::BytesBuffer;

    fn ascii_field(value: &str) -> Vec<u8> {
        let mut bytes = vec![value.len() as u8];
        bytes.extend_from_slice(value.as_bytes());
        bytes
    }

    #[test]
    fn decode_fw_version_parses_boot_and_target() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&4u16.to_le_bytes());
        payload.extend_from_slice(&5u16.to_le_bytes());
        payload.extend(ascii_field("boot-date"));
        payload.extend_from_slice(&8u16.to_le_bytes());
        payload.extend_from_slice(&4u16.to_le_bytes());
        payload.extend(ascii_field("target-date"));
        let versions = decode_fw_version(BytesBuffer::new(payload)).unwrap();
        assert_eq!(versions.boot.major, 5);
        assert_eq!(versions.boot.minor, 4);
        assert_eq!(versions.boot.date, "boot-date");
        assert_eq!(versions.target.major, 4);
        assert_eq!(versions.target.minor, 8);
        assert_eq!(versions.target.date, "target-date");
    }
}
