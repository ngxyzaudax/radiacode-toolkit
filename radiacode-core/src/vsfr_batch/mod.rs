mod decode;
mod payload;
mod read;
#[cfg(test)]
mod tests;
mod write;

pub use payload::extract_vsfr_value;
pub use read::{read_vsfr_batch, read_vsfr_batch_raw};
pub use write::write_vsfr_batch;

const VSFR_BATCH_RETRIES: usize = 2;
const VSFR_BATCH_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(100);
