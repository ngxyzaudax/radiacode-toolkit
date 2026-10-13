use crate::model::{GammaLine, RadiationKind};

pub fn strongest_gamma(gammas: &[GammaLine]) -> Option<&GammaLine> {
    gammas
        .iter()
        .filter(|line| line.kind == RadiationKind::Gamma)
        .max_by(|left, right| left.intensity_pct.total_cmp(&right.intensity_pct))
}

#[cfg(test)]
mod tests {
    use super::strongest_gamma;
    use crate::model::{DecayMode, GammaLine, RadiationKind};

    fn line(energy_kev: f64, intensity_pct: f64, kind: RadiationKind) -> GammaLine {
        GammaLine {
            energy_kev,
            intensity_pct,
            decay: DecayMode::BetaMinus,
            kind,
        }
    }

    #[test]
    fn strongest_gamma_picks_max_intensity_gamma() {
        let gammas = vec![
            line(100.0, 10.0, RadiationKind::Gamma),
            line(662.0, 85.0, RadiationKind::Gamma),
            line(32.0, 99.0, RadiationKind::XRay),
        ];
        assert_eq!(
            strongest_gamma(&gammas).map(|entry| entry.energy_kev),
            Some(662.0)
        );
        assert!(strongest_gamma(&[]).is_none());
    }
}
