use super::energy_search::closeness;
use crate::catalog::catalog;
use crate::match_peaks::resolution_fwhm_kev;
use crate::model::{GammaLine, Nuclide, RadiationKind};
use crate::plausibility::is_plausible;

pub const MIN_NUCLIDE_LINE_INTENSITY_PCT: f64 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct NuclideLineMatch {
    pub nuclide: &'static Nuclide,
    pub line: &'static GammaLine,
    pub delta_kev: f64,
    pub score: f64,
}

pub fn match_nuclides_at_energy(
    energy_kev: f64,
    fwhm_pct: f64,
    min_supported_half_life_secs: f64,
) -> Vec<NuclideLineMatch> {
    let fwhm = resolution_fwhm_kev(energy_kev, fwhm_pct);
    let mut matches: Vec<NuclideLineMatch> = catalog()
        .nuclides
        .iter()
        .filter(|nuclide| is_plausible(nuclide.id, min_supported_half_life_secs))
        .filter_map(|nuclide| best_nuclide_line(nuclide, energy_kev, fwhm))
        .collect();
    matches.sort_by(|left, right| right.score.total_cmp(&left.score));
    matches
}

fn best_nuclide_line(
    nuclide: &'static Nuclide,
    energy_kev: f64,
    fwhm_kev: f64,
) -> Option<NuclideLineMatch> {
    nuclide
        .gammas
        .iter()
        .filter(|line| is_searchable(line, energy_kev, fwhm_kev))
        .map(|line| {
            let delta_kev = line.energy_kev - energy_kev;
            NuclideLineMatch {
                nuclide,
                line,
                delta_kev,
                score: line.intensity_pct * closeness(delta_kev, fwhm_kev),
            }
        })
        .max_by(|left, right| left.score.total_cmp(&right.score))
}

fn is_searchable(line: &GammaLine, energy_kev: f64, fwhm_kev: f64) -> bool {
    line.kind == RadiationKind::Gamma
        && line.intensity_pct >= MIN_NUCLIDE_LINE_INTENSITY_PCT
        && (line.energy_kev - energy_kev).abs() <= fwhm_kev
}
