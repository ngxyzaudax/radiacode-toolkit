//! In-process RadiaCode stand-in for application tests.
//!
//! `radiacode-lib` keeps this harness in its integration tests and does not
//! export it. The desktop app's worker tests need the same virtual device, so
//! it lives here until the library offers a public test-support feature.

#![allow(dead_code)]

mod dispatch;
pub mod encode;
mod fake_transport;
mod virtual_device;

pub use fake_transport::DeviceHandle;
pub use virtual_device::{ScriptedReply, VirtualDevice};

use radiacode_lib::{OpenOptions, RadiaCode, Result};

pub async fn open_device(device: VirtualDevice) -> (RadiaCode, DeviceHandle) {
    let handle = DeviceHandle::new(device);
    let radiacode = RadiaCode::open(handle.transport(Some(-60)), OpenOptions::default())
        .await
        .expect("virtual device opens");
    (radiacode, handle)
}

pub async fn try_open_device(device: VirtualDevice) -> (Result<RadiaCode>, DeviceHandle) {
    let handle = DeviceHandle::new(device);
    let outcome = RadiaCode::open(handle.transport(Some(-60)), OpenOptions::default()).await;
    (outcome, handle)
}
