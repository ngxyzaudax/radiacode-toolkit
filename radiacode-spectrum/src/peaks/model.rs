#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectedPeak {
    pub energy_kev: f64,
    pub net_area: f64,
    pub significance: f64,
    pub fwhm_kev: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectionParams {
    pub sigma_min: f64,
    pub detector_fwhm_pct: f64,
    pub min_net_fraction: f64,
}

impl Default for DetectionParams {
    fn default() -> Self {
        Self {
            sigma_min: 3.0,
            detector_fwhm_pct: 7.0,
            min_net_fraction: 0.02,
        }
    }
}

impl DetectionParams {
    pub fn from_app_config(config: &crate::app_config::AppConfig) -> Self {
        Self {
            sigma_min: config.peak_sensitivity_sigma,
            detector_fwhm_pct: config.detector_fwhm_pct,
            min_net_fraction: 0.02,
        }
    }
}
