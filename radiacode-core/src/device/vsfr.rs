use tracing::debug;

use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{BytesBuffer, Command, VirtSfr, VirtString};

use crate::error::Result;

use super::RadiaCode;

impl RadiaCode {
    pub async fn read_virt_string(&mut self, id: VirtString) -> Result<BytesBuffer> {
        let mut response = self
            .execute_raw(Command::RdVirtString, &u32::from(id).to_le_bytes())
            .await?;
        let retcode = response.take_u32_le()?;
        let flen = response.take_u32_le()? as usize;
        if retcode != 1 {
            return Err(ProtocolError::UnexpectedReturnCode(retcode).into());
        }
        trim_trailing_nul_if_needed(&mut response, flen);
        let size = response.size();
        if size < flen {
            return Err(ProtocolError::BufferUnderrun {
                need: flen,
                have: size,
            }
            .into());
        }
        if size > flen {
            debug!(
                flen,
                size,
                ?id,
                "virt string payload truncated trailing bytes"
            );
            response = BytesBuffer::new(response.data()[..flen].to_vec());
        }
        Ok(response)
    }

    pub async fn read_vsfr_u32(&mut self, id: VirtSfr) -> Result<u32> {
        let mut response = self
            .execute_raw(Command::RdVirtSfr, &u32::from(id).to_le_bytes())
            .await?;
        let retcode = response.take_u32_le()?;
        if retcode != 1 {
            return Err(ProtocolError::UnexpectedReturnCode(retcode).into());
        }
        Ok(response.take_u32_le()?)
    }

    pub async fn read_vsfr_optional(&mut self, id: VirtSfr) -> Result<Option<u32>> {
        let mut response = self
            .execute_raw(Command::RdVirtSfr, &u32::from(id).to_le_bytes())
            .await?;
        let retcode = response.take_u32_le()?;
        if retcode == 1 {
            Ok(Some(response.take_u32_le()?))
        } else if retcode == 0 {
            Ok(None)
        } else {
            Err(ProtocolError::UnexpectedReturnCode(retcode).into())
        }
    }

    pub async fn write_vsfr(&mut self, id: VirtSfr, data: &[u8]) -> Result<()> {
        if !self.write_vsfr_optional(id, data).await? {
            return Err(ProtocolError::UnexpectedReturnCode(0).into());
        }
        Ok(())
    }

    pub async fn write_vsfr_optional(&mut self, id: VirtSfr, data: &[u8]) -> Result<bool> {
        let mut args = u32::from(id).to_le_bytes().to_vec();
        args.extend_from_slice(data);
        let mut response = self.execute_raw(Command::WrVirtSfr, &args).await?;
        let retcode = response.take_u32_le()?;
        if retcode == 1 {
            if response.size() != 0 {
                return Err(ProtocolError::ProtocolMismatch {
                    expected: "empty payload".into(),
                    got: format!("{} trailing bytes", response.size()),
                }
                .into());
            }
            Ok(true)
        } else if retcode == 0 {
            Ok(false)
        } else {
            Err(ProtocolError::UnexpectedReturnCode(retcode).into())
        }
    }

    pub async fn read_vsfr_f32(&mut self, id: VirtSfr) -> Result<f32> {
        let raw = self.read_vsfr_u32(id).await?;
        Ok(f32::from_le_bytes(raw.to_le_bytes()))
    }

    pub async fn read_vsfr_batch(&mut self, ids: &[VirtSfr]) -> Result<Vec<u32>> {
        crate::vsfr_batch::read_vsfr_batch(self, ids).await
    }

    pub async fn write_vsfr_batch(&mut self, pairs: &[(VirtSfr, u32)]) -> Result<()> {
        crate::vsfr_batch::write_vsfr_batch(self, pairs).await
    }
}

pub(crate) fn parse_spectrum_format_version(configuration: &str) -> u32 {
    configuration
        .lines()
        .find_map(|line| line.strip_prefix("SpecFormatVersion="))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

pub(crate) fn trim_trailing_nul_if_needed(buffer: &mut BytesBuffer, expected_len: usize) {
    let data = buffer.data();
    if data.len() == expected_len + 1 && data.last() == Some(&0) {
        let trimmed = data[..expected_len].to_vec();
        *buffer = BytesBuffer::new(trimmed);
    }
}
