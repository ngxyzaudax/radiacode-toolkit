use std::sync::Arc;

use egui::{Context, TextureOptions};

use crate::spectrogram::layout::{SpectrogramLayout, channels_in_energy_range};
use crate::spectrogram::model::{SpectrogramDisplay, SpectrogramSeries};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::texture::{SpectrogramTexture, source_columns};

const TEXTURE_NAME: &str = "spectrogram_texture";

pub fn sync_texture(ctx: &Context, state: &mut SpectrogramState, layout: SpectrogramLayout) {
    let source_cols = resolve_source_columns(state, layout);
    let (width, height) = texture_size(&source_cols, layout);
    if texture_needs_resize(&state.texture, width, height) {
        state.texture.dirty = true;
    }
    if state.texture.dirty {
        rebuild_texture(state, layout, &source_cols, width, height);
    }
    upload_texture(ctx, state);
}

fn resolve_source_columns(state: &mut SpectrogramState, layout: SpectrogramLayout) -> Vec<usize> {
    let energy_min = state.view_range.energy_min_kev;
    let energy_max = state.view_range.energy_max_kev;
    let channels_in_view = state
        .active_series()
        .map(|series| channels_in_energy_range(&series.energies_kev, energy_min, energy_max))
        .unwrap_or(layout.display_cols)
        .max(1);
    state
        .view_range
        .clamp_channels(channels_in_view, layout.display_cols);
    let channel_start = state.view_range.channel_start;
    state
        .active_series()
        .map(|series| {
            source_columns(
                series,
                energy_min,
                energy_max,
                channel_start,
                layout.display_cols,
            )
        })
        .unwrap_or_default()
}

pub fn texture_size(source_cols: &[usize], layout: SpectrogramLayout) -> (usize, usize) {
    let width = if source_cols.is_empty() {
        layout.display_cols.max(1)
    } else {
        source_cols.len()
    };
    (width, layout.display_rows.max(1))
}

fn texture_needs_resize(texture: &SpectrogramTexture, width: usize, height: usize) -> bool {
    texture.image.width() != width || texture.image.height() != height
}

fn rebuild_texture(
    state: &mut SpectrogramState,
    layout: SpectrogramLayout,
    source_cols: &[usize],
    width: usize,
    height: usize,
) {
    if state.active_series().is_some() {
        state.ensure_z_range();
    }
    let series = series_for_texture(state);
    match (series.as_deref(), state.z_range) {
        (Some(series), Some(z_range)) => {
            let total_rows = series.row_count();
            let row_start = state
                .view_range
                .visible_start(total_rows, layout.display_rows);
            let visible = series.row_window(row_start, layout.display_rows);
            state.texture.rebuild(
                series,
                visible,
                source_cols,
                layout.display_rows,
                &state.settings,
                &z_range,
            );
        }
        _ => state.texture = SpectrogramTexture::new(width, height),
    }
}

fn series_for_texture(state: &SpectrogramState) -> Option<Arc<SpectrogramSeries>> {
    match state.display {
        SpectrogramDisplay::Live => state.live_series.clone(),
        SpectrogramDisplay::Loaded => state
            .loaded_series
            .as_ref()
            .map(|series| Arc::new(series.clone())),
    }
}

fn upload_texture(ctx: &Context, state: &mut SpectrogramState) {
    if state.texture_handle.is_none() {
        state.texture_handle = Some(ctx.load_texture(
            TEXTURE_NAME,
            state.texture.image.clone(),
            TextureOptions::NEAREST,
        ));
    }
    if !state.texture.dirty {
        return;
    }
    if let Some(texture) = state.texture_handle.as_mut() {
        texture.set(state.texture.image.clone(), TextureOptions::NEAREST);
    }
    state.texture.dirty = false;
}
