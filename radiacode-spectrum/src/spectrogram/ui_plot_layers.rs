use egui::{Painter, Pos2, Rect, Response};

use crate::identify::PeakAnalysis;
use crate::peak_overlay::draw_spectrogram_peaks;
use crate::scale::YScale;
use crate::spectrogram::axes::draw_x_axis;
use crate::spectrogram::count_rate::draw_count_rate_overlay;
use crate::spectrogram::layout::SpectrogramLayout;
use crate::spectrogram::model::{SpectrogramDisplay, SpectrogramRow, SpectrogramSeries};
use crate::spectrogram::overlays::draw_crosshair;
use crate::spectrogram::preview::{draw_preview_strip, preview_strip_response};
use crate::spectrogram::time_axis::draw_time_axis;
use crate::spectrogram::ui_plot_hover::{snapped_peak_hover, spectrogram_tooltip};
use crate::spectrogram::view_interact::hover_details;

pub struct SeriesLayers<'a> {
    pub series: &'a SpectrogramSeries,
    pub source_cols: &'a [usize],
    pub visible: &'a [SpectrogramRow],
    pub layout: SpectrogramLayout,
    pub preview_area: Rect,
    pub totals: Option<&'a [f64]>,
    pub peak_analysis: Option<&'a PeakAnalysis>,
    pub preview_scale: YScale,
    pub display: SpectrogramDisplay,
    pub row_start: usize,
    pub total_rows: usize,
    pub follow_live: bool,
}

pub fn draw_series_layers(response: &Response, painter: &Painter, layers: SeriesLayers) {
    draw_preview_layer(response, painter, &layers);
    let (effective_hover, focused) = hover_target(response, &layers);
    if let Some(analysis) = layers.peak_analysis {
        draw_spectrogram_peaks(
            painter,
            layers.layout.image_rect,
            layers.series,
            layers.source_cols,
            &analysis.identifications,
            focused,
        );
    }
    draw_count_rate_overlay(
        painter,
        layers.layout.image_rect,
        layers.layout,
        layers.visible,
    );
    if response.hover_pos().is_some() {
        draw_crosshair(
            painter,
            effective_hover,
            layers.layout.image_rect,
            Some(layers.preview_area),
        );
    }
    draw_time_axis(
        painter,
        layers.layout.image_rect,
        layers.layout,
        layers.visible,
    );
    draw_x_axis(
        painter,
        layers.layout.image_rect,
        layers.series,
        layers.source_cols,
    );
    show_tooltip(response, &layers, effective_hover, focused);
}

fn draw_preview_layer(response: &Response, painter: &Painter, layers: &SeriesLayers) {
    if let Some(totals) = layers.totals {
        draw_preview_strip(
            painter,
            layers.preview_area,
            layers.series,
            layers.source_cols,
            totals,
            layers.preview_scale,
            layers.peak_analysis,
        );
    }
    preview_strip_response(
        response,
        layers.preview_area,
        layers.series,
        layers.display,
        layers.total_rows,
        layers.follow_live,
    );
}

fn hover_target(response: &Response, layers: &SeriesLayers) -> (Pos2, Option<usize>) {
    response
        .hover_pos()
        .map(|hover| {
            snapped_peak_hover(
                hover,
                layers.layout.image_rect,
                layers.preview_area,
                layers.series,
                layers.source_cols,
                layers.peak_analysis,
            )
        })
        .unwrap_or((Pos2::ZERO, None))
}

fn show_tooltip(response: &Response, layers: &SeriesLayers, hover: Pos2, focused: Option<usize>) {
    let details = hover_details(
        hover,
        layers.layout.image_rect,
        layers.series,
        layers.visible,
        layers.row_start,
        layers.source_cols,
        layers.total_rows,
    );
    let tooltip = spectrogram_tooltip(details, layers.peak_analysis, focused);
    if !tooltip.is_empty() {
        response.clone().on_hover_text(tooltip);
    }
}
