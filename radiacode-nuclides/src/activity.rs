use std::f64::consts::LN_2;

use crate::model::{GammaLine, RadiationKind};

const AVOGADRO: f64 = 6.022_140_76e23;
const CI_PER_BQ: f64 = 1.0 / 3.7e10;

pub fn decay_constant_per_sec(half_life_secs: Option<f64>) -> Option<f64> {
    let secs = half_life_secs?;
    if !secs.is_finite() || secs <= 0.0 {
        return None;
    }
    Some(LN_2 / secs)
}

pub fn mean_lifetime_secs(half_life_secs: Option<f64>) -> Option<f64> {
    let secs = half_life_secs?;
    if !secs.is_finite() || secs <= 0.0 {
        return None;
    }
    Some(secs / LN_2)
}

pub fn specific_activity_bq_per_g(half_life_secs: Option<f64>, mass_number: u16) -> Option<f64> {
    let secs = half_life_secs?;
    if !secs.is_finite() || secs <= 0.0 || mass_number == 0 {
        return None;
    }
    Some(LN_2 * AVOGADRO / (secs * mass_number as f64))
}

pub fn specific_activity_ci_per_g(half_life_secs: Option<f64>, mass_number: u16) -> Option<f64> {
    specific_activity_bq_per_g(half_life_secs, mass_number).map(|bq| bq * CI_PER_BQ)
}

pub fn total_gamma_yield_pct(gammas: &[GammaLine]) -> f64 {
    gammas
        .iter()
        .filter(|line| line.kind == RadiationKind::Gamma)
        .map(|gamma| gamma.intensity_pct)
        .sum()
}

pub fn strongest_gamma(gammas: &[GammaLine]) -> Option<&GammaLine> {
    gammas
        .iter()
        .filter(|line| line.kind == RadiationKind::Gamma)
        .max_by(|left, right| {
            left.intensity_pct
                .partial_cmp(&right.intensity_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DecayMode, GammaLine, RadiationKind};

    #[test]
    fn cs137_specific_activity() {
        let secs = 30.08 * 31_557_600.0;
        let bq = specific_activity_bq_per_g(Some(secs), 137).expect("activity");
        let expected = 3.2e12;
        assert!((bq - expected).abs() / expected < 0.01);
    }

    #[test]
    fn strongest_gamma_picks_max_intensity() {
        let gammas = vec![
            GammaLine {
                energy_kev: 100.0,
                intensity_pct: 10.0,
                decay: DecayMode::BetaMinus,
                kind: RadiationKind::Gamma,
            },
            GammaLine {
                energy_kev: 662.0,
                intensity_pct: 85.0,
                decay: DecayMode::BetaMinus,
                kind: RadiationKind::Gamma,
            },
        ];
        assert_eq!(strongest_gamma(&gammas).unwrap().energy_kev, 662.0);
    }
}
