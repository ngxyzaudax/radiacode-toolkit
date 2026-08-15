mod activity;
mod catalog;
mod chain;
mod chain_graph;
mod elements;
mod half_life;
mod index;
mod match_peaks;
mod model;
mod search;

pub use activity::{
    decay_constant_per_sec, mean_lifetime_secs, specific_activity_bq_per_g,
    specific_activity_ci_per_g, strongest_gamma, total_gamma_yield_pct,
};
pub use catalog::{catalog, energy_index, nuclide_by_id, nuclide_count, nuclide_index};
pub use chain::{ChainStep, decay_branch_label, decay_chain, decay_mode_label};
pub use chain_graph::{ChainEdge, ChainNode, DecayGraph, decay_graph, decay_graph_focused};
pub use elements::{element_symbol, nuclide_display_name};
pub use half_life::format_half_life;
pub use index::{EnergyIndex, IndexedLine, gamma_line};
pub use match_peaks::{
    MatchParams, NuclideMatch, PeakIdentification, SpectrumPeak, best_match, match_peaks,
    tolerance_kev,
};
pub use model::{Catalog, DecayBranch, DecayMode, GammaLine, Nuclide, NuclideId, RadiationKind};
pub use search::{SearchFilters, search_nuclides};

#[cfg(test)]
mod tests;
