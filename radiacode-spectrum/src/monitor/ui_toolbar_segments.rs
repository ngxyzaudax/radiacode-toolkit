use egui::Ui;

pub type ToolbarSegment<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

pub fn segment<'a>(draw: impl FnOnce(&mut Ui) + 'a) -> ToolbarSegment<'a> {
    Box::new(draw)
}

/// Adds segments listed in visual left-to-right order into a right-to-left `Ui`,
/// where the first widget added ends up rightmost.
pub fn draw_segments_right_aligned(ui: &mut Ui, segments: Vec<ToolbarSegment<'_>>) {
    segments
        .into_iter()
        .rev()
        .for_each(|draw_segment| draw_segment(ui));
}
