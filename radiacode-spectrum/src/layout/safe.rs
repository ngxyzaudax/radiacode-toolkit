pub fn positive(value: f32) -> f32 {
    value.max(0.0)
}

pub fn safe_span(available: f32, reserve: f32, floor: f32) -> f32 {
    (available - reserve).max(floor)
}

pub fn clamp_max(value: f32, max: f32) -> f32 {
    value.min(max)
}
