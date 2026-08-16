use serde::{Deserialize, Serialize};

use crate::persist::json_store::{load_json, save_json};
use crate::spectrogram::color_scheme::ColorScheme;

const SETTINGS_PATH: &str = "spectrogram_settings.json";

pub const DEFAULT_CAPTURE_INTERVAL_SECS: f64 = 5.0;
pub const DEFAULT_MAX_SAMPLES: usize = 10_000;
pub const MIN_CAPTURE_INTERVAL_SECS: f64 = 1.0;
pub const MAX_CAPTURE_INTERVAL_SECS: f64 = 20.0;
pub const MIN_MAX_SAMPLES: usize = 100;
pub const MAX_MAX_SAMPLES: usize = 10_000;
pub const MIN_Z: f32 = 0.0;
pub const MAX_Z: f32 = 100.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrogramSettings {
    pub capture_interval_secs: f64,
    pub max_samples: usize,
    pub z_min: f32,
    pub z_max: f32,
    pub auto_brightness: bool,
    pub palette: ColorScheme,
    pub newest_at_bottom: bool,
    #[serde(default)]
    pub recordings_dir: String,
}

impl Default for SpectrogramSettings {
    fn default() -> Self {
        Self {
            capture_interval_secs: DEFAULT_CAPTURE_INTERVAL_SECS,
            max_samples: DEFAULT_MAX_SAMPLES,
            z_min: 0.0,
            z_max: 100.0,
            auto_brightness: true,
            palette: ColorScheme::Viridis,
            newest_at_bottom: true,
            recordings_dir: String::new(),
        }
    }
}

impl SpectrogramSettings {
    pub fn clamp(&mut self) {
        self.capture_interval_secs = self
            .capture_interval_secs
            .clamp(MIN_CAPTURE_INTERVAL_SECS, MAX_CAPTURE_INTERVAL_SECS)
            .round();
        self.max_samples = self.max_samples.clamp(MIN_MAX_SAMPLES, MAX_MAX_SAMPLES);
        self.z_min = self.z_min.clamp(MIN_Z, MAX_Z);
        self.z_max = self.z_max.clamp(MIN_Z, MAX_Z);
        if self.z_max <= self.z_min {
            self.z_max = (self.z_min + 1.0).min(MAX_Z);
        }
    }

    pub fn capture_interval(&self) -> f64 {
        self.capture_interval_secs
    }
}

pub fn load_settings() -> SpectrogramSettings {
    load_json(SETTINGS_PATH)
}

pub fn save_settings(settings: &SpectrogramSettings) -> std::io::Result<()> {
    save_json(SETTINGS_PATH, settings)
}
