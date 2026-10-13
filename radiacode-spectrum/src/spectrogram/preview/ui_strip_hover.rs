use egui::{Rect, Response};

use crate::spectrogram::model::{SpectrogramDisplay, SpectrogramSeries};
use crate::time_format::format_hms;

fn display_mode_label(display: SpectrogramDisplay) -> &'static str {
    match display {
        SpectrogramDisplay::Live => "live",
        SpectrogramDisplay::Loaded => "library",
    }
}

fn follow_label(follow_live: bool) -> &'static str {
    if follow_live { "live" } else { "history" }
}

fn gap_suffix(series: &SpectrogramSeries) -> String {
    let (gap_count, gap_offline) = series.gap_summary();
    if gap_count > 0 {
        format!("\ngaps: {gap_count} ({})", format_hms(gap_offline))
    } else {
        String::new()
    }
}

pub fn preview_hover_text(
    series: &SpectrogramSeries,
    display: SpectrogramDisplay,
    total_rows: usize,
    follow_live: bool,
) -> String {
    let duration = format_hms(series.duration_secs());
    let interval = series.header.interval_secs.round() as u64;
    let mode = display_mode_label(display);
    let follow = follow_label(follow_live);
    let energy_max = series.energies_kev.last().copied().unwrap_or(0.0);
    let gaps = gap_suffix(series);
    format!(
        "{mode}  |  total {duration}  |  interval {interval}s  |  {} ch  |  0–{energy_max:.0} keV{gaps}\n{total_rows} rows  |  {follow}\nscroll zoom, drag pan, double-click fit all",
        series.header.channel_count
    )
}

pub fn preview_strip_response(
    response: &Response,
    strip_rect: Rect,
    series: &SpectrogramSeries,
    display: SpectrogramDisplay,
    total_rows: usize,
    follow_live: bool,
) {
    if response
        .hover_pos()
        .is_some_and(|pos| strip_rect.contains(pos))
    {
        response.clone().on_hover_text(preview_hover_text(
            series,
            display,
            total_rows,
            follow_live,
        ));
    }
}
