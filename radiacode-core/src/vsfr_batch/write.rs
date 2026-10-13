use tracing::warn;

use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{Command, VirtSfr};

use crate::device::RadiaCode;
use crate::error::{Error, Result};

use super::payload::{
    all_valid_flags, encode_batch_ids, ensure_empty_payload, validate_batch_size,
};
use super::{VSFR_BATCH_RETRIES, VSFR_BATCH_RETRY_DELAY};

pub async fn write_vsfr_batch(device: &mut RadiaCode, pairs: &[(VirtSfr, u32)]) -> Result<()> {
    validate_batch_size(pairs.len())?;
    let mut last_error: Option<Error> = None;
    for attempt in 0..=VSFR_BATCH_RETRIES {
        if attempt > 0 {
            tokio::time::sleep(VSFR_BATCH_RETRY_DELAY).await;
        }
        match write_vsfr_batch_once(device, pairs).await {
            Ok(()) => return Ok(()),
            Err(error) if error.is_transient() && attempt < VSFR_BATCH_RETRIES => {
                warn!(
                    attempt,
                    ?error,
                    count = pairs.len(),
                    "vsfr batch write failed, retrying"
                );
                last_error = Some(error);
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or(ProtocolError::Timeout.into()))
}

async fn write_vsfr_batch_once(device: &mut RadiaCode, pairs: &[(VirtSfr, u32)]) -> Result<()> {
    let ids: Vec<VirtSfr> = pairs.iter().map(|(id, _)| *id).collect();
    let mut args = encode_batch_ids(&ids);
    for (_, value) in pairs {
        args.extend_from_slice(&value.to_le_bytes());
    }
    let mut response = device.execute_raw(Command::WrVirtSfrBatch, &args).await?;
    let valid_flags = response.take_u32_le()?;
    let expected = all_valid_flags(pairs.len());
    if valid_flags != expected {
        return Err(ProtocolError::ProtocolMismatch {
            expected: format!("valid_flags {expected:#x}"),
            got: format!("valid_flags {valid_flags:#x}"),
        }
        .into());
    }
    ensure_empty_payload(&mut response)
}
