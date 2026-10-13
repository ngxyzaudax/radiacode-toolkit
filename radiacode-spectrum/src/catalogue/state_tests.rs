use radiacode_nuclides::{NuclideId, nuclide_index};

use crate::catalogue::state::CatalogueState;

fn u238_id() -> NuclideId {
    NuclideId::new(92, 146, 0)
}

fn th234_id() -> NuclideId {
    NuclideId::new(90, 144, 0)
}

#[test]
fn reveal_expands_and_requests_list_scroll() {
    let mut state = CatalogueState::new();
    state.reveal(u238_id());
    assert_eq!(state.selected, Some(u238_id()));
    assert!(state.pending_list_scroll);
}

#[test]
fn reveal_keeps_chain_target_in_filtered_list() {
    let mut state = CatalogueState::new();
    state.filters.query = "U-238".into();
    state.refresh_results();
    let thorium = nuclide_index(th234_id()).expect("Th-234");
    assert!(!state.results.contains(&thorium));
    state.reveal(th234_id());
    assert_eq!(state.selected, Some(th234_id()));
    assert!(state.results.contains(&thorium));
    assert!(state.pending_list_scroll);
}

#[test]
fn refreshing_results_drops_hidden_selection() {
    let mut state = CatalogueState::new();
    state.select(th234_id());
    state.filters.query = "U-238".into();
    state.refresh_results();
    assert!(state.selected.is_none());
}

#[test]
fn reselecting_same_nuclide_keeps_chain_view_cache_flag() {
    let mut state = CatalogueState::new();
    state.select(u238_id());
    state.hovered_gamma = Some(2);
    state.select(u238_id());
    assert_eq!(state.selected, Some(u238_id()));
    assert!(state.hovered_gamma.is_none());
}

#[test]
fn clearing_selection_resets_hover_and_scroll_state() {
    let mut state = CatalogueState::new();
    state.reveal(u238_id());
    state.hovered_chain_node = Some(th234_id());
    state.clear_selection();
    assert!(state.selected.is_none());
    assert!(state.hovered_chain_node.is_none());
    assert!(!state.pending_list_scroll);
}

#[test]
fn selecting_chain_by_head_switches_to_chain_mode() {
    let mut state = CatalogueState::new();
    state.select_chain_by_head(u238_id());
    assert_eq!(
        state.mode,
        crate::catalogue::browse_mode::CatalogueMode::Chains
    );
    assert!(state.chains.selected.is_some());
    let mode_before = state.mode;
    state.select_chain_by_head(NuclideId::new(1, 0, 0));
    assert_eq!(state.mode, mode_before);
}

#[test]
fn tab_enter_refreshes_both_result_lists() {
    let mut state = CatalogueState::new();
    state.results.clear();
    state.on_tab_enter();
    assert!(!state.results.is_empty());
}
