use std::env;
use std::fs;
use std::path::PathBuf;

use radiacode_protocol::decode_data_buf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("databuf_capture.bin"));
    let bytes = fs::read(&path)?;
    let frame = decode_data_buf(&bytes);
    println!("records={} warnings={}", frame.records.len(), frame.warnings.len());
    for record in &frame.records {
        println!("{record:?}");
    }
    for warning in &frame.warnings {
        println!("warning: {warning:?}");
    }
    Ok(())
}
