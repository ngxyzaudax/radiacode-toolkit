use egui::{Rect, Scene, Ui, Vec2};

use radiacode_nuclides::NuclideId;

use crate::catalogue::state::{CatalogueState, ChainFitMode};
use crate::catalogue::ui_chain_graph::{draw_chain_graph, fit_zoom};
use crate::catalogue::ui_chain_legend::draw_legend;
use crate::catalogue::ui_chain_toolbar::{ChainToolbarAction, draw_chain_toolbar};

const LEGEND_ROW_HEIGHT: f32 = 28.0;
const FIT_ZOOM_THRESHOLD: f32 = 0.75;
const ZOOM_STEP: f32 = 1.25;
const MIN_ZOOM: f32 = 0.15;
const MAX_ZOOM: f32 = 1.0;

pub fn draw_chain_viewport(
    ui: &mut Ui,
    focus: NuclideId,
    state: &mut CatalogueState,
    viewport_height: f32,
) {
    state.sync_chain_scene_focus();
    let action = draw_legend_toolbar_row(ui);
    match action {
        Some(ChainToolbarAction::Fit) => {
            state.chain_scene.needs_fit = true;
            state.chain_scene.fit_mode = ChainFitMode::All;
        }
        Some(ChainToolbarAction::Focus) => {
            state.chain_scene.needs_fit = true;
            state.chain_scene.fit_mode = ChainFitMode::Focus;
        }
        Some(ChainToolbarAction::ZoomIn | ChainToolbarAction::ZoomOut) | None => {}
    }
    let scene_height = (viewport_height - LEGEND_ROW_HEIGHT - 8.0).max(140.0);
    let viewport_size = Vec2::new(ui.available_width().max(200.0), scene_height);
    let mut scene_rect = state.chain_scene.scene_rect;
    let mut inner_rect = Rect::NOTHING;
    let mut focus_rect = Rect::NOTHING;
    let mut content_size = Vec2::ZERO;
    let mut needs_fit = state.chain_scene.needs_fit;
    let fit_mode = state.chain_scene.fit_mode;
    egui::Frame::group(ui.style())
        .inner_margin(0.0)
        .show(ui, |ui| {
            ui.set_min_height(scene_height);
            let response = Scene::new()
                .zoom_range(MIN_ZOOM..=MAX_ZOOM)
                .max_inner_size(Vec2::new(8000.0, 4000.0))
                .show(ui, &mut scene_rect, |ui| {
                    let output = draw_chain_graph(ui, focus, state);
                    inner_rect = output.content_rect;
                    focus_rect = output.focus_rect;
                    content_size = ui.min_rect().size();
                });
            if needs_fit || response.response.double_clicked() {
                let mode = if response.response.double_clicked() {
                    ChainFitMode::All
                } else {
                    fit_mode
                };
                scene_rect =
                    compute_scene_rect(mode, inner_rect, focus_rect, content_size, viewport_size);
                needs_fit = false;
            }
        });
    if let Some(ChainToolbarAction::ZoomIn) = action {
        scene_rect = zoom_scene_rect(scene_rect, viewport_size, true);
    }
    if let Some(ChainToolbarAction::ZoomOut) = action {
        scene_rect = zoom_scene_rect(scene_rect, viewport_size, false);
    }
    state.chain_scene.scene_rect = scene_rect;
    state.chain_scene.needs_fit = needs_fit;
}

fn draw_legend_toolbar_row(ui: &mut Ui) -> Option<ChainToolbarAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        draw_legend(ui);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            action = draw_chain_toolbar(ui);
        });
    });
    action
}

fn zoom_scene_rect(scene_rect: Rect, viewport_size: Vec2, zoom_in: bool) -> Rect {
    if scene_rect.width() <= 1.0 || scene_rect.height() <= 1.0 {
        return scene_rect;
    }
    let current = (viewport_size.x / scene_rect.width())
        .min(viewport_size.y / scene_rect.height())
        .clamp(MIN_ZOOM, MAX_ZOOM);
    let next = if zoom_in {
        (current * ZOOM_STEP).min(MAX_ZOOM)
    } else {
        (current / ZOOM_STEP).max(MIN_ZOOM)
    };
    if (next - current).abs() < 0.001 {
        return scene_rect;
    }
    Rect::from_center_size(scene_rect.center(), scene_rect.size() * (current / next))
}

fn compute_scene_rect(
    mode: ChainFitMode,
    inner_rect: Rect,
    focus_rect: Rect,
    content_size: Vec2,
    viewport_size: Vec2,
) -> Rect {
    if inner_rect.is_negative() {
        return Rect::NOTHING;
    }
    let use_focus = match mode {
        ChainFitMode::Focus => true,
        ChainFitMode::All => false,
        ChainFitMode::Adaptive => fit_zoom(content_size, viewport_size) < FIT_ZOOM_THRESHOLD,
    };
    if use_focus && !focus_rect.is_negative() {
        Rect::from_center_size(focus_rect.center(), viewport_size)
    } else {
        Rect::from_center_size(inner_rect.center(), inner_rect.size().max(viewport_size))
    }
}
