use std::time::Duration;

use btleplug::api::ValueNotification;

pub fn notification(value: Vec<u8>) -> ValueNotification {
    ValueNotification {
        uuid: uuid::Uuid::nil(),
        value,
    }
}

pub async fn delayed_notification(delay: Duration, value: Vec<u8>) -> ValueNotification {
    tokio::time::sleep(delay).await;
    notification(value)
}

pub fn framed_response(payload: &[u8]) -> Vec<u8> {
    let mut frame = (payload.len() as i32).to_le_bytes().to_vec();
    frame.extend_from_slice(payload);
    frame
}
