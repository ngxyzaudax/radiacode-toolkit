use super::confirmation::{LineConfirmation, confirm_other_lines};
use super::library::gamma_sources;
use super::lines::detectable_lines;
use super::model::GammaSource;
use crate::chain_lines::AttributedLine;
use crate::match_peaks::resolution_fwhm_kev;

#[derive(Debug, Clone, Copy)]
pub struct EnergyQuery<'a> {
    pub energy_kev: f64,
    pub fwhm_pct: f64,
    pub peaks_kev: &'a [f64],
}

#[derive(Debug, Clone, Copy)]
pub struct SourceLineMatch {
    pub source: &'static GammaSource,
    pub line: &'static AttributedLine,
    pub delta_kev: f64,
    pub confirmation: Option<LineConfirmation>,
    pub score: f64,
}

pub fn closeness(delta_kev: f64, fwhm_kev: f64) -> f64 {
    if fwhm_kev <= 0.0 {
        return 0.0;
    }
    (-4.0 * std::f64::consts::LN_2 * (delta_kev / fwhm_kev).powi(2)).exp()
}

pub fn match_sources_at_energy(query: EnergyQuery<'_>) -> Vec<SourceLineMatch> {
    let mut matches: Vec<SourceLineMatch> = gamma_sources()
        .filter_map(|source| best_source_line(source, query))
        .map(|candidate| with_confirmation(candidate, query))
        .collect();
    matches.sort_by(|left, right| right.score.total_cmp(&left.score));
    matches
}

fn best_source_line(
    source: &'static GammaSource,
    query: EnergyQuery<'_>,
) -> Option<SourceLineMatch> {
    let fwhm = resolution_fwhm_kev(query.energy_kev, query.fwhm_pct);
    detectable_lines(source)
        .iter()
        .filter(|line| (line.line.energy_kev - query.energy_kev).abs() <= fwhm)
        .map(|line| score_line(source, line, query.energy_kev, fwhm))
        .max_by(|left, right| left.score.total_cmp(&right.score))
}

fn score_line(
    source: &'static GammaSource,
    line: &'static AttributedLine,
    energy_kev: f64,
    fwhm_kev: f64,
) -> SourceLineMatch {
    let delta_kev = line.line.energy_kev - energy_kev;
    SourceLineMatch {
        source,
        line,
        delta_kev,
        confirmation: None,
        score: line.scaled_intensity_pct
            * closeness(delta_kev, fwhm_kev)
            * source.category.prevalence(),
    }
}

fn with_confirmation(candidate: SourceLineMatch, query: EnergyQuery<'_>) -> SourceLineMatch {
    if query.peaks_kev.is_empty() {
        return candidate;
    }
    let confirmation = confirm_other_lines(
        candidate.source,
        candidate.line.line.energy_kev,
        query.peaks_kev,
        query.fwhm_pct,
    );
    SourceLineMatch {
        confirmation: Some(confirmation),
        score: candidate.score * confirmation.factor(),
        ..candidate
    }
}
