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

#[test]
fn terrain_aware_route_planning_is_deterministic_and_stays_on_land() {
    use crate::geography::routes::{
        plan_route, RoutePlanningStatus, MAX_ROUTE_PATH_POINTS, ROUTE_GRID_SPACING,
    };
    use crate::geography::terrain;

    let seed = 9_317;
    let mut endpoints = None;
    'search: for y in (-2_048..=2_048).step_by(ROUTE_GRID_SPACING as usize) {
        for x in (-2_048..=2_048).step_by(ROUTE_GRID_SPACING as usize) {
            let start = MacroCoordinate::new(x, y);
            let end = MacroCoordinate::new(x + ROUTE_GRID_SPACING, y);
            let start_land = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, start)
                .is_some_and(|sample| sample.is_land());
            let end_land = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, end)
                .is_some_and(|sample| sample.is_land());
            if start_land && end_land {
                endpoints = Some((start, end));
                break 'search;
            }
        }
    }
    let (start, end) = endpoints.expect("test seed should contain adjacent land samples");

    let first = plan_route(seed, GEOGRAPHY_RULESET_VERSION, start, end);
    let second = plan_route(seed, GEOGRAPHY_RULESET_VERSION, start, end);
    assert_eq!(first, second);
    assert_eq!(first.status, RoutePlanningStatus::Planned);
    assert_eq!(first.path.first(), Some(&start));
    assert_eq!(first.path.last(), Some(&end));
    assert!(first.path.len() <= MAX_ROUTE_PATH_POINTS);
    assert!(first.path.iter().all(|coordinate| {
        terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate)
            .is_some_and(|sample| sample.is_land())
    }));
}

#[test]
fn route_geography_heading_is_reversed_at_the_destination() {
    use crate::geography::routes::{RouteGeography, RouteHeading, RoutePlanningStatus, RouteProfile};

    let route = RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::Trail,
        path: vec![
            MacroCoordinate::new(0, 0),
            MacroCoordinate::new(32, 0),
            MacroCoordinate::new(64, 0),
        ],
        crossings: Vec::new(),
        total_cost: 200,
    };
    assert_eq!(route.heading_from(true), Some(RouteHeading::East));
    assert_eq!(route.heading_from(false), Some(RouteHeading::West));
}

#[test]
fn legacy_world_routes_deserialize_without_geography_metadata() {
    let route = crate::model::WorldRoute::new(
        "route.legacy",
        10,
        20,
        crate::model::RouteDirection::Bidirectional,
    );
    let mut value = serde_json::to_value(route).unwrap();
    value.as_object_mut().unwrap().remove("geography");

    let loaded: crate::model::WorldRoute = serde_json::from_value(value).unwrap();
    assert!(loaded.geography.is_none());
}
