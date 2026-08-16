mod activity;
mod catalog;
mod chain;
mod chain_graph;
#[cfg(test)]
mod chain_graph_tests;
mod chain_lines;
#[cfg(test)]
mod chain_tests;
mod elements;
mod equilibrium;
mod half_life;
mod index;
mod match_peaks;
mod model;
mod search;
mod series;
mod series_search;
mod source_summary;
mod topology;

pub use activity::{
    mean_lifetime_secs, specific_activity_bq_per_g, specific_activity_ci_per_g, strongest_gamma,
    total_gamma_yield_pct,
};
pub use catalog::{catalog, nuclide_by_id, nuclide_count, nuclide_index};
pub use chain::{ChainStep, decay_branch_label, decay_chain, decay_mode_label};
pub use chain_graph::{ChainEdge, ChainNode, DecayGraph, decay_graph};
pub use chain_lines::{AttributedLine, chain_lines, strongest_chain_line};
pub use elements::{element_symbol, nuclide_display_name};
pub use equilibrium::{
    MemberWeight, bottleneck_member, equilibrium_weights, time_to_equilibrium_secs,
};
pub use half_life::format_half_life;
pub use index::IndexedLine;
pub use match_peaks::{
    MatchParams, NuclideMatch, PeakIdentification, SpectrumPeak, best_match, match_peaks,
    resolution_fwhm_kev, tolerance_kev,
};
pub use model::{
    Catalog, DecayBranch, DecayCatalog, DecayMode, GammaLine, Nuclide, NuclideId, RadiationKind,
    TopologyEntry,
};
pub use search::{SearchFilters, search_nuclides};
pub use series::{
    ChainSeries, chain_series, chain_series_by_head, family_label, series_for_member,
};
pub use series_search::{ChainFilters, search_chains};
pub use source_summary::{ChainEvidence, NuclideEvidence, SourceSummary, summarize_sources};
pub use topology::{
    decay_catalog, has_emissions, topology_decays, topology_display_name, topology_entry,
    topology_half_life_secs, topology_parents,
};

#[cfg(test)]
mod tests;
