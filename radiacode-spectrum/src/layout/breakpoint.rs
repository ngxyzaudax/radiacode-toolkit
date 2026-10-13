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

#[cfg(test)]
mod tests {
    use super::{Breakpoint, breakpoint_for, column_count};

    #[test]
    fn breakpoints_switch_at_width_thresholds() {
        assert_eq!(breakpoint_for(719.9), Breakpoint::Compact);
        assert_eq!(breakpoint_for(720.0), Breakpoint::Medium);
        assert_eq!(breakpoint_for(1079.9), Breakpoint::Medium);
        assert_eq!(breakpoint_for(1080.0), Breakpoint::Wide);
    }

    #[test]
    fn column_count_picks_value_for_breakpoint() {
        assert_eq!(column_count(Breakpoint::Wide, 3, 2, 1), 3);
        assert_eq!(column_count(Breakpoint::Medium, 3, 2, 1), 2);
        assert_eq!(column_count(Breakpoint::Compact, 3, 2, 1), 1);
    }
}
