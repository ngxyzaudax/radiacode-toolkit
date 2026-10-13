use egui::{RichText, Ui};

use crate::layout::{MasterDetailRegion, draw_master_detail};
use crate::model::SpectrumView;
use crate::sources_tab::action::SourcesAction;
use crate::sources_tab::state::SourcesState;
use crate::sources_tab::ui_detail::draw_source_detail;
use crate::sources_tab::ui_pane::draw_sources_pane;
use crate::theme::{MUTED, SPACE_MD};

const FOOTER_RESERVE: f32 = 28.0;

#[derive(Clone, Copy)]
pub struct SourcesViewProps<'a> {
    pub fwhm_pct: f64,
    pub measured: Option<&'a SpectrumView>,
}

pub fn draw_sources_view(
    ui: &mut Ui,
    state: &mut SourcesState,
    props: SourcesViewProps<'_>,
) -> Option<SourcesAction> {
    let body_height = (ui.available_height() - FOOTER_RESERVE - SPACE_MD).max(160.0);
    ui.set_min_height(body_height);
    let mut action = None;
    let mut pane_open = state.pane_open;
    draw_master_detail(
        ui,
        "sources_list",
        "Sources",
        &mut pane_open,
        |ui, region| match region {
            MasterDetailRegion::Pane => draw_sources_pane(ui, state),
            MasterDetailRegion::Detail => {
                action = draw_source_detail(ui, state, props).or(action);
            }
        },
    );
    state.pane_open = pane_open;
    ui.add_space(SPACE_MD);
    ui.label(
        RichText::new("Gamma and X-ray lines from IAEA Livechart / ENSDF.")
            .small()
            .color(MUTED),
    );
    action
}
