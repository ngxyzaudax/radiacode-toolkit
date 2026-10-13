use crate::dosimeter::format::format_session_duration;

#[test]
fn session_duration_uses_largest_relevant_unit() {
    assert_eq!(format_session_duration(0), "0m");
    assert_eq!(format_session_duration(59), "0m");
    assert_eq!(format_session_duration(60), "1m");
    assert_eq!(format_session_duration(3_600 + 120), "1h 2m");
    assert_eq!(
        format_session_duration(86_400 * 2 + 3_600 * 3 + 60 * 4),
        "2d 3h 4m"
    );
}
