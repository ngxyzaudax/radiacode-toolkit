use crate::app_config::AppConfig;
use crate::peaks::PeakMemo;
use crate::peaks::detect_fixtures::fixture_spectrum_view;
use crate::spectrum::peak_analysis::peak_analysis_for_spectrum;
use crate::spectrum::startup::StartupChrome;
use crate::spectrum::tab_navigation::TabNavigation;
use crate::view_tab::ViewTab;
use crate::window::startup_resize_frames;

#[test]
fn switching_tabs_is_immediate_without_dirty_monitor_draft() {
    let mut navigation = TabNavigation::new();
    assert!(navigation.try_switch(ViewTab::Spectrum, true));
    assert_eq!(navigation.active, ViewTab::Spectrum);
    assert!(navigation.pending.is_none());
    assert!(!navigation.monitor_leave_open);
}

#[test]
fn leaving_monitor_with_dirty_draft_defers_switch() {
    let mut navigation = TabNavigation::new();
    navigation.active = ViewTab::Monitor;
    assert!(navigation.try_switch(ViewTab::Monitor, true));
    assert!(!navigation.try_switch(ViewTab::Settings, true));
    assert_eq!(navigation.active, ViewTab::Monitor);
    assert_eq!(navigation.pending, Some(ViewTab::Settings));
    assert!(navigation.monitor_leave_open);
    assert!(navigation.try_switch(ViewTab::Settings, false));
    assert_eq!(navigation.active, ViewTab::Settings);
}

#[test]
fn window_icon_is_sent_once() {
    let ctx = egui::Context::default();
    let mut chrome = StartupChrome::new();
    chrome.ensure_window_icon(&ctx);
    assert!(chrome.icon_sent);
    chrome.ensure_window_icon(&ctx);
    assert!(chrome.icon_sent);
}

#[test]
fn startup_window_size_is_requested_for_a_bounded_number_of_frames() {
    let ctx = egui::Context::default();
    let mut chrome = StartupChrome::new();
    for _ in 0..(startup_resize_frames() + 3) {
        chrome.ensure_startup_window_size(&ctx);
    }
    assert_eq!(chrome.window_size_frames, startup_resize_frames());
}

#[test]
fn spectrum_peak_analysis_respects_visibility_and_memo() {
    let spectrum = fixture_spectrum_view("thoriated_rods", 600);
    let config = AppConfig::default();
    let mut memo = PeakMemo::new();
    assert!(peak_analysis_for_spectrum(&spectrum, false, &config, 1, &mut memo).is_none());
    let first = peak_analysis_for_spectrum(&spectrum, true, &config, 1, &mut memo).expect("peaks");
    assert!(!first.peaks.is_empty());
    let again = peak_analysis_for_spectrum(&spectrum, true, &config, 1, &mut memo).expect("peaks");
    assert_eq!(again.peaks.len(), first.peaks.len());
}
