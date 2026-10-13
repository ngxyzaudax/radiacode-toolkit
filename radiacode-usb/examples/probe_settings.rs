use radiacode_core::{extract_vsfr_value, read_vsfr_batch_raw};
use radiacode_protocol::VirtSfr;
use radiacode_usb::{connect, scan_usb_devices};

#[tokio::main]
async fn main() {
    let devices = scan_usb_devices().expect("usb scan");
    let serial = devices[0].endpoint.address_label().to_string();
    let mut device = connect(&serial).await.expect("usb connect");
    let config = device.load_device_config().await.expect("load config");
    println!("{config:#?}");
    let probe_ids = [
        VirtSfr::DispBrt,
        VirtSfr::DispDir,
        VirtSfr::SoundOn,
        VirtSfr::VibroOn,
        VirtSfr::DsUnits,
        VirtSfr::CrUnits,
    ];
    let batch_raw = read_vsfr_batch_raw(&mut device, &probe_ids)
        .await
        .expect("batch raw");
    println!("batch raw words: {batch_raw:?}");
    let batch_decoded = device
        .read_vsfr_batch(&probe_ids)
        .await
        .expect("batch decoded");
    println!("batch decoded: {batch_decoded:?}");
    for id in probe_ids {
        let single = device.read_vsfr_u32(id).await.expect("single read");
        let extracted = extract_vsfr_value(&device, id, single);
        println!("{id:?}: single={single:#010x} extracted={extracted}");
    }
    let _ = device.disconnect().await;
}
