use std::time::Duration;

use radiacode_bluetooth::{connect, scan_radiacode_devices};
use radiacode_core::{AlarmLimits, DataBufCursor};
use radiacode_protocol::VirtSfr;

#[tokio::main]
async fn main() {
    let devices = scan_radiacode_devices(Duration::from_secs(8))
        .await
        .expect("ble scan");
    if devices.is_empty() {
        eprintln!("no radiacode devices found");
        return;
    }
    let address = devices[0].endpoint.address_label().to_string();
    println!("connecting to {address}");
    let mut device = connect(&address).await.expect("connect");
    let config = device.load_device_config().await.expect("load config");
    println!("loaded config: brightness={} display_dir={:?} sound_on={}", config.brightness, config.display_dir, config.sound_on);
    for (label, id) in [
        ("DispBrt", VirtSfr::DispBrt),
        ("DispDir", VirtSfr::DispDir),
        ("SoundOn", VirtSfr::SoundOn),
        ("SoundCtrl", VirtSfr::SoundCtrl),
        ("DeviceCtrl", VirtSfr::DeviceCtrl),
    ] {
        let raw = device.read_vsfr_u32(id).await.expect("read vsfr");
        println!("{label}: 0x{raw:08X} ({raw})");
    }
    let units = AlarmLimits {
        l1_count_rate: 0.0,
        l2_count_rate: 0.0,
        l1_dose_rate: 0.0,
        l2_dose_rate: 0.0,
        l1_dose: 0.0,
        l2_dose: 0.0,
        dose_unit: config.alarms.dose_unit,
        count_unit: config.alarms.count_unit,
    };
    let mut cursor = DataBufCursor::default();
    let before = device
        .poll_monitor(&units, &mut cursor, false)
        .await
        .expect("poll before");
    let dose_before = before
        .0
        .accumulated
        .map(|sample| sample.dose)
        .unwrap_or(0.0);
    println!("accumulated dose before reset: {dose_before}");
    device.dose_reset().await.expect("dose reset empty payload");
    tokio::time::sleep(Duration::from_secs(2)).await;
    let after = device
        .poll_monitor(&units, &mut cursor, false)
        .await
        .expect("poll after");
    let dose_after = after
        .0
        .accumulated
        .map(|sample| sample.dose)
        .unwrap_or(0.0);
    println!("accumulated dose after reset: {dose_after}");
    println!(
        "dose_reset cleared dose: {}",
        dose_before > 0.0 && dose_after < dose_before * 0.01
    );
    let _ = device.disconnect().await;
}
