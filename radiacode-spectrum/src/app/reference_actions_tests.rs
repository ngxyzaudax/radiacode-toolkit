use super::test_harness::AppHarness;
use crate::peak_id::PeakIdAction;
use crate::peak_overlay::SpectrumPlotAction;
use crate::sources_tab::SourcesAction;
use crate::view_tab::ViewTab;

#[test]
fn plot_actions_open_library_source_for_known_nuclide() {
    let mut harness = AppHarness::new();
    let u238 = radiacode_nuclides::NuclideId::new(92, 146, 0);
    harness
        .app
        .handle_spectrum_plot_action(SpectrumPlotAction::ShowSourceFor(u238));
    assert_eq!(harness.app.tabs.active, ViewTab::Sources);
    let selected = harness.app.sources.selected.map(|source| source.key);
    assert_eq!(selected, Some("u238-series"));
}

#[test]
fn plot_actions_fall_back_to_peak_id_for_unlisted_nuclide() {
    let mut harness = AppHarness::new();
    let co56 = radiacode_nuclides::NuclideId::new(27, 29, 0);
    harness
        .app
        .handle_spectrum_plot_action(SpectrumPlotAction::ShowSourceFor(co56));
    assert_eq!(harness.app.tabs.active, ViewTab::PeakId);
    assert!((harness.app.peak_id.energy_kev - 846.8).abs() < 1.0);
}

#[test]
fn reference_tabs_link_to_each_other() {
    let mut harness = AppHarness::new();
    harness
        .app
        .handle_sources_action(SourcesAction::FindEnergy(1460.8));
    assert_eq!(harness.app.tabs.active, ViewTab::PeakId);
    assert_eq!(harness.app.peak_id.energy_kev, 1460.8);
    harness
        .app
        .handle_peak_id_action(PeakIdAction::OpenSource("k40"));
    assert_eq!(harness.app.tabs.active, ViewTab::Sources);
    let selected = harness.app.sources.selected.map(|source| source.key);
    assert_eq!(selected, Some("k40"));
}

#[test]
fn peak_energies_are_empty_without_spectrum() {
    let mut harness = AppHarness::new();
    assert!(harness.app.current_peak_energies().is_empty());
}
