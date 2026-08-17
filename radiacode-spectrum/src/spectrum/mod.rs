pub mod peak_analysis;
pub mod plot_bars;
mod quit_hotkey;
mod shutdown;
mod startup;
mod tab_navigation;
mod ui_stats;
mod view_state;

pub use quit_hotkey::quit_hotkey_pressed;
pub use shutdown::{CloseAction, ShutdownSequence};
pub use startup::StartupChrome;
pub use tab_navigation::TabNavigation;
pub use ui_stats::draw_spectrum_stats;
pub use view_state::SpectrumViewState;
