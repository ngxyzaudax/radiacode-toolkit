mod continuum;
mod curve;
mod detect;
mod matched_filter;
mod memo;
mod model;
mod resolution;
mod source;

#[cfg(test)]
mod detect_tests;

pub use curve::sample_curve_y;
pub use detect::detect_peaks;
pub use memo::{PeakMemo, PeakMemoKey};
pub use model::{DetectedPeak, DetectionParams};
#[allow(unused_imports)]
pub use source::peaks_from_spectrogram_series;
pub use source::{
    peaks_from_channel_totals, peaks_from_collapsed, peaks_from_spectrum_view,
    spectrogram_series_peak_token,
};
