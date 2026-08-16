use egui::{Context, Ui};

use crate::app_config::AppConfig;
use crate::layout::{MasterDetailRegion, draw_master_detail};
use crate::model::ConnectionState;
use crate::peak_overlay::SpectrumPlotAction;
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::ui_library::draw_library;
use crate::spectrogram::ui_plot_area::draw_spectrogram_plot_area;
use crate::spectrogram::ui_toolbar::draw_spectrogram_toolbar;

pub fn draw_spectrogram_view(
    ui: &mut Ui,
    ctx: &Context,
    state: &mut SpectrogramState,
    config: &AppConfig,
    connection: ConnectionState,
) -> (
    Option<SpectrogramControlsAction>,
    Option<SpectrumPlotAction>,
) {
    let mut action = draw_spectrogram_toolbar(ui, state, connection);
    let mut pane_open = state.pane_open;
    let mut plot_action = None;
    draw_master_detail(
        ui,
        "spectrogram_library",
        "Library",
        &mut pane_open,
        |ui, region| match region {
            MasterDetailRegion::Pane => draw_library(ui, state, &mut action),
            MasterDetailRegion::Detail => {
                plot_action = draw_spectrogram_plot_area(ui, ctx, state, config);
            }
        },
    );
    state.pane_open = pane_open;
    (action, plot_action)
}
