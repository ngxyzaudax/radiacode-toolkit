use crate::catalog::nuclide_by_id;
use crate::equilibrium::MemberWeight;
use crate::model::{GammaLine, NuclideId};

#[derive(Debug, Clone, PartialEq)]
pub struct AttributedLine {
    pub line: GammaLine,
    pub source: NuclideId,
    pub source_name: String,
    pub scaled_intensity_pct: f64,
}

pub fn strongest_chain_peak_kev(weights: &[MemberWeight]) -> Option<f64> {
    let mut best_intensity = 0.0f64;
    let mut peak_kev = None;
    for member in weights {
        let Some(nuclide) = nuclide_by_id(member.id) else {
            continue;
        };
        for line in &nuclide.gammas {
            let scaled = line.intensity_pct * member.weight;
            if scaled > best_intensity {
                best_intensity = scaled;
                peak_kev = Some(line.energy_kev);
            }
        }
    }
    peak_kev
}

pub fn chain_lines(weights: &[MemberWeight]) -> Vec<AttributedLine> {
    let mut lines = Vec::new();
    for member in weights {
        let Some(nuclide) = nuclide_by_id(member.id) else {
            continue;
        };
        let source_name = nuclide.display_name.clone();
        for line in &nuclide.gammas {
            lines.push(AttributedLine {
                line: line.clone(),
                source: member.id,
                source_name: source_name.clone(),
                scaled_intensity_pct: line.intensity_pct * member.weight,
            });
        }
    }
    lines.sort_by(|left, right| {
        right
            .scaled_intensity_pct
            .partial_cmp(&left.scaled_intensity_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.line
                    .energy_kev
                    .partial_cmp(&right.line.energy_kev)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    lines
}

pub fn strongest_chain_line(lines: &[AttributedLine]) -> Option<&AttributedLine> {
    lines.iter().max_by(|left, right| {
        left.scaled_intensity_pct
            .partial_cmp(&right.scaled_intensity_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}
