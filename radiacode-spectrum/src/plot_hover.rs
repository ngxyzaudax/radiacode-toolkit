use egui_plot::{HoverPosition, PlotPoint};

use crate::scale::YScale;
use crate::spectrogram::model::RowKind;

pub fn hover_plot_point(pos: &HoverPosition<'_>) -> PlotPoint {
    match pos {
        HoverPosition::NearDataPoint { position, .. } => *position,
        HoverPosition::Elsewhere { position } => *position,
    }
}

pub fn format_spectrum_hover(energy_kev: f64, y_label: &str, y_value: &str) -> String {
    format!("{energy_kev:.1} keV\n{y_label}: {y_value}")
}

pub fn counts_plot_hover(pos: &HoverPosition<'_>, scale: YScale) -> Option<String> {
    let point = hover_plot_point(pos);
    Some(format_spectrum_hover(
        point.x,
        "Counts",
        &format_counts_value(point.y, scale),
    ))
}

pub fn rate_plot_hover(pos: &HoverPosition<'_>, scale: YScale, log_floor: f64) -> Option<String> {
    let point = hover_plot_point(pos);
    Some(format_spectrum_hover(
        point.x,
        "Rate",
        &format_rate_value(point.y, scale, log_floor),
    ))
}

pub fn relative_intensity_plot_hover(pos: &HoverPosition<'_>, scale: YScale) -> Option<String> {
    let point = hover_plot_point(pos);
    Some(format_spectrum_hover(
        point.x,
        "Relative intensity",
        &format_relative_intensity_value(point.y, scale),
    ))
}

pub struct SpectrogramHoverInfo {
    pub energy_kev: f64,
    pub counts: u32,
    pub channel: usize,
    pub absolute_row: usize,
    pub total_rows: usize,
    pub age_secs: f64,
    pub interval_secs: f64,
    pub row_total: u64,
    pub kind: RowKind,
}

pub fn spectrogram_hover_text(info: &SpectrogramHoverInfo) -> String {
    let header = format_spectrum_hover(info.energy_kev, "Counts", &info.counts.to_string());
    let kind_line = spectrogram_kind_line(info.kind, info.row_total);
    format!(
        "{header}\n\nchannel {channel}\nrow {absolute_row} / {total_rows}\n{age_secs:.0} s ago\ninterval {interval_secs:.0} s\nrow total {row_total}\n{kind_line}",
        channel = info.channel,
        absolute_row = info.absolute_row,
        total_rows = info.total_rows,
        age_secs = info.age_secs,
        interval_secs = info.interval_secs,
        row_total = info.row_total,
    )
}

fn format_counts_value(displayed: f64, scale: YScale) -> String {
    match scale {
        YScale::Linear => format!("{displayed:.1}"),
        YScale::Logarithmic => format!("{:.1}", 10_f64.powf(displayed)),
    }
}

fn format_rate_value(displayed: f64, scale: YScale, log_floor: f64) -> String {
    match scale {
        YScale::Linear => format!("{displayed:.2} cps"),
        YScale::Logarithmic => format!("{:.3} cps", 10_f64.powf(displayed) * log_floor.max(1e-12)),
    }
}

fn format_relative_intensity_value(displayed: f64, scale: YScale) -> String {
    match scale {
        YScale::Linear => format!("{displayed:.3}"),
        YScale::Logarithmic => format!("{:.4}", 10_f64.powf(displayed)),
    }
}

fn spectrogram_kind_line(kind: RowKind, total: u64) -> String {
    match kind {
        RowKind::Normal => "kind: normal".into(),
        RowKind::GapRecovery {
            offline_secs,
            raw_total,
        } => {
            let rate = if offline_secs > 0.0 {
                raw_total as f64 / offline_secs
            } else {
                0.0
            };
            format!(
                "kind: gap recovery\noffline {offline_secs:.0} s\nraw total {raw_total}\n{rate:.1} counts/s"
            )
        }
        RowKind::LiveSpike { rate_factor } => {
            format!("kind: live spike\n{rate_factor:.1}× recent median\nrow total {total}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_spectrum_hover;

    #[test]
    fn spectrum_hover_uses_shared_header() {
        let label = format_spectrum_hover(662.0, "Counts", "1234.5");
        assert_eq!(label, "662.0 keV\nCounts: 1234.5");
    }
}
