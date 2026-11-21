use crate::geography::routes::{assess_corridor, MAX_CORRIDOR_SAMPLES};
use crate::geography::{MacroCoordinate, GEOGRAPHY_RULESET_VERSION};

#[test]
fn corridor_assessment_is_deterministic_and_bounded() {
    let start = MacroCoordinate::new(-300, 1_200);
    let end = MacroCoordinate::new(2_400, -900);
    let first = assess_corridor(8_812, GEOGRAPHY_RULESET_VERSION, start, end).unwrap();
    let second = assess_corridor(8_812, GEOGRAPHY_RULESET_VERSION, start, end).unwrap();

    assert_eq!(first, second);
    assert!(usize::from(first.sampled_points) <= MAX_CORRIDOR_SAMPLES);
    assert!(first.sampled_points > 0);
    assert!(first.undersampled);
    assert!(
        first.ocean_samples
            + first.lake_samples
            + first.river_samples
            + first.unresolved_water_samples
            <= first.sampled_points
    );
}

#[test]
fn corridor_rejects_underground_endpoints() {
    assert!(assess_corridor(
        42,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
        MacroCoordinate::new(100, 100),
    )
    .is_none());
}
