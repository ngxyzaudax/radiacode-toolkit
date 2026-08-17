use crate::app_config::AppConfig;
use crate::identify::{PeakAnalysis, analyze_peaks};
use crate::model::SpectrumView;
use crate::peaks::{DetectionParams, PeakMemo, PeakMemoKey, peaks_from_spectrum_view};

pub fn peak_analysis_for_spectrum(
    spectrum: &SpectrumView,
    show_peaks: bool,
    config: &AppConfig,
    spectrum_sequence: u64,
    peak_memo: &mut PeakMemo,
) -> Option<PeakAnalysis> {
    if !show_peaks {
        return None;
    }
    let params = DetectionParams::from_app_config(config);
    let key = PeakMemoKey::new(spectrum_sequence, params);
    let peaks = peak_memo
        .get_or_compute(key, || peaks_from_spectrum_view(spectrum, params))
        .to_vec();
    Some(analyze_peaks(&peaks, config))
}
