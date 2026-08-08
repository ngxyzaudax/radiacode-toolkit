use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DoseHistoryPoint {
    pub duration_secs: u32,
    pub dose: f32,
}
