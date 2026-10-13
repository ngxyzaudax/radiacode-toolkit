pub mod axes;
pub mod baseline;
pub mod capture;
mod capture_lock;
mod capture_progress;
#[cfg(test)]
mod capture_progress_tests;
pub mod color_scheme;
pub mod colormap;
pub mod controls_action;
pub mod count_rate;
pub mod delta;
pub mod gap;
pub mod ingest;
mod ingest_append;
mod ingest_baseline;
pub mod layout;
pub mod library;
pub mod library_meta;
#[cfg(test)]
mod library_tests;
pub mod model;
pub mod overlays;
pub mod peak_analysis;
#[cfg(test)]
mod peak_analysis_tests;
mod peak_cursor;
#[cfg(test)]
mod peak_cursor_tests;
pub mod preview;
pub mod rcspg;
mod rcspg_import_rows;
pub mod recording;
mod recording_control;
mod recording_seed;
pub mod settings;
pub mod state;
#[cfg(test)]
mod state_lifecycle_tests;
#[cfg(test)]
mod state_recording_tests;
pub mod storage;
#[cfg(test)]
pub(crate) mod test_fixtures;
pub mod texture;
pub mod texture_sync;
#[cfg(test)]
mod texture_sync_tests;
pub mod time_axis;
pub mod transport_gating;
pub mod ui_library;
pub mod ui_plot_area;
mod ui_plot_grid;
mod ui_plot_hover;
mod ui_plot_layers;
pub mod ui_settings;
pub mod ui_toolbar;
pub mod ui_transport;
mod ui_transport_reset;
pub mod ui_view;
pub mod view_interact;
mod view_interact_hover;
#[cfg(test)]
mod view_interact_tests;
pub mod view_range;
mod view_range_energy;
mod view_range_rows;
pub mod zscale;

pub use state::SpectrogramState;
