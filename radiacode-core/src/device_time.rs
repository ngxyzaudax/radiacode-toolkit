use crate::device::RadiaCode;
use crate::error::Result;
use radiacode_protocol::Command;

pub async fn set_local_time_now(device: &mut RadiaCode) -> Result<()> {
    let payload = local_time_payload(std::time::SystemTime::now(), local_offset());
    device.execute_raw(Command::SetTime, &payload).await?;
    Ok(())
}

pub(crate) fn local_time_payload(now: std::time::SystemTime, offset: time::UtcOffset) -> [u8; 8] {
    let datetime = time::OffsetDateTime::from(now).to_offset(offset);
    [
        datetime.day(),
        datetime.month() as u8,
        (datetime.year() - 2000) as u8,
        0,
        datetime.second(),
        datetime.minute(),
        datetime.hour(),
        0,
    ]
}

fn local_offset() -> time::UtcOffset {
    time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::local_time_payload;

    #[test]
    fn encodes_date_and_time_fields_in_device_order() {
        let moment = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let payload = local_time_payload(moment, time::UtcOffset::UTC);
        assert_eq!(payload, [14, 11, 23, 0, 20, 13, 22, 0]);
    }

    #[test]
    fn applies_utc_offset() {
        let moment = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let offset = time::UtcOffset::from_hms(3, 0, 0).expect("offset");
        let payload = local_time_payload(moment, offset);
        assert_eq!(payload[6], 1);
        assert_eq!(payload[0], 15);
    }
}
