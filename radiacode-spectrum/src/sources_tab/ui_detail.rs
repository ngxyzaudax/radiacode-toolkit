use egui::{Color32, Frame, RichText, Ui};

use radiacode_nuclides::{GammaSource, detectable_lines, format_half_life, source_half_life_secs};

use crate::sources_tab::action::SourcesAction;
use crate::sources_tab::state::SourcesState;
use crate::sources_tab::ui_lines::draw_source_lines;
use crate::sources_tab::ui_pane::main_line_label;
use crate::sources_tab::ui_preview::draw_source_preview;
use crate::sources_tab::ui_view::SourcesViewProps;
use crate::theme::{MUTED, SPACE_SM, SPACE_XS};

const CHIP_FILL: Color32 = Color32::from_rgb(34, 38, 46);
const PREVIEW_SHARE: f32 = 0.45;
const PREVIEW_MIN_HEIGHT: f32 = 140.0;

pub fn draw_source_detail(
    ui: &mut Ui,
    state: &mut SourcesState,
    props: SourcesViewProps<'_>,
) -> Option<SourcesAction> {
    let Some(source) = state.selected else {
        ui.label(RichText::new("Select a source from the list.").color(MUTED));
        return None;
    };
    draw_source_header(ui, source);
    ui.add_space(SPACE_SM);
    let preview_height = (ui.available_height() * PREVIEW_SHARE).max(PREVIEW_MIN_HEIGHT);
    draw_source_preview(ui, source, state, props, preview_height);
    ui.add_space(SPACE_SM);
    draw_source_lines(ui, source, state)
}

fn draw_source_header(ui: &mut Ui, source: &GammaSource) {
    ui.label(RichText::new(source.name).strong().size(18.0));
    ui.label(RichText::new(source.found_in).color(MUTED));
    ui.add_space(SPACE_XS);
    ui.horizontal_wrapped(|ui| {
        stat_chip(ui, "Category", source.category.label().to_string());
        stat_chip(ui, "t½", format_half_life(source_half_life_secs(source)));
        stat_chip(ui, "Main line", main_line_label(source));
        stat_chip(ui, "Lines", detectable_lines(source).len().to_string());
    });
}

fn stat_chip(ui: &mut Ui, label: &str, value: String) {
    Frame::new()
        .fill(CHIP_FILL)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(11.0).color(MUTED));
                ui.add_space(SPACE_XS);
                ui.label(RichText::new(value).size(11.0));
            });
        });
}
