pub const STARTUP_WIDTH: f32 = 1400.0;
pub const STARTUP_HEIGHT: f32 = 860.0;
pub const MIN_WIDTH: f32 = 880.0;
pub const MIN_HEIGHT: f32 = 560.0;
const STARTUP_RESIZE_FRAMES: u8 = 4;

pub fn startup_inner_size() -> [f32; 2] {
    [STARTUP_WIDTH, STARTUP_HEIGHT]
}

pub fn min_inner_size() -> [f32; 2] {
    [MIN_WIDTH, MIN_HEIGHT]
}

pub fn startup_inner_vec() -> egui::Vec2 {
    egui::vec2(STARTUP_WIDTH, STARTUP_HEIGHT)
}

pub fn startup_viewport_builder() -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_inner_size(startup_inner_size())
        .with_min_inner_size(min_inner_size())
}

pub fn startup_resize_frames() -> u8 {
    STARTUP_RESIZE_FRAMES
}
