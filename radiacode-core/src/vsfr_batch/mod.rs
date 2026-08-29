mod decode;

use tracing::warn;

use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{BytesBuffer, Command, VirtSfr, extract_vsfr_u32, resolve_value_kind};

use crate::device::RadiaCode;
use crate::error::{Error, Result};

use decode::decode_batch_words;

const VSFR_BATCH_RETRIES: usize = 2;
const VSFR_BATCH_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

pub async fn read_vsfr_batch(device: &mut RadiaCode, ids: &[VirtSfr]) -> Result<Vec<u32>> {
    if ids.is_empty() {
        return Err(ProtocolError::ProtocolMismatch {
            expected: "at least one VSFR".into(),
            got: "empty batch".into(),
        }
        .into());
    }
    let entries = device.sfr_entries().to_vec();
    let mut last_error: Option<Error> = None;
    for attempt in 0..=VSFR_BATCH_RETRIES {
        if attempt > 0 {
            tokio::time::sleep(VSFR_BATCH_RETRY_DELAY).await;
        }
        match read_vsfr_batch_once(device, ids).await {
            Ok(values) => return Ok(decode_batch_words(ids, &values, &entries)),
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
            Err(error) => {
                let values = fill_missing_vsfrs(device, ids, error).await?;
                return Ok(decode_batch_words(ids, &values, &entries));
            }
        }
    }
    let values = fill_missing_vsfrs(
        device,
        ids,
        last_error.unwrap_or(ProtocolError::Timeout.into()),
    )
    .await?;
    Ok(decode_batch_words(ids, &values, &entries))
}

pub async fn read_vsfr_batch_raw(device: &mut RadiaCode, ids: &[VirtSfr]) -> Result<Vec<u32>> {
    read_vsfr_batch_once(device, ids).await
}

pub async fn write_vsfr_batch(device: &mut RadiaCode, pairs: &[(VirtSfr, u32)]) -> Result<()> {
    if pairs.is_empty() {
        return Err(ProtocolError::ProtocolMismatch {
            expected: "at least one VSFR".into(),
            got: "empty batch".into(),
        }
        .into());
    }
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

async fn read_vsfr_batch_once(device: &mut RadiaCode, ids: &[VirtSfr]) -> Result<Vec<u32>> {
    let mut args = (ids.len() as u32).to_le_bytes().to_vec();
    for id in ids {
        args.extend_from_slice(&u32::from(*id).to_le_bytes());
    }
    let mut response = device.execute_raw(Command::RdVirtSfrBatch, &args).await?;
    let valid_flags = response.take_u32_le()?;
    if valid_flags == 0 {
        return Err(ProtocolError::VsfrBatchEmpty.into());
    }
    let expected = (1u32 << ids.len()) - 1;
    if valid_flags == expected {
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

fn take_vsfr_values(response: &mut BytesBuffer, count: usize) -> Result<Vec<u32>> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(response.take_u32_le()?);
    }
    ensure_empty_payload(response)?;
    Ok(values)
}

async fn write_vsfr_batch_once(device: &mut RadiaCode, pairs: &[(VirtSfr, u32)]) -> Result<()> {
    let mut args = (pairs.len() as u32).to_le_bytes().to_vec();
    for (id, _) in pairs {
        args.extend_from_slice(&u32::from(*id).to_le_bytes());
    }
    for (_, value) in pairs {
        args.extend_from_slice(&value.to_le_bytes());
    }
    let mut response = device.execute_raw(Command::WrVirtSfrBatch, &args).await?;
    let valid_flags = response.take_u32_le()?;
    let expected = (1u32 << pairs.len()) - 1;
    if valid_flags != expected {
        return Err(ProtocolError::ProtocolMismatch {
            expected: format!("valid_flags {expected:#x}"),
            got: format!("valid_flags {valid_flags:#x}"),
        }
        .into());
    }
    ensure_empty_payload(&mut response)
}

fn ensure_empty_payload(response: &mut BytesBuffer) -> Result<()> {
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
    let kind = resolve_value_kind(register, device.sfr_entries()).unwrap_or_else(|| {
        register
            .static_value_kind()
            .unwrap_or(radiacode_protocol::VirtSfrValueKind::U32)
    });
    extract_vsfr_u32(raw, kind)
}

#[cfg(test)]
mod tests {
    use radiacode_protocol::BytesBuffer;

    use super::take_vsfr_values;

    #[test]
    fn take_vsfr_values_reads_count_values() {
        let mut response = BytesBuffer::new(vec![1, 0, 0, 0, 2, 0, 0, 0]);
        let values = take_vsfr_values(&mut response, 2).expect("values");
        assert_eq!(values, vec![1, 2]);
    }
}
