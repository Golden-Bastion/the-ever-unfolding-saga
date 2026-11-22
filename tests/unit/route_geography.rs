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
    world.geography_seed = Some(88);
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
    assert!(world
        .route_by_id("route.test")
        .is_some_and(|route| route.geography.is_none()));
    ensure_location_geography(&mut world, None);

    assert!(assess_world_route(&world, "route.test").is_some());
    assert!(assess_world_route(&world, "missing").is_none());

    let original_plan = world
        .route_by_id("route.test")
        .and_then(|route| route.geography.clone())
        .expect("location embedding should plan established routes");
    crate::geography::locations::ensure_location_geography(&mut world, None);
    assert_eq!(
        world
            .route_by_id("route.test")
            .and_then(|route| route.geography.clone()),
        Some(original_plan)
    );

    assert!(world.add_route(WorldRoute::new(
        "route.parallel",
        3,
        4,
        RouteDirection::Bidirectional,
    )));
    assert!(world
        .route_by_id("route.parallel")
        .is_some_and(|route| route.geography.is_some()));
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
    use crate::geography::routes::{
        RouteGeography, RouteHeading, RoutePlanningStatus, RouteProfile,
    };

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

#[test]
fn water_crossings_distinguish_fords_bridges_ferries_and_impassable_water() {
    use crate::geography::rivers::RiverWidthClass;
    use crate::geography::routes::{
        infrastructure_for_water, RouteCrossingKind, RouteInfrastructure,
    };

    assert_eq!(
        infrastructure_for_water(RouteCrossingKind::River, Some(RiverWidthClass::Narrow)),
        RouteInfrastructure::Ford
    );
    assert_eq!(
        infrastructure_for_water(RouteCrossingKind::River, Some(RiverWidthClass::Medium)),
        RouteInfrastructure::Ford
    );
    assert_eq!(
        infrastructure_for_water(RouteCrossingKind::River, Some(RiverWidthClass::Wide)),
        RouteInfrastructure::Bridge
    );
    assert_eq!(
        infrastructure_for_water(RouteCrossingKind::River, Some(RiverWidthClass::VeryWide)),
        RouteInfrastructure::Bridge
    );
    assert_eq!(
        infrastructure_for_water(RouteCrossingKind::Lake, None),
        RouteInfrastructure::Ferry
    );
    assert_eq!(
        infrastructure_for_water(RouteCrossingKind::Ocean, None),
        RouteInfrastructure::Impassable
    );
    assert_eq!(
        infrastructure_for_water(RouteCrossingKind::UnresolvedWater, None),
        RouteInfrastructure::Survey
    );
}

#[test]
fn generated_route_crossings_capture_river_identity_width_and_flow() {
    use crate::geography::hydrology::{self, SurfaceWaterKind};
    use crate::geography::routes::{classify_crossings, RouteCrossingKind, RouteInfrastructure};
    use crate::geography::watershed;

    let fixture = [8_211, 42, 1_701, 7_321, 12_345, 1, 2, 3]
        .into_iter()
        .find_map(|seed| {
            for coarse_y in -48..=48 {
                for coarse_x in -48..=48 {
                    let Some(coordinate) = watershed::cell_center(coarse_x, coarse_y) else {
                        continue;
                    };
                    let Some(sample) =
                        hydrology::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                    else {
                        continue;
                    };
                    let SurfaceWaterKind::River(river) = sample.kind() else {
                        continue;
                    };
                    if river
                        .flow_direction(seed, GEOGRAPHY_RULESET_VERSION)
                        .is_none()
                        || river.network_id(seed, GEOGRAPHY_RULESET_VERSION).is_none()
                    {
                        continue;
                    }

                    // All three points remain inside the same 4,096-sample watershed cell.
                    let path = [
                        coordinate,
                        MacroCoordinate::new(coordinate.x + 1, coordinate.y),
                        MacroCoordinate::new(coordinate.x + 2, coordinate.y),
                    ];
                    let crossings = classify_crossings(seed, GEOGRAPHY_RULESET_VERSION, &path);
                    let river_crossings = crossings
                        .iter()
                        .filter(|crossing| crossing.kind == RouteCrossingKind::River)
                        .collect::<Vec<_>>();
                    if river_crossings.len() == 1 {
                        return Some((river, (*river_crossings[0]).clone(), seed));
                    }
                }
            }
            None
        })
        .expect("expected a deterministic confirmed river crossing");

    let (river, crossing, seed) = fixture;
    assert_eq!(
        crossing.river_id,
        river.network_id(seed, GEOGRAPHY_RULESET_VERSION)
    );
    assert_eq!(crossing.river_width, Some(river.width_class()));
    assert_eq!(
        crossing.river_flow_direction,
        river.flow_direction(seed, GEOGRAPHY_RULESET_VERSION)
    );
    assert_eq!(
        crossing.infrastructure,
        if river.width_class().is_fordable() {
            RouteInfrastructure::Ford
        } else {
            RouteInfrastructure::Bridge
        }
    );
    let encoded = serde_json::to_value(&crossing).unwrap();
    let decoded: crate::geography::routes::RouteCrossing = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, crossing);
}

#[test]
fn legacy_route_crossings_deserialize_without_river_metadata() {
    use crate::geography::routes::{RouteCrossing, RouteCrossingKind, RouteInfrastructure};

    let value = serde_json::json!({
        "coordinate": { "x": 32, "y": 64, "stratum": 0 },
        "kind": "river",
        "infrastructure": "ford_or_bridge"
    });
    let crossing: RouteCrossing = serde_json::from_value(value).unwrap();

    assert_eq!(crossing.kind, RouteCrossingKind::River);
    assert_eq!(crossing.infrastructure, RouteInfrastructure::FordOrBridge);
    assert_eq!(crossing.river_id, None);
    assert_eq!(crossing.river_width, None);
    assert_eq!(crossing.river_flow_direction, None);
}

#[test]
fn route_profiles_reflect_combined_terrain_crossing_requirements() {
    use crate::geography::routes::{
        classify_profile, RouteCrossing, RouteCrossingKind, RouteInfrastructure, RouteProfile,
    };

    let crossing = |kind, infrastructure| RouteCrossing {
        coordinate: MacroCoordinate::new(32, 64),
        kind,
        infrastructure,
        river_id: None,
        river_width: None,
        river_flow_direction: None,
    };
    assert_eq!(classify_profile(&[]), RouteProfile::Trail);
    assert_eq!(
        classify_profile(&[crossing(
            RouteCrossingKind::MountainPass,
            RouteInfrastructure::None,
        )]),
        RouteProfile::MountainPass
    );
    assert_eq!(
        classify_profile(&[crossing(
            RouteCrossingKind::River,
            RouteInfrastructure::FordOrBridge,
        )]),
        RouteProfile::RiverCrossing
    );
    assert_eq!(
        classify_profile(&[crossing(
            RouteCrossingKind::Lake,
            RouteInfrastructure::Ferry,
        )]),
        RouteProfile::WaterCrossing
    );
    assert_eq!(
        classify_profile(&[
            crossing(RouteCrossingKind::MountainPass, RouteInfrastructure::None),
            crossing(RouteCrossingKind::River, RouteInfrastructure::FordOrBridge),
        ]),
        RouteProfile::Mixed
    );
    assert_eq!(
        classify_profile(&[crossing(
            RouteCrossingKind::UnresolvedWater,
            RouteInfrastructure::Survey,
        )]),
        RouteProfile::Inconclusive
    );
}
