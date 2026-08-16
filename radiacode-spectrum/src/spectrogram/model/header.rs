use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrogramHeader {
    pub created_at: String,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub channel_count: u32,
    pub interval_secs: f64,
    pub device_serial: Option<String>,
    pub energies_kev: Vec<f64>,
}
