use egui::{RichText, Ui, Vec2};

use crate::monitor::ui_toolbar_segments::{ToolbarSegment, segment};
use crate::settings::{SignalIconKind, paint_signal_icon};
use crate::theme::MUTED;

const LIMIT_VALUE_WIDTH: f32 = 54.0;

pub fn alarm_limit_segments<'a>(
    warning: &'a mut f32,
    danger: &'a mut f32,
    speed: f64,
    signals: [(&'a mut bool, &'a mut bool); 3],
) -> Vec<ToolbarSegment<'a>> {
    let [(warn_sound, warn_vibro), (danger_sound, danger_vibro), (oos_sound, oos_vibro)] = signals;
    [
        level_segments("Warn", Some((warning, speed)), warn_sound, warn_vibro),
        level_segments("Danger", Some((danger, speed)), danger_sound, danger_vibro),
        level_segments("OOS", None, oos_sound, oos_vibro),
    ]
    .into_iter()
    .enumerate()
    .flat_map(prepend_separator)
    .collect()
}

fn prepend_separator<'a>(
    (index, group): (usize, Vec<ToolbarSegment<'a>>),
) -> Vec<ToolbarSegment<'a>> {
    if index == 0 {
        return group;
    }
    std::iter::once(segment(|ui: &mut Ui| {
        ui.separator();
    }))
    .chain(group)
    .collect()
}

fn level_segments<'a>(
    label: &'static str,
    limit: Option<(&'a mut f32, f64)>,
    sound: &'a mut bool,
    vibro: &'a mut bool,
) -> Vec<ToolbarSegment<'a>> {
    let label_segment = segment(move |ui: &mut Ui| {
        ui.label(RichText::new(label).small().color(MUTED));
    });
    let value_segment =
        limit.map(|(value, speed)| segment(move |ui: &mut Ui| draw_limit_value(ui, value, speed)));
    [
        Some(label_segment),
        value_segment,
        Some(signal_icon_segment(SignalIconKind::Sound)),
        Some(signal_check_segment(sound, format!("{label} sound"))),
        Some(signal_icon_segment(SignalIconKind::Vibro)),
        Some(signal_check_segment(vibro, format!("{label} vibration"))),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn signal_icon_segment<'a>(kind: SignalIconKind) -> ToolbarSegment<'a> {
    segment(move |ui: &mut Ui| {
        paint_signal_icon(ui, kind, true);
    })
}

fn signal_check_segment<'a>(enabled: &'a mut bool, hover: String) -> ToolbarSegment<'a> {
    segment(move |ui: &mut Ui| {
        ui.checkbox(enabled, "").on_hover_text(hover);
    })
}

fn draw_limit_value(ui: &mut Ui, value: &mut f32, speed: f64) {
    let height = ui.spacing().interact_size.y;
    ui.add_sized(
        Vec2::new(LIMIT_VALUE_WIDTH, height),
        egui::DragValue::new(value)
            .speed(speed)
            .range(0.0..=f64::MAX)
            .min_decimals(0)
            .max_decimals(2),
    );
}
