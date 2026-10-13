use egui::Ui;

use crate::settings::ui_alarm_cells::{
    CHECK_WIDTH, COMPACT_COL_SPACING, COMPACT_FRAME_MARGIN, COMPACT_ROW_SPACING,
    COMPACT_UNIT_WIDTH, COMPACT_VALUE_WIDTH, LABEL_WIDTH, UNIT_WIDTH, VALUE_WIDTH, compact_gap,
    compact_label, compact_oos_label, compact_signal_check, compact_title, compact_unit,
    compact_value, fixed_gap, fixed_label, fixed_title, fixed_unit, fixed_value, oos_label,
    signal_check,
};
use crate::theme::{SPACE_SM, SPACE_XS};

const COL_SPACING: f32 = SPACE_SM;
pub const ROW_SPACING: f32 = SPACE_XS / 2.0;
const FRAME_MARGIN: f32 = SPACE_SM;
pub const PAD_Y: f32 = FRAME_MARGIN * 2.0;
pub const INNER_WIDTH: f32 =
    LABEL_WIDTH + VALUE_WIDTH + UNIT_WIDTH + CHECK_WIDTH + CHECK_WIDTH + COL_SPACING * 4.0;

pub struct CellStyle {
    pub title: fn(&mut Ui, &str),
    pub gap: fn(&mut Ui, f32),
    pub label: fn(&mut Ui, &str),
    pub value: fn(&mut Ui, &mut f32, f64),
    pub unit: fn(&mut Ui, &str),
    pub oos: fn(&mut Ui),
    pub check: fn(&mut Ui, &mut bool, &str),
    pub value_width: f32,
    pub unit_width: f32,
    pub row_scale: f32,
    pub spacing: [f32; 2],
    pub frame_margin: f32,
}

pub const FIXED: CellStyle = CellStyle {
    title: fixed_title,
    gap: fixed_gap,
    label: fixed_label,
    value: fixed_value,
    unit: fixed_unit,
    oos: oos_label,
    check: signal_check,
    value_width: VALUE_WIDTH,
    unit_width: UNIT_WIDTH,
    row_scale: 1.0,
    spacing: [COL_SPACING, ROW_SPACING],
    frame_margin: FRAME_MARGIN,
};

pub const COMPACT: CellStyle = CellStyle {
    title: compact_title,
    gap: compact_gap,
    label: compact_label,
    value: compact_value,
    unit: compact_unit,
    oos: compact_oos_label,
    check: compact_signal_check,
    value_width: COMPACT_VALUE_WIDTH,
    unit_width: COMPACT_UNIT_WIDTH,
    row_scale: 0.85,
    spacing: [COMPACT_COL_SPACING, COMPACT_ROW_SPACING],
    frame_margin: COMPACT_FRAME_MARGIN,
};
