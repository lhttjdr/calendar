use super::*;

#[test]
fn ephemeris_default_paths_are_set() {
    assert!(!DEFAULT_VSOP87_EARTH_PATH.is_empty());
    assert!(DEFAULT_VSOP87_EARTH_PATH.contains("VSOP87"));
    assert!(!DEFAULT_ELPMPP02_PATH.is_empty());
    assert!(DEFAULT_ELPMPP02_PATH.contains("elpmpp02"));
}
