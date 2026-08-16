use crate::model::{Nuclide, NuclideId, RadiationKind};

#[derive(Debug, Clone, Copy)]
pub struct IndexedLine {
    pub nuclide_id: NuclideId,
    pub gamma_index: u16,
    pub energy_kev: f64,
    pub intensity_pct: f64,
}

pub(crate) struct EnergyIndex {
    lines: Vec<IndexedLine>,
}

impl EnergyIndex {
    pub(crate) fn build(nuclides: &[Nuclide]) -> Self {
        let mut lines = Vec::new();
        for nuclide in nuclides {
            for (gamma_index, gamma) in nuclide.gammas.iter().enumerate() {
                if gamma.kind != RadiationKind::Gamma {
                    continue;
                }
                lines.push(IndexedLine {
                    nuclide_id: nuclide.id,
                    gamma_index: gamma_index as u16,
                    energy_kev: gamma.energy_kev,
                    intensity_pct: gamma.intensity_pct,
                });
            }
        }
        lines.sort_by(|left, right| {
            left.energy_kev
                .partial_cmp(&right.energy_kev)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self { lines }
    }

    pub(crate) fn range(&self, min_kev: f64, max_kev: f64) -> &[IndexedLine] {
        let start = self.lines.partition_point(|line| line.energy_kev < min_kev);
        let end = self
            .lines
            .partition_point(|line| line.energy_kev <= max_kev);
        &self.lines[start..end]
    }
}
