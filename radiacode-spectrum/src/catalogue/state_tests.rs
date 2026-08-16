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
