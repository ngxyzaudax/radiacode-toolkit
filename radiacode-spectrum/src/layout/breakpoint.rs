#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Compact,
    Medium,
    Wide,
}

const COMPACT_MAX: f32 = 720.0;
const MEDIUM_MAX: f32 = 1080.0;

pub fn breakpoint_for(width: f32) -> Breakpoint {
    if width < COMPACT_MAX {
        Breakpoint::Compact
    } else if width < MEDIUM_MAX {
        Breakpoint::Medium
    } else {
        Breakpoint::Wide
    }
}

pub fn column_count(breakpoint: Breakpoint, wide: usize, medium: usize, compact: usize) -> usize {
    match breakpoint {
        Breakpoint::Wide => wide,
        Breakpoint::Medium => medium,
        Breakpoint::Compact => compact,
    }
}
