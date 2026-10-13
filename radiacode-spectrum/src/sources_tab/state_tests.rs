use radiacode_nuclides::{gamma_sources, source_by_key};

use crate::sources_tab::state::SourcesState;

#[test]
fn new_state_lists_every_source_and_selects_first() {
    let state = SourcesState::new();
    assert_eq!(state.results.len(), gamma_sources().count());
    assert!(state.selected.is_some());
}

#[test]
fn search_narrows_results() {
    let mut state = SourcesState::new();
    state.query = "smoke".into();
    state.refresh_results();
    assert!(state.results.iter().any(|source| source.key == "am241"));
    assert!(state.results.len() < gamma_sources().count());
}

#[test]
fn reveal_selects_and_clears_hiding_query() {
    let mut state = SourcesState::new();
    state.query = "smoke".into();
    state.refresh_results();
    state.reveal("th232-series");
    assert!(state.query.is_empty());
    assert_eq!(
        state.selected.map(|source| source.key),
        Some("th232-series")
    );
    assert!(state.pending_list_scroll);
}

#[test]
fn reveal_ignores_unknown_keys() {
    let mut state = SourcesState::new();
    let before = state.selected.map(|source| source.key);
    state.reveal("unknown");
    assert_eq!(state.selected.map(|source| source.key), before);
    assert!(!state.pending_list_scroll);
}

#[test]
fn selecting_other_source_clears_hovered_line() {
    let mut state = SourcesState::new();
    state.hovered_line = Some(3);
    state.select(source_by_key("co60").expect("co60"));
    assert!(state.hovered_line.is_none());
    assert!(state.is_selected(source_by_key("co60").expect("co60")));
}
