use super::*;

use crate::geography::{climate, MacroCoordinate, GEOGRAPHY_RULESET_VERSION};

#[test]
fn location_projection_is_deterministic_and_keeps_macro_and_tile_resolutions_separate() {
    let seed = 41;
    let anchor = MacroCoordinate::new(1_237, -842);
    let first = LocalSurfaceMap::sample_location(
        seed,
        GEOGRAPHY_RULESET_VERSION,
        anchor,
        4,
        64,
        48,
    )
    .unwrap();
    let second = LocalSurfaceMap::sample_location(
        seed,
        GEOGRAPHY_RULESET_VERSION,
        anchor,
        4,
        64,
        48,
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.dimensions(), (64, 48));
    assert_eq!(first.sample_dimensions(), (9, 7));

    let center = first.sample_at_tile(32, 24).unwrap();
    assert_eq!(
        center.biome(),
        climate::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, anchor)
            .unwrap()
            .biome()
    );
    assert!(first.sample_at_tile(64, 24).is_none());
}

#[test]
fn macro_sample_axis_is_evenly_spaced_and_bounded() {
    let samples = sample_axis(-17, 4).unwrap();
    assert_eq!(samples.len(), 9);
    assert_eq!(samples.first(), Some(&-21));
    assert_eq!(samples.get(4), Some(&-17));
    assert_eq!(samples.last(), Some(&-13));

    let capped = sample_axis(10, 100).unwrap();
    assert_eq!(capped.len(), 17);
    assert_eq!(capped.first(), Some(&-90));
    assert_eq!(capped.last(), Some(&110));
}

#[test]
fn projection_rejects_underground_and_invalid_local_dimensions() {
    assert!(LocalSurfaceMap::sample_location(
        1,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
        4,
        64,
        48,
    )
    .is_none());
    assert!(LocalSurfaceMap::sample_location(
        1,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::ORIGIN,
        4,
        0,
        48,
    )
    .is_none());
    assert!(LocalSurfaceMap::sample_location(
        1,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::ORIGIN,
        4,
        129,
        48,
    )
    .is_none());
}

#[test]
fn sample_axis_rejects_coordinate_overflow() {
    assert!(sample_axis(i64::MAX, 1).is_none());
    assert!(sample_axis(i64::MIN, 1).is_none());
}
