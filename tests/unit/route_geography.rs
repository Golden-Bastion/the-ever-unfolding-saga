use crate::geography::routes::{assess_corridor, assess_world_route, MAX_CORRIDOR_SAMPLES};
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

#[test]
fn world_route_assessment_uses_persisted_location_anchors() {
    use crate::geography::locations::ensure_location_geography;
    use crate::model::{Location, Region, RouteDirection, World, WorldMode, WorldRoute};

    let mut world = World::new("Route test", WorldMode::New);
    world.generation = Some(crate::model::WorldGenerationMetadata {
        seed: 88,
        region_count: 1,
        location_count: 2,
        extra_edges: 0,
        geography_ruleset_version: GEOGRAPHY_RULESET_VERSION,
    });
    world.regions.push(Region {
        id: 2,
        name: "Test".into(),
        description: String::new(),
        location_ids: vec![3, 4],
        coordinate: Some(crate::geography::RegionCoordinate::ORIGIN),
    });
    world.locations = vec![
        Location {
            id: 3,
            name: "Start".into(),
            description: String::new(),
            region_id: 2,
            dangerous: false,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
            spatial_layout: None,
        },
        Location {
            id: 4,
            name: "End".into(),
            description: String::new(),
            region_id: 2,
            dangerous: false,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
            spatial_layout: None,
        },
    ];
    world.add_route(WorldRoute::new(
        "route.test",
        3,
        4,
        RouteDirection::Bidirectional,
    ));
    ensure_location_geography(&mut world, None);

    assert!(assess_world_route(&world, "route.test").is_some());
    assert!(assess_world_route(&world, "missing").is_none());
}
