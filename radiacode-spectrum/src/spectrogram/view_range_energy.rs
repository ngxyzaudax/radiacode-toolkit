use crate::energy::{ENERGY_MAX_KEV, ENERGY_MIN_KEV};
use crate::spectrogram::view_range::{FIT_FULL_THRESHOLD, SpectrogramViewRange};

impl SpectrogramViewRange {
    pub fn fit_series_energy(&mut self, energies_kev: &[f64]) {
        if energies_kev.is_empty() {
            self.reset();
            return;
        }
        self.set_series_energy_bounds(energies_kev);
        self.energy_min_kev = self.series_energy_min_kev;
        self.energy_max_kev = self.series_energy_max_kev;
        self.channel_start = 0;
        self.row_start = 0;
        self.follow_live = true;
        self.fit_full_spectrum = true;
    }

    pub fn set_series_energy_bounds(&mut self, energies_kev: &[f64]) {
        if energies_kev.is_empty() {
            return;
        }
        let min = energies_kev
            .first()
            .copied()
            .unwrap_or(ENERGY_MIN_KEV)
            .max(ENERGY_MIN_KEV);
        let max = energies_kev
            .last()
            .copied()
            .unwrap_or(ENERGY_MAX_KEV)
            .min(ENERGY_MAX_KEV);
        self.series_energy_min_kev = min;
        self.series_energy_max_kev = max;
        self.clamp_energy_viewport();
    }

    fn clamp_energy_viewport(&mut self) {
        let bounds_min = self.series_energy_min_kev;
        let bounds_max = self.series_energy_max_kev;
        let span = (self.energy_max_kev - self.energy_min_kev).max(1.0);
        let full_span = self.full_series_span();
        if span >= full_span * FIT_FULL_THRESHOLD {
            self.energy_min_kev = bounds_min;
            self.energy_max_kev = bounds_max;
            if self.fit_full_spectrum {
                self.channel_start = 0;
            }
            return;
        }
        self.fit_full_spectrum = false;
        let mut min = self.energy_min_kev.max(bounds_min);
        let mut max = self.energy_max_kev.min(bounds_max);
        if max - min < 1.0 {
            min = bounds_min;
            max = bounds_min + span.min(full_span);
        }
        if max > bounds_max {
            max = bounds_max;
            min = (max - span).max(bounds_min);
        }
        if min < bounds_min {
            min = bounds_min;
            max = (min + span).min(bounds_max);
        }
        self.energy_min_kev = min;
        self.energy_max_kev = max.max(min + 1.0);
    }

    pub fn zoom_energy(&mut self, anchor_kev: f64, factor: f64) {
        let bounds_min = self.series_energy_min_kev;
        let bounds_max = self.series_energy_max_kev;
        let full_span = self.full_series_span();
        let span = (self.energy_max_kev - self.energy_min_kev).max(1.0);
        let new_span = (span * factor).clamp(1.0, full_span);
        if new_span >= full_span * FIT_FULL_THRESHOLD {
            self.snap_full_spectrum();
            return;
        }
        self.fit_full_spectrum = false;
        let ratio = ((anchor_kev - self.energy_min_kev) / span).clamp(0.0, 1.0);
        let mut min = anchor_kev - new_span * ratio;
        let mut max = min + new_span;
        if min < bounds_min {
            min = bounds_min;
            max = min + new_span;
        }
        if max > bounds_max {
            max = bounds_max;
            min = max - new_span;
        }
        min = min.max(bounds_min);
        max = max.min(bounds_max).max(min + 1.0);
        self.energy_min_kev = min;
        self.energy_max_kev = max;
        self.channel_start = 0;
    }

    pub fn pan_energy(&mut self, delta_kev: f64) {
        if delta_kev.abs() > 0.0 {
            self.fit_full_spectrum = false;
        }
        let bounds_min = self.series_energy_min_kev;
        let bounds_max = self.series_energy_max_kev;
        let span = self.energy_max_kev - self.energy_min_kev;
        let mut min = self.energy_min_kev + delta_kev;
        let mut max = min + span;
        if min < bounds_min {
            min = bounds_min;
            max = min + span;
        }
        if max > bounds_max {
            max = bounds_max;
            min = max - span;
        }
        self.energy_min_kev = min.max(bounds_min);
        self.energy_max_kev = max.min(bounds_max);
        self.channel_start = 0;
    }

    fn snap_full_spectrum(&mut self) {
        self.energy_min_kev = self.series_energy_min_kev;
        self.energy_max_kev = self.series_energy_max_kev;
        self.channel_start = 0;
        self.fit_full_spectrum = true;
    }

    pub fn scroll_channels(&mut self, delta: i32, channels_in_view: usize, display_cols: usize) {
        if delta != 0 {
            self.fit_full_spectrum = false;
        }
        if channels_in_view <= display_cols {
            self.channel_start = 0;
            return;
        }
        let max_start = channels_in_view - display_cols;
        let next = (self.channel_start as i32 + delta).clamp(0, max_start as i32) as usize;
        self.channel_start = next;
    }

    pub fn clamp_channels(&mut self, channels_in_view: usize, display_cols: usize) {
        if self.fit_full_spectrum {
            self.channel_start = 0;
            return;
        }
        if channels_in_view <= display_cols {
            self.channel_start = 0;
            return;
        }
        let max_start = channels_in_view - display_cols;
        self.channel_start = self.channel_start.min(max_start);
    }
}
