use egui::{Pos2, pos2};
use radiacode_nuclides::NuclideId;

use crate::catalogue::chain_state::ChainBrowseState;
use crate::catalogue::chain_view_cache::{ChainViewCache, chain_view_cache};
use crate::catalogue::detail_layout::{DetailLayoutConfig, detail_layout, is_tight_layout};

fn u238_id() -> NuclideId {
    NuclideId::new(92, 146, 0)
}

fn config() -> DetailLayoutConfig {
    DetailLayoutConfig {
        chain_share: 0.4,
        chain_max_share: 0.6,
        preview_min_height: 100.0,
        chain_min_height: 120.0,
        section_gap: 10.0,
    }
}

#[test]
fn chain_browse_starts_with_all_series_and_filters_by_query() {
    let mut state = ChainBrowseState::new();
    let all = state.results.len();
    assert!(all >= 3);
    state.select(state.results[all - 1]);
    state.filters.query = "thorium".into();
    state.refresh_results();
    assert!(state.results.len() < all);
    assert!(!state.results.is_empty());
    assert_eq!(state.selected, None);
}

#[test]
fn reveal_marks_pending_scroll_and_clear_resets_hover_state() {
    let mut state = ChainBrowseState::new();
    state.hovered_line = Some(3);
    state.hovered_member = Some(u238_id());
    state.reveal(0);
    assert_eq!(state.selected, Some(0));
    assert!(state.pending_scroll);
    assert_eq!(state.hovered_line, None);
    assert_eq!(state.hovered_member, None);
    state.clear_selection();
    assert_eq!(state.selected, None);
    assert!(!state.pending_scroll);
}

#[test]
fn chain_view_cache_rebuilds_only_when_focus_changes() {
    let mut cache: Option<ChainViewCache> = None;
    let (graph, grid) = chain_view_cache(&mut cache, u238_id());
    assert!(!graph.nodes.is_empty());
    assert_eq!(grid.nodes.len(), graph.nodes.len());
    let first_size = grid.size;
    let (_, again) = chain_view_cache(&mut cache, u238_id());
    assert_eq!(again.size, first_size);
    let other = NuclideId::new(55, 82, 0);
    chain_view_cache(&mut cache, other);
    assert_eq!(cache.as_ref().map(|entry| entry.focus), Some(other));
}

#[test]
fn chain_grid_hit_testing_uses_scene_origin() {
    let mut cache: Option<ChainViewCache> = None;
    let (_, grid) = chain_view_cache(&mut cache, u238_id());
    let focus_center = grid.focus_rect.center();
    let origin = pos2(50.0, 20.0);
    let hit = grid.node_at(focus_center + origin.to_vec2(), origin);
    assert!(hit.is_some());
    assert!(grid.node_at(Pos2::new(-1000.0, -1000.0), origin).is_none());
    assert_eq!(grid.content_rect().size(), grid.size);
}

#[test]
fn detail_layout_splits_chain_and_preview_heights() {
    let expanded = detail_layout(500.0, false, config());
    assert_eq!(expanded.chain_height, 200.0);
    assert_eq!(expanded.preview_height, 290.0);
    let collapsed = detail_layout(500.0, true, config());
    assert_eq!(collapsed.chain_height, 0.0);
    assert_eq!(collapsed.preview_height, 500.0);
    let tight = detail_layout(100.0, false, config());
    assert!((tight.chain_height - 60.0).abs() < 1e-3);
    assert_eq!(tight.preview_height, 100.0);
    assert!(is_tight_layout(100.0, 150.0));
    assert!(!is_tight_layout(200.0, 150.0));
}
