use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use radiacode_core::DeviceEndpoint;
use serde::{Deserialize, Serialize};

use crate::smooth::DEFAULT_SMOOTHING_WINDOW;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub monitor_poll_secs: u64,
    pub spectrum_refresh_secs: u64,
    pub monitor_smoothing_window: usize,
    #[serde(default = "default_monitor_window_minutes")]
    pub monitor_window_minutes: u32,
    pub remember_device: bool,
    pub last_endpoint: Option<DeviceEndpoint>,
    pub auto_connect: bool,
    pub pc_alarm_repeat: bool,
    #[serde(default = "default_match_tolerance_frac")]
    pub match_tolerance_frac: f64,
    #[serde(default = "default_match_tolerance_floor_kev")]
    pub match_tolerance_floor_kev: f64,
    #[serde(default = "default_match_min_intensity_pct")]
    pub match_min_intensity_pct: f64,
    #[serde(default = "default_peak_sensitivity_sigma")]
    pub peak_sensitivity_sigma: f64,
    #[serde(default = "default_detector_fwhm_pct")]
    pub detector_fwhm_pct: f64,
    #[serde(default = "default_catalogue_fwhm_pct")]
    pub catalogue_fwhm_pct: f64,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
}

fn default_match_tolerance_frac() -> f64 {
    0.02
}

fn default_match_tolerance_floor_kev() -> f64 {
    3.0
}

fn default_match_min_intensity_pct() -> f64 {
    1.7
}

fn default_peak_sensitivity_sigma() -> f64 {
    3.0
}

fn default_detector_fwhm_pct() -> f64 {
    7.0
}

fn default_catalogue_fwhm_pct() -> f64 {
    7.5
}

fn default_ui_scale() -> f32 {
    1.0
}

fn default_monitor_window_minutes() -> u32 {
    2
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            monitor_poll_secs: 1,
            spectrum_refresh_secs: 1,
            monitor_smoothing_window: DEFAULT_SMOOTHING_WINDOW,
            monitor_window_minutes: default_monitor_window_minutes(),
            remember_device: true,
            last_endpoint: None,
            auto_connect: false,
            pc_alarm_repeat: false,
            match_tolerance_frac: default_match_tolerance_frac(),
            match_tolerance_floor_kev: default_match_tolerance_floor_kev(),
            match_min_intensity_pct: default_match_min_intensity_pct(),
            peak_sensitivity_sigma: default_peak_sensitivity_sigma(),
            detector_fwhm_pct: default_detector_fwhm_pct(),
            catalogue_fwhm_pct: default_catalogue_fwhm_pct(),
            ui_scale: default_ui_scale(),
        }
    }
}

impl AppConfig {
    pub fn clamp(&mut self) {
        self.monitor_poll_secs = self.monitor_poll_secs.clamp(1, 60);
        self.spectrum_refresh_secs = self.spectrum_refresh_secs.clamp(1, 60);
        self.monitor_smoothing_window =
            crate::smooth::normalize_window(self.monitor_smoothing_window);
        self.monitor_window_minutes =
            crate::monitor_window::snap_window_minutes(self.monitor_window_minutes);
        self.match_tolerance_frac = self.match_tolerance_frac.clamp(0.001, 0.05);
        self.match_tolerance_floor_kev = self.match_tolerance_floor_kev.clamp(1.0, 20.0);
        self.match_min_intensity_pct = self.match_min_intensity_pct.clamp(0.1, 50.0);
        self.peak_sensitivity_sigma = self.peak_sensitivity_sigma.clamp(2.0, 15.0);
        self.detector_fwhm_pct = self.detector_fwhm_pct.clamp(4.0, 15.0);
        self.catalogue_fwhm_pct = self.catalogue_fwhm_pct.clamp(1.0, 20.0);
        self.ui_scale = self.ui_scale.clamp(0.75, 1.5);
    }

    pub fn monitor_window_secs(&self) -> f64 {
        f64::from(self.monitor_window_minutes) * 60.0
    }
}

pub fn config_path() -> PathBuf {
    ProjectDirs::from("com", "radiacode", "radiacode-spectrum")
        .map(|dirs| dirs.data_dir().join("app_config.json"))
        .unwrap_or_else(|| PathBuf::from("app_config.json"))
}

pub fn load_app_config() -> AppConfig {
    let Ok(bytes) = fs::read(config_path()) else {
        return AppConfig::default();
    };
    let mut config: AppConfig = serde_json::from_slice(&bytes).unwrap_or_default();
    config.clamp();
    config
}

pub fn save_app_config(config: &AppConfig) -> std::io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut config = config.clone();
    config.clamp();
    fs::write(path, serde_json::to_vec_pretty(&config)?)
}
