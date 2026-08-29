use radiacode_protocol::{Command, VirtString};

use crate::device::RadiaCode;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareSignature {
    pub signature: u32,
    pub filename: String,
    pub id_string: String,
}

pub async fn status(device: &mut RadiaCode) -> Result<u32> {
    let mut response = device.execute_raw(Command::GetStatus, &[]).await?;
    Ok(response.take_u32_le()?)
}

pub async fn fw_signature(device: &mut RadiaCode) -> Result<FirmwareSignature> {
    let mut response = device.execute_raw(Command::FwSignature, &[]).await?;
    let signature = response.take_u32_le()?;
    let filename = response.take_length_prefixed_ascii()?;
    let id_string = response.take_length_prefixed_ascii()?;
    Ok(FirmwareSignature {
        signature,
        filename,
        id_string,
    })
}

pub async fn text_message(device: &mut RadiaCode) -> Result<String> {
    let response = device.read_virt_string(VirtString::TextMessage).await?;
    Ok(String::from_utf8_lossy(response.data()).into_owned())
}

pub async fn sfr_file_text(device: &mut RadiaCode) -> Result<String> {
    let response = device.read_virt_string(VirtString::SfrFile).await?;
    Ok(String::from_utf8_lossy(response.data()).into_owned())
}

impl RadiaCode {
    pub async fn status(&mut self) -> Result<u32> {
        status(self).await
    }

    pub async fn fw_signature(&mut self) -> Result<FirmwareSignature> {
        fw_signature(self).await
    }

    pub async fn text_message(&mut self) -> Result<String> {
        text_message(self).await
    }

    pub async fn sfr_file_text(&mut self) -> Result<String> {
        sfr_file_text(self).await
    }
}

#[cfg(test)]
mod tests {
    use radiacode_protocol::BytesBuffer;

    use super::FirmwareSignature;

    #[test]
    fn firmware_signature_fields_parse() {
        let mut buffer = BytesBuffer::new(
            [
                0x34, 0x12, 0x00, 0x00, 0x03, b'f', b'm', b'w', 0x02, b'R', b'C',
            ]
            .to_vec(),
        );
        let signature = buffer.take_u32_le().expect("signature");
        let filename = buffer.take_length_prefixed_ascii().expect("filename");
        let id_string = buffer.take_length_prefixed_ascii().expect("id");
        assert_eq!(
            FirmwareSignature {
                signature,
                filename,
                id_string,
            },
            FirmwareSignature {
                signature: 0x1234,
                filename: "fmw".into(),
                id_string: "RC".into(),
            }
        );
    }
}
