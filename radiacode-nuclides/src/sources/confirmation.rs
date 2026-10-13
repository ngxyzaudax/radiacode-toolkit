use super::lines::detectable_lines;
use super::model::GammaSource;
use crate::chain_lines::AttributedLine;
use crate::match_peaks::resolution_fwhm_kev;

const MAJOR_LINE_FRACTION: f64 = 0.2;
const MAJOR_LINE_MIN_ENERGY_KEV: f64 = 50.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineConfirmation {
    pub found: u32,
    pub expected: u32,
}

impl LineConfirmation {
    pub fn factor(self) -> f64 {
        f64::from(self.found + 1) / f64::from(self.expected + 1)
    }
}

pub fn confirm_other_lines(
    source: &GammaSource,
    matched_energy_kev: f64,
    peaks_kev: &[f64],
    fwhm_pct: f64,
) -> LineConfirmation {
    let others: Vec<&AttributedLine> = major_lines(source)
        .filter(|line| !same_peak(line.line.energy_kev, matched_energy_kev, fwhm_pct))
        .collect();
    LineConfirmation {
        found: others
            .iter()
            .filter(|line| has_peak_near(line.line.energy_kev, peaks_kev, fwhm_pct))
            .count() as u32,
        expected: others.len() as u32,
    }
}

fn major_lines(source: &GammaSource) -> impl Iterator<Item = &'static AttributedLine> {
    let lines = detectable_lines(source);
    let floor = lines
        .first()
        .map(|line| line.scaled_intensity_pct * MAJOR_LINE_FRACTION)
        .unwrap_or(0.0);
    lines.iter().filter(move |line| {
        line.scaled_intensity_pct >= floor && line.line.energy_kev >= MAJOR_LINE_MIN_ENERGY_KEV
    })
}

fn has_peak_near(energy_kev: f64, peaks_kev: &[f64], fwhm_pct: f64) -> bool {
    peaks_kev
        .iter()
        .any(|&peak| same_peak(energy_kev, peak, fwhm_pct))
}

fn same_peak(left_kev: f64, right_kev: f64, fwhm_pct: f64) -> bool {
    (left_kev - right_kev).abs() <= resolution_fwhm_kev(left_kev, fwhm_pct) / 2.0
}
