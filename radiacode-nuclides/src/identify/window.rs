#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnalysisWindow {
    pub min_kev: f64,
    pub max_kev: f64,
}

impl AnalysisWindow {
    pub fn from_peaks(peaks: &[crate::match_peaks::SpectrumPeak]) -> Self {
        let min_kev = peaks
            .iter()
            .map(|peak| peak.energy_kev)
            .fold(f64::INFINITY, f64::min);
        let max_kev = peaks
            .iter()
            .map(|peak| peak.energy_kev)
            .fold(0.0_f64, f64::max);
        Self {
            min_kev: min_kev.min(max_kev),
            max_kev,
        }
    }

    pub fn contains(&self, energy_kev: f64) -> bool {
        energy_kev >= self.min_kev && energy_kev <= self.max_kev
    }
}

impl Default for AnalysisWindow {
    fn default() -> Self {
        Self {
            min_kev: 0.0,
            max_kev: 3000.0,
        }
    }
}
