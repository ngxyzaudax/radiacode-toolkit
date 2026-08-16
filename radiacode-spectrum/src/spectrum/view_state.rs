use crate::peaks::PeakMemo;
use crate::scale::YScale;
use crate::smooth::DEFAULT_SMOOTHING_WINDOW;

pub struct SpectrumViewState {
    pub y_scale: YScale,
    pub smooth_window: usize,
    pub plot_outline_only: bool,
    pub show_spectrum_peaks: bool,
    pub peak_memo: PeakMemo,
}

impl SpectrumViewState {
    pub fn new() -> Self {
        Self {
            y_scale: YScale::Linear,
            smooth_window: DEFAULT_SMOOTHING_WINDOW,
            plot_outline_only: false,
            show_spectrum_peaks: false,
            peak_memo: PeakMemo::new(),
        }
    }
}
