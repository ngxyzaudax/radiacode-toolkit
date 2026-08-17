use super::nearest::nearest_index_within;
use super::radius::PEAK_SNAP_RADIUS_PX;

#[test]
fn nearest_within_radius() {
    let candidates = [Some(100.0_f32), Some(120.0), Some(140.0)];
    assert_eq!(
        nearest_index_within(118.0, &candidates, PEAK_SNAP_RADIUS_PX),
        Some(1)
    );
}

#[test]
fn nearest_beyond_radius_returns_none() {
    let candidates = [Some(100.0_f32), Some(200.0)];
    assert!(nearest_index_within(150.0, &candidates, PEAK_SNAP_RADIUS_PX).is_none());
}

#[test]
fn nearest_skips_none_entries() {
    let candidates = [None, Some(100.0_f32), None];
    assert_eq!(
        nearest_index_within(105.0, &candidates, PEAK_SNAP_RADIUS_PX),
        Some(1)
    );
}

#[test]
fn nearest_tie_prefers_smaller_distance() {
    let candidates = [Some(100.0_f32), Some(112.0)];
    assert_eq!(
        nearest_index_within(106.0, &candidates, PEAK_SNAP_RADIUS_PX),
        Some(0)
    );
}
