mod dispatch;
pub mod encode;
mod fake_transport;
mod virtual_device;

pub use fake_transport::DeviceHandle;
pub use virtual_device::{LEDS_SFR_FILE, ScriptedReply, VirtualDevice};

use crate::device::RadiaCode;
use crate::error::Result;

pub async fn open_device(device: VirtualDevice) -> (RadiaCode, DeviceHandle) {
    let handle = DeviceHandle::new(device);
    let radiacode = RadiaCode::open(handle.transport(Some(-60)), false, None)
        .await
        .expect("virtual device opens");
    (radiacode, handle)
}

pub async fn try_open_device(device: VirtualDevice) -> (Result<RadiaCode>, DeviceHandle) {
    let handle = DeviceHandle::new(device);
    let outcome = RadiaCode::open(handle.transport(Some(-60)), false, None).await;
    (outcome, handle)
}
