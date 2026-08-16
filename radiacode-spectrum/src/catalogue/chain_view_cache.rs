use radiacode_nuclides::{DecayGraph, NuclideId};

use crate::catalogue::chain_grid_layout::layout_chain_grid;
use crate::catalogue::chain_grid_model::ChainGrid;

pub struct ChainViewCache {
    pub focus: NuclideId,
    pub graph: DecayGraph,
    pub grid: ChainGrid,
}

pub fn chain_view_cache(
    cache: &mut Option<ChainViewCache>,
    focus: NuclideId,
) -> (&DecayGraph, &ChainGrid) {
    let rebuild = cache.as_ref().is_none_or(|entry| entry.focus != focus);
    if rebuild {
        let graph = radiacode_nuclides::decay_graph(focus, 64);
        let grid = layout_chain_grid(&graph);
        *cache = Some(ChainViewCache { focus, graph, grid });
    }
    let entry = cache.as_ref().expect("chain view cache");
    (&entry.graph, &entry.grid)
}
