mod activity;
mod catalog;
mod chain_lines;
#[cfg(test)]
mod chain_lines_tests;
#[cfg(test)]
mod chain_tests;
mod elements;
mod equilibrium;
mod half_life;
mod identify;
mod index;
mod match_peaks;
mod model;
mod plausibility;
mod series;
mod source_summary;
mod sources;
mod topology;

pub use activity::strongest_gamma;
pub use catalog::{catalog, nuclide_by_id};
pub use chain_lines::{AttributedLine, chain_lines};
pub use equilibrium::{MemberWeight, equilibrium_weights};
pub use half_life::format_half_life;
pub use identify::{
    AnalysisWindow, Confidence, PeakAnalysisResult, SourceVerdict, VerdictKind, analyze_peaks,
};
pub use index::IndexedLine;
pub use match_peaks::{
    MatchParams, NuclideMatch, PeakIdentification, SpectrumPeak, best_match, match_peaks,
    resolution_fwhm_kev, tolerance_kev,
};
pub use model::{
    Catalog, DecayBranch, DecayCatalog, DecayMode, GammaLine, Nuclide, NuclideId, RadiationKind,
    TopologyEntry,
};
pub use plausibility::{is_plausible, supported_half_life_secs};
pub use series::{
    ChainSeries, chain_series, chain_series_by_head, family_label, is_natural_series_head,
    series_for_member,
};
pub use source_summary::{ChainEvidence, NuclideEvidence, SourceSummary, collect_nuclide_evidence};
pub use sources::{
    EnergyQuery, GammaSource, LineConfirmation, NuclideLineMatch, SourceCategory, SourceComponent,
    SourceLineMatch, SourceMember, closeness, confirm_other_lines, detectable_lines,
    filter_sources, gamma_sources, is_detectable, match_nuclides_at_energy,
    match_sources_at_energy, source_by_key, source_for_nuclide, source_half_life_secs,
    source_lines, sources_in_category,
};
pub use topology::{
    decay_catalog, has_emissions, topology_decays, topology_display_name, topology_entry,
    topology_half_life_secs, topology_parents,
};

#[cfg(test)]
mod identify_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod tests;
