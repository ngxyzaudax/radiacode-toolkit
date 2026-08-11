use radiacode_core::RadiaCode;
use radiacode_protocol::{VirtString, decode_data_buf};
use radiacode_usb::connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let serial = std::env::args()
        .nth(1)
        .expect("usage: databuf_capture <serial> [out.bin]");
    let path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "databuf_capture.bin".into());
    let mut device: RadiaCode = connect(&serial).await?;
    let response = device.read_virt_string(VirtString::DataBuf).await?;
    let bytes = response.data().to_vec();
    std::fs::write(&path, &bytes)?;
    println!("wrote {} bytes to {path}", bytes.len());
    let frame = decode_data_buf(&bytes);
    println!(
        "records={} warnings={}",
        frame.records.len(),
        frame.warnings.len()
    );
    device.disconnect().await?;
    Ok(())
}
