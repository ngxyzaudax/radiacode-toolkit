use tracing::warn;

use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{BytesBuffer, Command, VirtSfr};

use crate::device::RadiaCode;
use crate::error::{Error, Result};

use super::decode::decode_batch_words;
use super::payload::{
    all_valid_flags, encode_batch_ids, ensure_empty_payload, take_vsfr_values, validate_batch_size,
};
use super::{VSFR_BATCH_RETRIES, VSFR_BATCH_RETRY_DELAY};

pub async fn read_vsfr_batch(device: &mut RadiaCode, ids: &[VirtSfr]) -> Result<Vec<u32>> {
    validate_batch_size(ids.len())?;
    let entries = device.sfr_entries().to_vec();
    let raw_values = read_vsfr_batch_with_retries(device, ids).await?;
    Ok(decode_batch_words(ids, &raw_values, &entries))
}

async fn read_vsfr_batch_with_retries(device: &mut RadiaCode, ids: &[VirtSfr]) -> Result<Vec<u32>> {
    let mut last_error: Option<Error> = None;
    for attempt in 0..=VSFR_BATCH_RETRIES {
        if attempt > 0 {
            tokio::time::sleep(VSFR_BATCH_RETRY_DELAY).await;
        }
        match read_vsfr_batch_once(device, ids).await {
            Ok(values) => return Ok(values),
            Err(Error::Protocol(ProtocolError::VsfrBatchEmpty)) => {
                return Err(Error::Protocol(ProtocolError::VsfrBatchEmpty));
            }
            Err(error) if error.is_transient() && attempt < VSFR_BATCH_RETRIES => {
                warn!(
                    attempt,
                    ?error,
                    count = ids.len(),
                    "vsfr batch read failed, retrying"
                );
                last_error = Some(error);
            }
            Err(error) => return fill_missing_vsfrs(device, ids, error).await,
        }
    }
    fill_missing_vsfrs(
        device,
        ids,
        last_error.unwrap_or(ProtocolError::Timeout.into()),
    )
    .await
}

pub async fn read_vsfr_batch_raw(device: &mut RadiaCode, ids: &[VirtSfr]) -> Result<Vec<u32>> {
    validate_batch_size(ids.len())?;
    read_vsfr_batch_once(device, ids).await
}

async fn read_vsfr_batch_once(device: &mut RadiaCode, ids: &[VirtSfr]) -> Result<Vec<u32>> {
    let args = encode_batch_ids(ids);
    let mut response = device.execute_raw(Command::RdVirtSfrBatch, &args).await?;
    let valid_flags = response.take_u32_le()?;
    if valid_flags == 0 {
        return Err(ProtocolError::VsfrBatchEmpty.into());
    }
    if valid_flags == all_valid_flags(ids.len()) {
        return take_vsfr_values(&mut response, ids.len());
    }
    take_sparse_vsfr_values(device, ids, valid_flags, &mut response).await
}

async fn take_sparse_vsfr_values(
    device: &mut RadiaCode,
    ids: &[VirtSfr],
    valid_flags: u32,
    response: &mut BytesBuffer,
) -> Result<Vec<u32>> {
    let mut values = Vec::with_capacity(ids.len());
    for (index, id) in ids.iter().enumerate() {
        if valid_flags & (1 << index) != 0 {
            values.push(response.take_u32_le()?);
        } else {
            values.push(device.read_vsfr_u32(*id).await?);
        }
    }
    ensure_empty_payload(response)?;
    Ok(values)
}

async fn fill_missing_vsfrs(
    device: &mut RadiaCode,
    ids: &[VirtSfr],
    batch_error: Error,
) -> Result<Vec<u32>> {
    warn!(
        count = ids.len(),
        ?batch_error,
        "vsfr batch read failed, falling back to sequential reads"
    );
    let mut values = Vec::with_capacity(ids.len());
    for id in ids {
        values.push(device.read_vsfr_u32(*id).await?);
    }
    Ok(values)
}
