use std::collections::HashMap;

use crate::match_peaks::PeakIdentification;
use crate::model::NuclideId;
use crate::series::{ChainSeries, series_for_member};

#[derive(Debug, Clone, PartialEq)]
pub struct NuclideEvidence {
    pub id: NuclideId,
    pub display_name: String,
    pub matched_lines: u32,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainEvidence {
    pub head: NuclideId,
    pub name: String,
    pub family: String,
    pub matched_members: u32,
    pub score: f64,
    pub members: Vec<NuclideEvidence>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourceSummary {
    pub nuclides: Vec<NuclideEvidence>,
    pub chains: Vec<ChainEvidence>,
}

pub fn summarize_sources(identifications: &[PeakIdentification]) -> SourceSummary {
    let nuclides = collect_nuclide_evidence(identifications, 1);
    let chains = summarize_chains(&nuclides);
    SourceSummary { nuclides, chains }
}

fn collect_nuclide_evidence(
    identifications: &[PeakIdentification],
    max_candidates: usize,
) -> Vec<NuclideEvidence> {
    let mut nuclide_totals: HashMap<NuclideId, NuclideEvidence> = HashMap::new();
    for identification in identifications {
        for (rank, candidate) in identification
            .candidates
            .iter()
            .take(max_candidates)
            .enumerate()
        {
            let weight = 1.0 / (rank as f64 + 1.0);
            nuclide_totals
                .entry(candidate.nuclide_id)
                .and_modify(|entry| {
                    entry.matched_lines += 1;
                    entry.score += candidate.score * weight;
                })
                .or_insert(NuclideEvidence {
                    id: candidate.nuclide_id,
                    display_name: candidate.display_name.clone(),
                    matched_lines: 1,
                    score: candidate.score * weight,
                });
        }
    }
    let mut nuclides: Vec<NuclideEvidence> = nuclide_totals.into_values().collect();
    nuclides.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    nuclides
}

fn summarize_chains(nuclides: &[NuclideEvidence]) -> Vec<ChainEvidence> {
    let mut grouped: HashMap<NuclideId, ChainEvidence> = HashMap::new();
    for nuclide in nuclides {
        let Some(series) = series_for_member(nuclide.id) else {
            continue;
        };
        grouped
            .entry(series.head)
            .and_modify(|entry| {
                entry.matched_members += 1;
                entry.score += nuclide.score;
                entry.members.push(nuclide.clone());
            })
            .or_insert_with(|| chain_evidence(series, nuclide));
    }
    let mut chains: Vec<ChainEvidence> = grouped.into_values().collect();
    for chain in &mut chains {
        chain.members.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    chains.sort_by(|left, right| {
        right
            .matched_members
            .cmp(&left.matched_members)
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.name.cmp(&right.name))
    });
    chains
}

fn chain_evidence(series: &ChainSeries, nuclide: &NuclideEvidence) -> ChainEvidence {
    ChainEvidence {
        head: series.head,
        name: series.name.clone(),
        family: series.family.clone(),
        matched_members: 1,
        score: nuclide.score,
        members: vec![nuclide.clone()],
    }
}

#[cfg(test)]
mod tests {
    use super::summarize_sources;
    use crate::match_peaks::{NuclideMatch, PeakIdentification, SpectrumPeak};
    use crate::model::NuclideId;

    fn thorium_member(z: u8, n: u16, name: &str) -> PeakIdentification {
        let id = NuclideId::new(z, n, 0);
        PeakIdentification {
            peak: SpectrumPeak {
                energy_kev: 0.0,
                counts: 100.0,
            },
            candidates: vec![NuclideMatch {
                nuclide_id: id,
                display_name: name.to_string(),
                line_energy_kev: 0.0,
                intensity_pct: 50.0,
                delta_kev: 0.0,
                score: 10.0,
                matched_lines: 1,
            }],
        }
    }

    #[test]
    fn thorium_lines_group_into_series() {
        let identifications = vec![
            thorium_member(82, 130, "Pb-212"),
            thorium_member(83, 129, "Bi-212"),
            thorium_member(81, 127, "Tl-208"),
        ];
        let summary = summarize_sources(&identifications);
        assert!(
            summary
                .chains
                .iter()
                .any(|chain| chain.name.contains("Thorium")),
            "expected thorium series in {summary:?}"
        );
        assert_eq!(summary.chains[0].matched_members, 3);
    }
}
