use egui::{Panel, Ui};

use crate::layout::breakpoint::{breakpoint_for, Breakpoint};
use crate::layout::safe::{positive, safe_span};
use crate::layout::toolbar::draw_toolbar;
use crate::theme::SPACE_SM;

const PANE_DEFAULT: f32 = 300.0;
const PANE_MIN: f32 = 220.0;
const PANE_MAX: f32 = 420.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterDetailRegion {
    Pane,
    Detail,
}

pub fn draw_master_detail(
    ui: &mut Ui,
    id: &'static str,
    pane_label: &str,
    pane_open: &mut bool,
    mut draw: impl FnMut(&mut Ui, MasterDetailRegion),
) {
    let width = ui.available_width();
    let breakpoint = breakpoint_for(width);
    if breakpoint == Breakpoint::Compact {
        draw_compact_master_detail(ui, pane_label, pane_open, &mut draw);
        return;
    }
    Panel::left(id)
        .resizable(true)
        .default_size(PANE_DEFAULT)
        .min_size(PANE_MIN)
        .max_size(PANE_MAX)
        .show(ui, |ui| {
            ui.set_min_width(safe_span(ui.available_width(), 0.0, PANE_MIN));
            ui.set_max_width(ui.available_width());
            draw(ui, MasterDetailRegion::Pane);
        });
    ui.vertical(|ui| {
        draw(ui, MasterDetailRegion::Detail);
    });
}

fn draw_compact_master_detail(
    ui: &mut Ui,
    pane_label: &str,
    pane_open: &mut bool,
    draw: &mut impl FnMut(&mut Ui, MasterDetailRegion),
) {
    draw_toolbar(ui, |ui| {
        let label = if *pane_open {
            format!("Hide {pane_label}")
        } else {
            format!("Show {pane_label}")
        };
        if ui.button(label).clicked() {
            *pane_open = !*pane_open;
        }
    });
    if *pane_open {
        let pane_height = safe_span(ui.available_height(), SPACE_SM, 160.0);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), pane_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.set_min_height(positive(pane_height));
                draw(ui, MasterDetailRegion::Pane);
            },
        );
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_SM);
    }
    draw(ui, MasterDetailRegion::Detail);
}
