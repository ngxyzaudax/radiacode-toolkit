mod continuum;
mod curve;
mod detect;
mod matched_filter;
mod model;
mod resolution;
mod source;

#[cfg(test)]
mod detect_tests;

pub use curve::sample_curve_y;
pub use detect::detect_peaks;
pub use model::{DetectedPeak, DetectionParams};
pub use source::{
    peaks_from_collapsed, peaks_from_spectrogram_series, peaks_from_spectrum_view,
};
