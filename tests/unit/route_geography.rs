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
fn coarse_channel_cells_do_not_create_false_local_route_crossings() {
    use crate::geography::hydrology::{self, SurfaceWaterKind};
    use crate::geography::routes::{classify_crossings, RouteCrossingKind};
    use crate::geography::watershed;

    let fixture = [8_211, 42, 1_701, 7_321, 12_345, 1, 2, 3]
        .into_iter()
        .find_map(|seed| {
            for coarse_y in -48..=48 {
                for coarse_x in -48..=48 {
                    let coordinate = watershed::cell_center(coarse_x, coarse_y)?;
                    let Some(sample) =
                        hydrology::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                    else {
                        continue;
                    };
                    let SurfaceWaterKind::River(river) = sample.kind() else {
                        continue;
                    };
                    let Some(flow) = river.flow_direction(seed, GEOGRAPHY_RULESET_VERSION) else {
                        continue;
                    };
                    let (flow_x, flow_y) = match flow {
                        crate::geography::rivers::RiverFlowDirection::North => (0, -1),
                        crate::geography::rivers::RiverFlowDirection::NorthEast => (1, -1),
                        crate::geography::rivers::RiverFlowDirection::East => (1, 0),
                        crate::geography::rivers::RiverFlowDirection::SouthEast => (1, 1),
                        crate::geography::rivers::RiverFlowDirection::South => (0, 1),
                        crate::geography::rivers::RiverFlowDirection::SouthWest => (-1, 1),
                        crate::geography::rivers::RiverFlowDirection::West => (-1, 0),
                        crate::geography::rivers::RiverFlowDirection::NorthWest => (-1, -1),
                    };
                    // Stay comfortably inside this cell but away from the
                    // bounded channel bend around its centreline.
                    let offset_x = -flow_y * 1_000;
                    let offset_y = flow_x * 1_000;
                    return Some((
                        seed,
                        MacroCoordinate::new(coordinate.x + offset_x, coordinate.y + offset_y),
                    ));
                }
            }
            None
        })
        .expect("expected a deterministic confirmed river channel");

    let (seed, coordinate) = fixture;
    // This short road lies inside a channel-bearing watershed cell but well
    // away from its locally derived channel. Coarse membership alone is not a
    // bank intersection and must not create a bridge/ford requirement.
    let path = [
        MacroCoordinate::new(coordinate.x + 100, coordinate.y + 300),
        MacroCoordinate::new(coordinate.x + 101, coordinate.y + 300),
    ];
    let crossings = classify_crossings(seed, GEOGRAPHY_RULESET_VERSION, &path);
    assert!(!crossings
        .iter()
        .any(|crossing| crossing.kind == RouteCrossingKind::River));
}

#[test]
fn route_crossing_classifier_records_generated_bank_intersections() {
    use crate::geography::hydrology::{self, SurfaceWaterKind};
    use crate::geography::river_geometry::{
        RiverGeometryCache, RiverPosition, RIVER_POSITION_SCALE,
    };
    use crate::geography::routes::{classify_crossings, RouteCrossingKind};
    use crate::geography::terrain;
    use crate::geography::watershed;

    let mut fixture = None;
    'search: for seed in [8_211, 42, 1_701, 7_321, 12_345, 1, 2, 3] {
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
                if !matches!(sample.kind(), SurfaceWaterKind::River(_)) {
                    continue;
                }

                let mut geometry = RiverGeometryCache::default();
                let Some(position) = RiverPosition::from_macro_coordinate(coordinate) else {
                    continue;
                };
                for segment in geometry.segments_near(seed, GEOGRAPHY_RULESET_VERSION, position) {
                    if segment.width_class().is_fordable() {
                        continue;
                    }

                    // Test the middle of a tessellated chord, not a shared curve
                    // vertex. This keeps the fixture independent of how adjacent
                    // chords approximate their bank joins.
                    for pair in segment.centerline().windows(2) {
                        let Some(center) = pair[0].midpoint(pair[1]) else {
                            continue;
                        };
                        let dx = pair[1].x_q16 - pair[0].x_q16;
                        let dy = pair[1].y_q16 - pair[0].y_q16;
                        let normal_scale = dx.unsigned_abs().max(dy.unsigned_abs());
                        if normal_scale == 0 {
                            continue;
                        }

                        // A perpendicular span of 16 macro samples is wider
                        // than every generated channel band, with fixed-point
                        // arithmetic retaining the curve's local tangent.
                        let span_q16 = i128::from(RIVER_POSITION_SCALE) * 16;
                        let normal_scale = i128::from(normal_scale);
                        let Some(offset_x) =
                            i64::try_from(-i128::from(dy) * span_q16 / normal_scale).ok()
                        else {
                            continue;
                        };
                        let Some(offset_y) =
                            i64::try_from(i128::from(dx) * span_q16 / normal_scale).ok()
                        else {
                            continue;
                        };
                        let Some(start_position) = center.offset(-offset_x, -offset_y) else {
                            continue;
                        };
                        let Some(end_position) = center.offset(offset_x, offset_y) else {
                            continue;
                        };
                        let (Some(start), Some(end)) = (
                            start_position.rounded_macro_coordinate(),
                            end_position.rounded_macro_coordinate(),
                        ) else {
                            continue;
                        };
                        let (Some(route_start), Some(route_end)) = (
                            RiverPosition::from_macro_coordinate(start),
                            RiverPosition::from_macro_coordinate(end),
                        ) else {
                            continue;
                        };

                        // Reject a quantized fixture unless the selected channel
                        // itself confirms that this route crosses both banks.
                        if segment.intersect_route(route_start, route_end).is_none() {
                            continue;
                        }
                        if ![start, end].into_iter().all(|point| {
                            terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, point)
                                .is_some_and(|terrain| terrain.is_land())
                        }) {
                            continue;
                        }

                        let crossings =
                            classify_crossings(seed, GEOGRAPHY_RULESET_VERSION, &[start, end]);
                        if let Some(crossing) = crossings.into_iter().find(|crossing| {
                            crossing.kind == RouteCrossingKind::River
                                && crossing.river_id == Some(segment.river_id())
                                && crossing.river_banks.is_some()
                        }) {
                            fixture = Some(crossing);
                            break 'search;
                        }
                    }
                }
            }
        }
    }

    let crossing = fixture.expect("expected a planned path to cross both generated river banks");
    let banks = crossing
        .river_banks
        .expect("confirmed crossings retain both banks");
    assert_ne!(banks.entry, banks.exit);
    assert_eq!(crossing.kind, RouteCrossingKind::River);
}

#[test]
fn exact_river_bank_metadata_roundtrips_and_old_records_default_empty() {
    use crate::geography::river_geometry::{
        RiverBankIntersection, RiverPosition, RIVER_POSITION_SCALE,
    };
    use crate::geography::rivers::{RiverFlowDirection, RiverId, RiverOutletKind, RiverWidthClass};
    use crate::geography::routes::{RouteCrossing, RouteCrossingKind, RouteInfrastructure};

    let crossing = RouteCrossing {
        coordinate: MacroCoordinate::new(8, 12),
        kind: RouteCrossingKind::River,
        infrastructure: RouteInfrastructure::Bridge,
        river_id: Some(RiverId {
            outlet_coordinate: MacroCoordinate::new(128, 256),
            outlet_kind: RiverOutletKind::Ocean,
        }),
        river_width: Some(RiverWidthClass::Wide),
        river_flow_direction: Some(RiverFlowDirection::SouthEast),
        river_banks: Some(RiverBankIntersection {
            entry: RiverPosition {
                x_q16: 8 * RIVER_POSITION_SCALE,
                y_q16: 11 * RIVER_POSITION_SCALE + RIVER_POSITION_SCALE / 4,
            },
            exit: RiverPosition {
                x_q16: 8 * RIVER_POSITION_SCALE,
                y_q16: 12 * RIVER_POSITION_SCALE + RIVER_POSITION_SCALE / 4,
            },
        }),
    };
    let encoded = serde_json::to_value(&crossing).unwrap();
    let decoded: crate::geography::routes::RouteCrossing = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, crossing);

    let legacy = serde_json::json!({
        "coordinate": { "x": 32, "y": 64, "stratum": 0 },
        "kind": "river",
        "infrastructure": "ford_or_bridge"
    });
    let legacy: RouteCrossing = serde_json::from_value(legacy).unwrap();
    assert_eq!(legacy.river_banks, None);
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
    assert_eq!(crossing.river_banks, None);
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
        river_banks: None,
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
