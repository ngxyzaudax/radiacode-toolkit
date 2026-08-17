use crate::energy::{ENERGY_MAX_KEV, ENERGY_MIN_KEV};

pub(crate) use crate::plot_zoom::FIT_FULL_THRESHOLD;

#[derive(Debug, Clone, Copy)]
pub struct SpectrogramViewRange {
    pub energy_min_kev: f64,
    pub energy_max_kev: f64,
    pub series_energy_min_kev: f64,
    pub series_energy_max_kev: f64,
    pub channel_start: usize,
    pub row_start: usize,
    pub follow_live: bool,
    pub fit_full_spectrum: bool,
}

impl SpectrogramViewRange {
    pub fn new() -> Self {
        Self {
            energy_min_kev: ENERGY_MIN_KEV,
            energy_max_kev: ENERGY_MAX_KEV,
            series_energy_min_kev: ENERGY_MIN_KEV,
            series_energy_max_kev: ENERGY_MAX_KEV,
            channel_start: 0,
            row_start: 0,
            follow_live: true,
            fit_full_spectrum: true,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn full_series_span(&self) -> f64 {
        (self.series_energy_max_kev - self.series_energy_min_kev).max(1.0)
    }
}

#[cfg(test)]
#[path = "view_range_tests.rs"]
mod view_range_tests;
