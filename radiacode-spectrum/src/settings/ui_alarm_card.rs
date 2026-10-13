use egui::{Ui, Vec2};

use crate::settings::ui_alarm_cells::icon_cell;
use crate::settings::ui_alarm_style::{COMPACT, CellStyle, FIXED, PAD_Y, ROW_SPACING};
use crate::settings::ui_icons::{SignalIconKind, paint_signal_icon};

pub use crate::settings::ui_alarm_style::INNER_WIDTH;

const ROW_COUNT: f32 = 4.0;

pub type SignalPair<'a> = (&'a mut bool, &'a mut bool);

pub struct AlarmCardValues<'a> {
    pub title: &'a str,
    pub warning: &'a mut f32,
    pub danger: &'a mut f32,
    pub unit: &'a str,
    pub speed: f64,
    pub signals: [SignalPair<'a>; 3],
}

pub fn alarm_card(ui: &mut Ui, values: AlarmCardValues<'_>) {
    let row_h = ui.spacing().interact_size.y;
    let size = Vec2::new(
        ui.available_width(),
        ROW_COUNT * row_h + (ROW_COUNT - 1.0) * ROW_SPACING + PAD_Y,
    );
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_size(size);
        ui.set_max_size(size);
        framed(ui, &FIXED, |ui| {
            ui.set_min_size(Vec2::new(INNER_WIDTH, size.y - PAD_Y));
            ui.set_max_width(INNER_WIDTH);
            draw_alarm_grid(ui, values, &FIXED, false);
        });
    });
}

pub fn compact_alarm_card(ui: &mut Ui, values: AlarmCardValues<'_>) {
    framed(ui, &COMPACT, |ui| {
        draw_alarm_grid(ui, values, &COMPACT, true)
    });
}

fn framed(ui: &mut Ui, style: &CellStyle, content: impl FnOnce(&mut Ui)) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(style.frame_margin as i8))
        .show(ui, content);
}

fn draw_alarm_grid(ui: &mut Ui, values: AlarmCardValues<'_>, style: &CellStyle, compact: bool) {
    let [warn, danger, oos] = values.signals;
    egui::Grid::new(ui.id().with(("alarm_card", values.title, compact)))
        .num_columns(5)
        .spacing(style.spacing)
        .min_col_width(0.0)
        .show(ui, |ui| {
            draw_header_row(ui, style, values.title, values.unit);
            draw_level_row(
                ui,
                style,
                "Warn",
                values.warning,
                values.speed,
                values.unit,
                warn,
            );
            draw_level_row(
                ui,
                style,
                "Danger",
                values.danger,
                values.speed,
                values.unit,
                danger,
            );
            draw_oos_row(ui, style, oos);
        });
}

fn draw_header_row(ui: &mut Ui, style: &CellStyle, title: &str, unit: &str) {
    (style.title)(ui, title);
    (style.gap)(ui, style.value_width);
    (style.unit)(ui, unit);
    icon_cell(ui, |ui| paint_signal_icon(ui, SignalIconKind::Sound, true));
    icon_cell(ui, |ui| paint_signal_icon(ui, SignalIconKind::Vibro, true));
    ui.end_row();
}

fn draw_level_row(
    ui: &mut Ui,
    style: &CellStyle,
    label: &str,
    value: &mut f32,
    speed: f64,
    unit: &str,
    signals: SignalPair<'_>,
) {
    (style.label)(ui, label);
    (style.value)(ui, value, speed);
    (style.unit)(ui, unit);
    draw_signal_checks(ui, style, signals);
    ui.end_row();
}

fn draw_oos_row(ui: &mut Ui, style: &CellStyle, signals: SignalPair<'_>) {
    (style.label)(ui, "OOS");
    (style.oos)(ui);
    let height = ui.spacing().interact_size.y * style.row_scale;
    ui.allocate_exact_size(egui::vec2(style.unit_width, height), egui::Sense::hover());
    draw_signal_checks(ui, style, signals);
    ui.end_row();
}

fn draw_signal_checks(ui: &mut Ui, style: &CellStyle, (sound, vibro): SignalPair<'_>) {
    (style.check)(ui, sound, "Sound");
    (style.check)(ui, vibro, "Vibration");
}
