use egui::Rect;

use crate::spectrogram::model::SpectrogramSeries;

pub const PREVIEW_HEIGHT: f32 = 56.0;

pub fn split_preview_area(rect: Rect) -> (Rect, Rect) {
    let preview_height = PREVIEW_HEIGHT.min(rect.height().max(0.0));
    let preview_rect = Rect::from_min_max(
        rect.min,
        egui::pos2(rect.right(), rect.top() + preview_height),
    );
    let grid_rect = Rect::from_min_max(egui::pos2(rect.left(), preview_rect.bottom()), rect.max);
    (preview_rect, grid_rect)
}

pub fn strip_rect(preview_rect: Rect, image_rect: Rect) -> Rect {
    Rect::from_min_max(
        egui::pos2(image_rect.left(), preview_rect.top()),
        egui::pos2(image_rect.right(), preview_rect.bottom()),
    )
}

pub fn column_center_x(image_rect: Rect, column_count: usize, column_index: usize) -> f32 {
    if column_count == 0 {
        return image_rect.center().x;
    }
    let column_width = image_rect.width() / column_count as f32;
    image_rect.left() + (column_index as f32 + 0.5) * column_width
}

pub fn energy_to_x(
    image_rect: Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    energy_kev: f64,
) -> Option<f32> {
    let column_count = source_cols.len();
    if column_count == 0 {
        return None;
    }
    if column_count == 1 {
        let channel_energy = series.energies_kev.get(source_cols[0]).copied()?;
        return (energy_kev == channel_energy).then_some(column_center_x(image_rect, 1, 0));
    }
    let energies: Vec<f64> = source_cols
        .iter()
        .map(|&channel| series.energies_kev.get(channel).copied().unwrap_or(0.0))
        .collect();
    let first = energies[0];
    let last = energies[column_count - 1];
    if energy_kev < first || energy_kev > last {
        return None;
    }
    let segment = energy_segment_index(&energies, energy_kev)?;
    let e0 = energies[segment];
    let e1 = energies[segment + 1];
    let fraction = if (e1 - e0).abs() < 1e-9 {
        0.0
    } else {
        ((energy_kev - e0) / (e1 - e0)).clamp(0.0, 1.0)
    };
    let x0 = column_center_x(image_rect, column_count, segment);
    let x1 = column_center_x(image_rect, column_count, segment + 1);
    Some(egui::lerp(x0..=x1, fraction as f32))
}

fn energy_segment_index(energies: &[f64], energy_kev: f64) -> Option<usize> {
    energies
        .windows(2)
        .position(|pair| energy_kev >= pair[0] && energy_kev <= pair[1])
}

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2};

    use super::{column_center_x, energy_to_x, split_preview_area, strip_rect};
    use crate::spectrogram::model::{SpectrogramHeader, SpectrogramSeries};

    fn series_with_energies(energies_kev: Vec<f64>) -> SpectrogramSeries {
        let channel_count = energies_kev.len() as u32;
        SpectrogramSeries::new(
            SpectrogramHeader {
                created_at: "t".into(),
                a0: 0.0,
                a1: 1.0,
                a2: 0.0,
                channel_count,
                interval_secs: 1.0,
                device_serial: None,
                energies_kev: energies_kev.clone(),
            },
            energies_kev,
        )
    }

    #[test]
    fn column_centers_are_half_cell_inset() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
        assert!((column_center_x(rect, 5, 0) - 10.0).abs() < 0.01);
        assert!((column_center_x(rect, 5, 4) - 90.0).abs() < 0.01);
    }

    #[test]
    fn energy_on_channel_lands_at_column_center() {
        let series = series_with_energies((0..5).map(|index| index as f64 * 100.0).collect());
        let source_cols = vec![0, 1, 2, 3, 4];
        let image_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
        let x = energy_to_x(image_rect, &series, &source_cols, 200.0).unwrap();
        assert!((x - 50.0).abs() < 0.01);
    }

    #[test]
    fn energy_outside_visible_columns_is_omitted() {
        let series = series_with_energies((0..5).map(|index| index as f64 * 100.0).collect());
        let source_cols = vec![2, 3, 4];
        let image_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
        assert!(energy_to_x(image_rect, &series, &source_cols, 50.0).is_none());
    }

    #[test]
    fn split_preview_area_shares_x_range() {
        let rect = Rect::from_min_max(pos2(10.0, 20.0), pos2(210.0, 320.0));
        let (preview, grid) = split_preview_area(rect);
        assert!(preview.bottom() <= grid.top() + 0.01);
        assert!((preview.left() - grid.left()).abs() < 0.01);
        assert!((preview.right() - grid.right()).abs() < 0.01);
    }

    #[test]
    fn strip_rect_aligns_with_image_x() {
        let preview = Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 56.0));
        let image = Rect::from_min_max(pos2(20.0, 100.0), pos2(180.0, 300.0));
        let strip = strip_rect(preview, image);
        assert!((strip.left() - 20.0).abs() < 0.01);
        assert!((strip.right() - 180.0).abs() < 0.01);
        assert!((strip.top() - 0.0).abs() < 0.01);
        assert!((strip.bottom() - 56.0).abs() < 0.01);
    }
}
