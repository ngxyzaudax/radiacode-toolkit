mod adjust;
pub(crate) mod candidates;
mod chains;
mod completeness;
mod lead_chain;
mod rerank;
mod verdict;
mod verdict_peaks;
mod verdict_select;
mod window;

#[cfg(test)]
mod verdict_tests;

use crate::identify::candidates::{peak_candidates, rank_candidates};
use crate::identify::chains::score_chains;
use crate::identify::lead_chain::leading_chain_for_rerank;
use crate::identify::verdict::build_verdict;
use crate::match_peaks::{MatchParams, PeakIdentification, SpectrumPeak};
use crate::series::is_natural_series_head;
use crate::source_summary::{ChainEvidence, SourceSummary, collect_nuclide_evidence};

pub use verdict::{Confidence, SourceVerdict, VerdictKind};
pub use window::AnalysisWindow;

const TOP_CANDIDATES_PER_PEAK: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct PeakAnalysisResult {
    pub identifications: Vec<PeakIdentification>,
    pub sources: SourceSummary,
}

pub fn analyze_peaks(
    peaks: &[SpectrumPeak],
    params: MatchParams,
    window: AnalysisWindow,
) -> PeakAnalysisResult {
    if peaks.is_empty() {
        return PeakAnalysisResult {
            identifications: Vec::new(),
            sources: SourceSummary::default(),
        };
    }
    let max_counts = peaks.iter().map(|peak| peak.counts).fold(1.0_f64, f64::max);
    let identifications: Vec<PeakIdentification> = peaks
        .iter()
        .map(|peak| {
            let normalized = SpectrumPeak {
                energy_kev: peak.energy_kev,
                counts: peak.counts / max_counts,
            };
            let candidates = rank_candidates(peak_candidates(normalized, params));
            PeakIdentification {
                peak: *peak,
                candidates,
            }
        })
        .collect();
    let identifications = adjust::apply_completeness_adjustments(identifications, window, params);
    let (identifications, chains) = score_and_rerank(identifications, window, params);
    let identifications = refine_natural_chain_rerank(identifications, &chains);
    let nuclides = collect_nuclide_evidence(&identifications, TOP_CANDIDATES_PER_PEAK);
    let chains = score_chains(&identifications, window, params, &nuclides);
    let verdict = build_verdict(&identifications, &chains, &nuclides);
    let mut sources = SourceSummary {
        nuclides,
        chains,
        verdict,
    };
    sources.sort_evidence();
    let identifications = rerank::rerank_identifications(identifications, &sources);
    PeakAnalysisResult {
        identifications,
        sources,
    }
}

fn score_and_rerank(
    identifications: Vec<PeakIdentification>,
    window: AnalysisWindow,
    params: MatchParams,
) -> (Vec<PeakIdentification>, Vec<ChainEvidence>) {
    let nuclides = collect_nuclide_evidence(&identifications, TOP_CANDIDATES_PER_PEAK);
    let chains = score_chains(&identifications, window, params, &nuclides);
    let identifications = if let Some(chain) = leading_chain_for_rerank(&chains) {
        rerank::rerank_for_chain_head(identifications, chain.head)
    } else {
        identifications
    };
    let nuclides = collect_nuclide_evidence(&identifications, TOP_CANDIDATES_PER_PEAK);
    let chains = score_chains(&identifications, window, params, &nuclides);
    (identifications, chains)
}

fn refine_natural_chain_rerank(
    identifications: Vec<PeakIdentification>,
    chains: &[ChainEvidence],
) -> Vec<PeakIdentification> {
    match leading_chain_for_rerank(chains).filter(|chain| is_natural_series_head(chain.head)) {
        Some(chain) => rerank::rerank_for_chain_head(identifications, chain.head),
        None => identifications,
    }
}
