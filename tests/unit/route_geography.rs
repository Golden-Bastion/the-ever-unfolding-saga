use crate::geography::routes::{
    assess_corridor, assess_world_route, MAX_CORRIDOR_SAMPLES, ROUTE_GRID_SPACING,
};
use crate::geography::{MacroCoordinate, GEOGRAPHY_RULESET_VERSION};

fn find_lake_route_fixture() -> Option<(u64, String, Vec<MacroCoordinate>)> {
    use crate::geography::{terrain, waterbodies, watershed};

    for seed in 1..=24 {
        for coarse_y in (-40..=-1).rev() {
            for coarse_x in (-40..=-1).rev() {
                let Some(center) = watershed::cell_center(coarse_x, coarse_y) else {
                    continue;
                };
                let Some(terrain_sample) =
                    terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, center)
                else {
                    continue;
                };
                if !terrain_sample.is_land()
                    || watershed::downstream_cell(
                        seed,
                        GEOGRAPHY_RULESET_VERSION,
                        coarse_x,
                        coarse_y,
                        terrain_sample.elevation_q16(),
                    )
                    .is_some()
                {
                    continue;
                }
                let Some(lake) =
                    waterbodies::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, center)
                else {
                    continue;
                };
                let lake_id = lake.id(seed, GEOGRAPHY_RULESET_VERSION);

                for (dx, dy) in [(1_i64, 0_i64), (0, 1), (1, 1), (1, -1)] {
                    let find_dry_bank = |direction: i64| {
                        (1_i64..=128).find_map(|steps| {
                            let distance = steps * ROUTE_GRID_SPACING;
                            let coordinate = MacroCoordinate::new(
                                center.x.checked_add(dx * distance * direction)?,
                                center.y.checked_add(dy * distance * direction)?,
                            );
                            let is_dry_land = terrain::sample_surface(
                                seed,
                                GEOGRAPHY_RULESET_VERSION,
                                coordinate,
                            )
                            .is_some_and(|sample| sample.is_land())
                                && waterbodies::sample_surface(
                                    seed,
                                    GEOGRAPHY_RULESET_VERSION,
                                    coordinate,
                                )
                                .is_none();
                            is_dry_land.then_some((steps, coordinate))
                        })
                    };
                    let Some((before_steps, start)) = find_dry_bank(-1) else {
                        continue;
                    };
                    let Some((after_steps, _end)) = find_dry_bank(1) else {
                        continue;
                    };
                    let total_steps = before_steps + after_steps;
                    if total_steps + 1 > crate::geography::routes::MAX_ROUTE_PATH_POINTS as i64 {
                        continue;
                    }
                    let path = (0..=total_steps)
                        .map(|step| {
                            MacroCoordinate::new(
                                start.x + dx * step * ROUTE_GRID_SPACING,
                                start.y + dy * step * ROUTE_GRID_SPACING,
                            )
                        })
                        .collect::<Vec<_>>();
                    let lake_points = path
                        .iter()
                        .enumerate()
                        .filter_map(|(index, coordinate)| {
                            waterbodies::sample_surface(
                                seed,
                                GEOGRAPHY_RULESET_VERSION,
                                *coordinate,
                            )
                            .is_some_and(|sample| {
                                sample.id(seed, GEOGRAPHY_RULESET_VERSION) == lake_id
                            })
                            .then_some(index)
                        })
                        .collect::<Vec<_>>();
                    let Some(first_lake_point) = lake_points.first().copied() else {
                        continue;
                    };
                    let Some(last_lake_point) = lake_points.last().copied() else {
                        continue;
                    };
                    if first_lake_point == 0 || last_lake_point + 1 >= path.len() {
                        continue;
                    }

                    let mut river_cache =
                        crate::geography::river_geometry::RiverGeometryCache::default();
                    let shores = [path[first_lake_point - 1], path[last_lake_point + 1]];
                    let has_two_dry_banks = shores.into_iter().all(|shore| {
                        if !terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, shore)
                            .is_some_and(|sample| sample.is_land())
                            || waterbodies::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, shore)
                                .is_some()
                        {
                            return false;
                        }
                        let Some(position) =
                            crate::geography::river_geometry::RiverPosition::from_macro_coordinate(
                                shore,
                            )
                        else {
                            return false;
                        };
                        !river_cache
                            .segments_near(seed, GEOGRAPHY_RULESET_VERSION, position)
                            .iter()
                            .any(|segment| segment.contains_position(position))
                    });
                    if has_two_dry_banks {
                        return Some((seed, lake_id, path));
                    }
                }
            }
        }
    }
    None
}

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
    use crate::geography::routes::{classify_crossings, RouteCrossingKind, RouteInfrastructure};
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
                    // Narrow and medium rivers still have exact banks; their
                    // infrastructure is a ford rather than a bridge. Exercise
                    // every generated width class instead of requiring a
                    // large river to appear in this bounded deterministic search.

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
                            fixture = Some((crossing, segment.width_class()));
                            break 'search;
                        }
                    }
                }
            }
        }
    }

    let (crossing, width) =
        fixture.expect("expected a planned path to cross both generated river banks");
    let banks = crossing
        .river_banks
        .expect("confirmed crossings retain both banks");
    assert_ne!(banks.entry, banks.exit);
    assert_eq!(crossing.kind, RouteCrossingKind::River);
    assert_eq!(crossing.river_width, Some(width));
    assert_eq!(
        crossing.infrastructure,
        if width.is_fordable() {
            RouteInfrastructure::Ford
        } else {
            RouteInfrastructure::Bridge
        }
    );
}

#[test]
fn lake_route_crossings_record_stable_identity_and_both_shores() {
    use crate::geography::routes::{
        classify_crossings, LakeBankIntersection, RouteCrossingKind, RouteInfrastructure,
    };

    let (seed, expected_lake_id, path) =
        find_lake_route_fixture().expect("expected a deterministic lake with two dry shores");
    let crossings = classify_crossings(seed, GEOGRAPHY_RULESET_VERSION, &path);
    let repeated = classify_crossings(seed, GEOGRAPHY_RULESET_VERSION, &path);
    assert_eq!(crossings, repeated);

    let crossing = crossings
        .iter()
        .find(|crossing| {
            crossing.kind == RouteCrossingKind::Lake
                && crossing.lake_id.as_deref() == Some(expected_lake_id.as_str())
        })
        .expect("the route should retain its crossed lake identity");
    assert_eq!(crossing.infrastructure, RouteInfrastructure::Ferry);
    let banks: LakeBankIntersection = crossing
        .lake_banks
        .expect("a land-to-land lake crossing should retain both exact shores");
    assert_ne!(banks.entry, banks.exit);
    assert_eq!(
        crossing.coordinate,
        banks
            .midpoint()
            .and_then(|position| position.rounded_macro_coordinate())
            .expect("the crossing midpoint should be representable")
    );
}

#[test]
fn resolved_lake_crossings_follow_travel_direction_and_require_both_shores() {
    use crate::geography::river_geometry::{RiverPosition, RIVER_POSITION_SCALE};
    use crate::geography::routes::{
        LakeBankIntersection, RouteCrossing, RouteCrossingKind, RouteGeography,
        RouteInfrastructure, RoutePlanningStatus, RouteProfile,
    };

    let crossing = |lake_id: &str, x: i64| RouteCrossing {
        coordinate: MacroCoordinate::new(x, 0),
        kind: RouteCrossingKind::Lake,
        infrastructure: RouteInfrastructure::Ferry,
        river_id: None,
        river_width: None,
        river_flow_direction: None,
        river_banks: None,
        lake_id: Some(lake_id.to_string()),
        lake_banks: Some(LakeBankIntersection {
            entry: RiverPosition {
                x_q16: x * RIVER_POSITION_SCALE,
                y_q16: 0,
            },
            exit: RiverPosition {
                x_q16: x * RIVER_POSITION_SCALE,
                y_q16: RIVER_POSITION_SCALE,
            },
        }),
    };
    let unresolved = RouteCrossing {
        coordinate: MacroCoordinate::new(30, 0),
        kind: RouteCrossingKind::Lake,
        infrastructure: RouteInfrastructure::Ferry,
        river_id: None,
        river_width: None,
        river_flow_direction: None,
        river_banks: None,
        lake_id: Some("lake.unresolved".into()),
        lake_banks: None,
    };
    let geography = RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::WaterCrossing,
        path: Vec::new(),
        crossings: vec![
            crossing("lake.first", 10),
            unresolved,
            crossing("lake.second", 20),
        ],
        total_cost: 0,
    };

    let forward = geography
        .resolved_lake_crossings(true)
        .into_iter()
        .map(|crossing| crossing.lake_id.as_deref().unwrap())
        .collect::<Vec<_>>();
    let reverse = geography
        .resolved_lake_crossings(false)
        .into_iter()
        .map(|crossing| crossing.lake_id.as_deref().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(forward, vec!["lake.first", "lake.second"]);
    assert_eq!(reverse, vec!["lake.second", "lake.first"]);
    assert!(geography.has_resolved_lake_crossing());
}

#[test]
fn lake_bank_metadata_roundtrips_and_legacy_records_default_empty() {
    use crate::geography::river_geometry::{RiverPosition, RIVER_POSITION_SCALE};
    use crate::geography::routes::{
        LakeBankIntersection, RouteCrossing, RouteCrossingKind, RouteInfrastructure,
    };

    let crossing = RouteCrossing {
        coordinate: MacroCoordinate::new(128, 256),
        kind: RouteCrossingKind::Lake,
        infrastructure: RouteInfrastructure::Ferry,
        river_id: None,
        river_width: None,
        river_flow_direction: None,
        river_banks: None,
        lake_id: Some("lake.128.256.stable".into()),
        lake_banks: Some(LakeBankIntersection {
            entry: RiverPosition {
                x_q16: 128 * RIVER_POSITION_SCALE,
                y_q16: 255 * RIVER_POSITION_SCALE + RIVER_POSITION_SCALE / 2,
            },
            exit: RiverPosition {
                x_q16: 128 * RIVER_POSITION_SCALE,
                y_q16: 257 * RIVER_POSITION_SCALE + RIVER_POSITION_SCALE / 2,
            },
        }),
    };
    let encoded = serde_json::to_value(&crossing).unwrap();
    let decoded: RouteCrossing = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, crossing);

    let legacy = serde_json::json!({
        "coordinate": { "x": 32, "y": 64, "stratum": 0 },
        "kind": "lake",
        "infrastructure": "ferry"
    });
    let decoded: RouteCrossing = serde_json::from_value(legacy).unwrap();
    assert_eq!(decoded.lake_id, None);
    assert_eq!(decoded.lake_banks, None);
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
        lake_id: None,
        lake_banks: None,
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
    assert_eq!(legacy.lake_id, None);
    assert_eq!(legacy.lake_banks, None);
}
#[test]
fn resolved_travel_crossings_keep_route_order_and_require_verified_banks() {
    use crate::geography::river_geometry::{
        RiverBankIntersection, RiverPosition, RIVER_POSITION_SCALE,
    };
    use crate::geography::routes::{
        LakeBankIntersection, RouteCrossing, RouteCrossingKind, RouteGeography,
        RouteInfrastructure, RoutePlanningStatus, RouteProfile,
    };

    let pos = |x, y| RiverPosition {
        x_q16: x * RIVER_POSITION_SCALE,
        y_q16: y * RIVER_POSITION_SCALE,
    };
    let river = RouteCrossing {
        coordinate: MacroCoordinate::new(4, 8),
        kind: RouteCrossingKind::River,
        infrastructure: RouteInfrastructure::Bridge,
        river_id: None,
        river_width: None,
        river_flow_direction: None,
        river_banks: Some(RiverBankIntersection {
            entry: pos(4, 8),
            exit: pos(5, 8),
        }),
        lake_id: None,
        lake_banks: None,
    };
    let lake = RouteCrossing {
        coordinate: MacroCoordinate::new(12, 8),
        kind: RouteCrossingKind::Lake,
        infrastructure: RouteInfrastructure::Ferry,
        river_id: None,
        river_width: None,
        river_flow_direction: None,
        river_banks: None,
        lake_id: Some("lake.route.fixture".into()),
        lake_banks: Some(LakeBankIntersection {
            entry: pos(12, 8),
            exit: pos(13, 8),
        }),
    };
    let unresolved_river = RouteCrossing {
        river_banks: None,
        ..river.clone()
    };
    let geography = RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::Mixed,
        path: Vec::new(),
        crossings: vec![river, lake],
        total_cost: 0,
    };

    assert_eq!(geography.travel_block_reason(), None);
    assert_eq!(
        geography
            .resolved_travel_crossings(true)
            .iter()
            .map(|crossing| crossing.kind)
            .collect::<Vec<_>>(),
        vec![RouteCrossingKind::River, RouteCrossingKind::Lake]
    );
    assert_eq!(
        geography
            .resolved_travel_crossings(false)
            .iter()
            .map(|crossing| crossing.kind)
            .collect::<Vec<_>>(),
        vec![RouteCrossingKind::Lake, RouteCrossingKind::River]
    );

    let unsupported = [
        RouteCrossing {
            kind: RouteCrossingKind::Ocean,
            infrastructure: RouteInfrastructure::Impassable,
            ..geography.crossings[0].clone()
        },
        RouteCrossing {
            kind: RouteCrossingKind::UnresolvedWater,
            infrastructure: RouteInfrastructure::Survey,
            ..geography.crossings[0].clone()
        },
        RouteCrossing {
            kind: RouteCrossingKind::Lake,
            infrastructure: RouteInfrastructure::Ferry,
            lake_id: Some("lake.without.shores".into()),
            lake_banks: None,
            ..geography.crossings[0].clone()
        },
        unresolved_river,
    ];
    for crossing in unsupported {
        let blocked = RouteGeography {
            crossings: vec![crossing],
            ..geography.clone()
        };
        assert!(blocked.travel_block_reason().is_some());
    }
}

#[test]
fn legacy_ford_or_bridge_crossings_remain_traversable() {
    use crate::geography::routes::{
        RouteCrossing, RouteCrossingKind, RouteGeography, RouteInfrastructure, RoutePlanningStatus,
        RouteProfile,
    };

    let geography = RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::RiverCrossing,
        path: Vec::new(),
        crossings: vec![RouteCrossing {
            coordinate: MacroCoordinate::new(1, 2),
            kind: RouteCrossingKind::River,
            infrastructure: RouteInfrastructure::FordOrBridge,
            river_id: None,
            river_width: None,
            river_flow_direction: None,
            river_banks: None,
            lake_id: None,
            lake_banks: None,
        }],
        total_cost: 0,
    };
    assert_eq!(geography.travel_block_reason(), None);
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
    assert_eq!(crossing.lake_id, None);
    assert_eq!(crossing.lake_banks, None);
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
        lake_id: None,
        lake_banks: None,
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
