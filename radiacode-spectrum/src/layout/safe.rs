pub fn positive(value: f32) -> f32 {
    value.max(0.0)
}

pub fn safe_span(available: f32, reserve: f32, floor: f32) -> f32 {
    (available - reserve).max(floor)
}

#[cfg(test)]
mod tests {
    use super::{positive, safe_span};

    #[test]
    fn positive_clamps_negative_values_to_zero() {
        assert_eq!(positive(-4.0), 0.0);
        assert_eq!(positive(2.5), 2.5);
    }

    #[test]
    fn safe_span_reserves_space_but_never_drops_below_floor() {
        assert_eq!(safe_span(100.0, 20.0, 10.0), 80.0);
        assert_eq!(safe_span(15.0, 20.0, 10.0), 10.0);
    }
}
