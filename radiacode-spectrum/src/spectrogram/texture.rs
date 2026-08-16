use egui::ColorImage;
use tracing::debug;

use crate::spectrogram::colormap::normalized_to_color;
use crate::spectrogram::model::{SpectrogramRow, SpectrogramSeries};
use crate::spectrogram::settings::SpectrogramSettings;
#[path = "texture_mapping.rs"]
mod texture_mapping;

use crate::spectrogram::zscale::{ZScaleRange, map_count};

pub struct SpectrogramTexture {
    pub image: ColorImage,
    pub dirty: bool,
}

impl SpectrogramTexture {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            image: ColorImage::filled(
                [width.max(1), height.max(1)],
                egui::Color32::from_rgb(8, 10, 16),
            ),
            dirty: true,
        }
    }

    pub fn rebuild(
        &mut self,
        series: &SpectrogramSeries,
        visible: &[SpectrogramRow],
        source_cols: &[usize],
        display_rows: usize,
        settings: &SpectrogramSettings,
        z_range: &ZScaleRange,
    ) {
        let width = source_cols.len().max(1);
        let height = display_rows.max(1);
        if self.image.width() != width || self.image.height() != height {
            self.image = ColorImage::filled([width, height], egui::Color32::from_rgb(8, 10, 16));
        } else {
            for pixel in &mut self.image.pixels {
                *pixel = egui::Color32::from_rgb(8, 10, 16);
            }
        }
        if source_cols.is_empty() || visible.is_empty() {
            self.dirty = true;
            return;
        }

        let capture_interval = series.header.interval_secs;
        let native = texture_mapping::native_rows(visible, source_cols, capture_interval);
        let start_row = height.saturating_sub(native.len());
        let mut lit_pixels = 0usize;
        for (index, row) in native.iter().enumerate() {
            let texture_row = start_row + index;
            for (col, &count) in row.iter().enumerate() {
                let t = map_count(count as f32, z_range);
                let color = if count == 0 {
                    egui::Color32::from_rgb(8, 10, 16)
                } else {
                    normalized_to_color(t, settings.palette)
                };
                if color != egui::Color32::from_rgb(8, 10, 16) {
                    lit_pixels += 1;
                }
                self.image[(col, texture_row)] = color;
            }
        }
        debug!(
            width,
            height,
            rows = native.len(),
            source_cols = source_cols.len(),
            lit_pixels,
            z_max = z_range.max,
            "spectrogram texture rebuilt"
        );
        self.dirty = true;
    }
}

pub use texture_mapping::source_columns;

#[cfg(test)]
pub use texture_mapping::native_rows;

#[cfg(test)]
#[path = "texture_tests.rs"]
mod texture_tests;
