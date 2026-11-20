use super::*;

#[test]
fn region_local_samples_map_to_global_coordinates() {
    let region = RegionCoordinate::new(2, -3);
    assert_eq!(region.global_sample(0, 0), Some(MacroCoordinate::new(512, -768)));
    assert_eq!(region.global_sample(255, 255), Some(MacroCoordinate::new(767, -513)));
    assert_eq!(region.global_sample(256, 0), None);
    assert_eq!(region.global_sample(0, 256), None);
}

#[test]
fn global_coordinates_partition_correctly_on_positive_and_negative_axes() {
    let cases = [
        (MacroCoordinate::new(0, 0), RegionCoordinate::ORIGIN, 0, 0),
        (MacroCoordinate::new(255, 255), RegionCoordinate::ORIGIN, 255, 255),
        (MacroCoordinate::new(256, 0), RegionCoordinate::new(1, 0), 0, 0),
        (MacroCoordinate::new(-1, -1), RegionCoordinate::new(-1, -1), 255, 255),
        (MacroCoordinate::new(-256, -257), RegionCoordinate::new(-1, -2), 0, 255),
        (MacroCoordinate::new(-257, 256), RegionCoordinate::new(-2, 1), 255, 0),
    ];
    for (global, expected_region, expected_x, expected_y) in cases {
        assert_eq!(global.region_and_local(), (expected_region, expected_x, expected_y));
    }
}

#[test]
fn adjacent_region_chunks_share_one_global_coordinate_lattice() {
    let west = RegionCoordinate::new(0, 0);
    let east = RegionCoordinate::new(1, 0);
    assert_eq!(west.global_sample(255, 73).unwrap().x + 1, east.global_sample(0, 73).unwrap().x);
    assert_eq!(west.global_sample(255, 73).unwrap().y, east.global_sample(0, 73).unwrap().y);
    let negative = RegionCoordinate::new(-1, 0);
    assert_eq!(negative.global_sample(255, 73).unwrap().x + 1, 0);
}

#[test]
fn global_sample_conversion_rejects_coordinate_overflow() {
    let region = RegionCoordinate::new(i64::MAX, i64::MIN);
    assert_eq!(region.global_sample(0, 0), None);
}

#[test]
fn coordinate_seed_is_stable_and_independent_of_sampling_order() {
    let coordinates = [
        MacroCoordinate::new(-256, -1),
        MacroCoordinate::new(0, 0),
        MacroCoordinate::new(12, 98),
        MacroCoordinate::new(256, 255),
    ];
    let forward = coordinates.iter()
        .map(|coordinate| (*coordinate, coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, *coordinate)))
        .collect::<Vec<_>>();
    let reverse = coordinates.iter().rev()
        .map(|coordinate| (*coordinate, coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, *coordinate)))
        .collect::<Vec<_>>();
    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());
}

#[test]
fn coordinate_seed_changes_with_world_ruleset_or_global_position() {
    let coordinate = MacroCoordinate::new(-7, 19);
    let baseline = coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, coordinate);
    assert_ne!(baseline, coordinate_seed(43, GEOGRAPHY_RULESET_VERSION, coordinate));
    assert_ne!(baseline, coordinate_seed(42, GEOGRAPHY_RULESET_VERSION + 1, coordinate));
    assert_ne!(baseline, coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, MacroCoordinate::new(-6, 19)));
}
