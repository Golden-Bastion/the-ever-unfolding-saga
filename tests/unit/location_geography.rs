use crate::geography::locations::ensure_location_geography;
use crate::geography::RegionCoordinate;
use crate::model::{Location, Region, RouteDirection, World, WorldMode, WorldRoute};

fn legacy_world() -> World {
    let mut world = World::new("Legacy", WorldMode::Inherited);
    world.next_id = 20;
    world.regions = vec![
        Region {
            id: 4,
            name: "Second".into(),
            description: String::new(),
            location_ids: vec![11],
            coordinate: None,
        },
        Region {
            id: 2,
            name: "First".into(),
            description: String::new(),
            location_ids: vec![10],
            coordinate: None,
        },
    ];
    world.locations = vec![
        Location {
            id: 10,
            name: "A".into(),
            description: String::new(),
            region_id: 2,
            dangerous: false,
            corpse_ids: vec![],
            exits: vec![11],
            spatial_layout: None,
        },
        Location {
            id: 11,
            name: "B".into(),
            description: String::new(),
            region_id: 4,
            dangerous: false,
            corpse_ids: vec![],
            exits: vec![10],
            spatial_layout: None,
        },
    ];
    world.add_route(WorldRoute::new(
        "route.legacy",
        10,
        11,
        RouteDirection::Bidirectional,
    ));
    world.record_history(3, "A memory from before geography embedding.");
    world.next_id = 30;
    let mut established_map = crate::spatial::generate_local_map(
        77,
        "location.10",
        crate::spatial::LocalMapConfig::default(),
    );
    established_map.set_terrain(established_map.entry, crate::spatial::Terrain::Rubble);
    established_map.mark_tile_discovered(established_map.entry);
    if let Some(poi) = established_map.points_of_interest.first_mut() {
        poi.discovered = true;
        poi.interacted = true;
    }
    established_map.props.push(crate::spatial::SpatialProp {
        id: "prop.legacy.pillar".into(),
        kind: crate::spatial::SpatialPropKind::Pillar,
        position: crate::spatial::GridPosition { x: 3, y: 3 },
    });
    established_map.exits.push(crate::spatial::MapExit {
        id: "location.10.exit.11".into(),
        route_id: None,
        route_profile: None,
        frontier: None,
        cave_target: None,
        position: crate::spatial::GridPosition {
            x: established_map.width - 1,
            y: 2,
        },
        target_location_id: 11,
    });
    world.local_maps.push(established_map);
    world
}

#[test]
fn legacy_embedding_is_deterministic_and_preserves_world_identity() {
    let mut first = legacy_world();
    let mut second = legacy_world();
    let original_history = first.history.clone();
    let original_maps = first.local_maps.clone();
    let original_region_ids = first
        .regions
        .iter()
        .map(|region| region.id)
        .collect::<Vec<_>>();
    let original_location_ids = first
        .locations
        .iter()
        .map(|location| location.id)
        .collect::<Vec<_>>();
    ensure_location_geography(&mut first, None);
    ensure_location_geography(&mut second, None);

    assert_eq!(first.geography_seed, second.geography_seed);
    assert_eq!(
        first
            .regions
            .iter()
            .map(|region| (region.id, region.coordinate))
            .collect::<Vec<_>>(),
        second
            .regions
            .iter()
            .map(|region| (region.id, region.coordinate))
            .collect::<Vec<_>>()
    );
    assert_eq!(first.location_geography, second.location_geography);
    assert_eq!(first.next_id, 30);
    assert_eq!(first.routes.len(), 1);
    assert_eq!(first.routes[0].id, "route.legacy");
    assert_eq!(first.routes[0].location_a_id, 10);
    assert_eq!(first.routes[0].location_b_id, 11);
    assert_eq!(
        first.routes[0].direction,
        crate::model::RouteDirection::Bidirectional
    );
    assert_eq!(first.history.len(), original_history.len());
    assert_eq!(first.history[0].id, original_history[0].id);
    assert_eq!(first.history[0].turn, original_history[0].turn);
    assert_eq!(first.history[0].text, original_history[0].text);
    assert_eq!(first.local_maps, original_maps);
    assert_eq!(
        first.local_maps[0].terrain_at(first.local_maps[0].entry),
        Some(crate::spatial::Terrain::Rubble)
    );
    assert!(first.local_maps[0].is_tile_discovered(first.local_maps[0].entry));
    assert!(first.local_maps[0]
        .points_of_interest
        .iter()
        .any(|poi| poi.discovered && poi.interacted));
    assert!(first.local_maps[0]
        .props
        .iter()
        .any(|prop| prop.id == "prop.legacy.pillar"));
    assert_eq!(first.local_maps[0].exits[0].id, "location.10.exit.11");
    assert_eq!(
        first
            .regions
            .iter()
            .map(|region| region.id)
            .collect::<Vec<_>>(),
        original_region_ids
    );
    assert_eq!(
        first
            .locations
            .iter()
            .map(|location| location.id)
            .collect::<Vec<_>>(),
        original_location_ids
    );
    assert_eq!(first.geography_embedding_version, 2);
}

#[test]
fn authored_location_anchors_are_applied_once_and_then_remain_authoritative() {
    let mut world = legacy_world();
    let authored: crate::content::CampaignContent = serde_json::from_value(serde_json::json!({
        "version": 1,
        "world": {
            "region": {
                "id": "region.authored",
                "name": "Authored Region",
                "description": ""
            },
            "locations": [
                {
                    "id": "location.authored.a",
                    "name": "A",
                    "description": "",
                    "dangerous": false,
                    "geography": {
                        "anchor": { "x": 12, "y": 18, "stratum": 0 },
                        "footprint_radius_samples": 7
                    }
                }
            ]
        }
    }))
    .expect("minimal authored campaign geography should deserialize");

    ensure_location_geography(&mut world, Some(&authored));

    let record = world
        .location_geography
        .iter()
        .find(|record| record.location_id == 10)
        .expect("authored location should receive a geography record");
    assert_eq!(record.stable_key, "location.content.location.authored.a");
    assert_eq!(
        record.anchor,
        crate::geography::MacroCoordinate::new(12, 18)
    );
    assert_eq!(record.footprint_radius_samples, 7);

    let changed_content: crate::content::CampaignContent =
        serde_json::from_value(serde_json::json!({
            "version": 1,
            "world": {
                "region": {
                    "id": "region.authored",
                    "name": "Authored Region",
                    "description": ""
                },
                "locations": [
                    {
                        "id": "location.authored.a",
                        "name": "A",
                        "description": "",
                        "dangerous": false,
                        "geography": {
                            "anchor": { "x": 40, "y": 50, "stratum": 0 },
                            "footprint_radius_samples": 12
                        }
                    }
                ]
            }
        }))
        .expect("updated authored campaign geography should deserialize");

    ensure_location_geography(&mut world, Some(&changed_content));

    let record = world
        .location_geography
        .iter()
        .find(|record| record.location_id == 10)
        .expect("established authored location should retain its geography record");
    assert_eq!(record.stable_key, "location.content.location.authored.a");
    assert_eq!(
        record.anchor,
        crate::geography::MacroCoordinate::new(12, 18)
    );
    assert_eq!(record.footprint_radius_samples, 7);
}

#[test]
fn embedding_is_idempotent_and_never_moves_existing_coordinates() {
    let mut world = legacy_world();
    world.regions[0].coordinate = Some(RegionCoordinate::new(-3, 7));
    ensure_location_geography(&mut world, None);
    let coordinates = world
        .regions
        .iter()
        .map(|region| (region.id, region.coordinate))
        .collect::<Vec<_>>();
    let anchors = world.location_geography.clone();

    ensure_location_geography(&mut world, None);

    assert_eq!(
        coordinates,
        world
            .regions
            .iter()
            .map(|region| (region.id, region.coordinate))
            .collect::<Vec<_>>()
    );
    assert_eq!(anchors, world.location_geography);
    assert_eq!(
        world.regions[0].coordinate,
        Some(RegionCoordinate::new(-3, 7))
    );
}

#[test]
fn location_anchors_are_surface_coordinates_owned_by_their_regions() {
    let mut world = legacy_world();
    ensure_location_geography(&mut world, None);

    for record in &world.location_geography {
        assert_eq!(record.anchor.stratum, 0);
        let location = world.location_by_id(record.location_id).unwrap();
        let region = world
            .regions
            .iter()
            .find(|region| region.id == location.region_id)
            .unwrap();
        assert_eq!(
            record.anchor.region_and_local().0,
            region.coordinate.unwrap()
        );
        assert!(record.footprint_radius_samples <= 4);
    }
}

#[test]
fn generated_location_keys_and_anchors_do_not_depend_on_runtime_entity_ids() {
    let mut original = legacy_world();
    let mut rekeyed = legacy_world();
    let metadata = crate::model::WorldGenerationMetadata {
        seed: 77,
        region_count: 2,
        location_count: 2,
        extra_edges: 0,
        geography_ruleset_version: crate::geography::GEOGRAPHY_RULESET_VERSION,
    };
    original.generation = Some(metadata);
    rekeyed.generation = Some(metadata);

    rekeyed.locations[0].id = 100;
    rekeyed.locations[0].exits = vec![101];
    rekeyed.locations[1].id = 101;
    rekeyed.locations[1].exits = vec![100];
    rekeyed.regions[0].location_ids = vec![101];
    rekeyed.regions[1].location_ids = vec![100];
    rekeyed.routes[0] = WorldRoute::new("route.legacy", 100, 101, RouteDirection::Bidirectional);

    ensure_location_geography(&mut original, None);
    ensure_location_geography(&mut rekeyed, None);

    assert_eq!(original.geography_seed, Some(77));
    let original_anchors = original
        .location_geography
        .iter()
        .map(|record| (record.stable_key.clone(), record.anchor))
        .collect::<Vec<_>>();
    let rekeyed_anchors = rekeyed
        .location_geography
        .iter()
        .map(|record| (record.stable_key.clone(), record.anchor))
        .collect::<Vec<_>>();
    assert_eq!(original_anchors, rekeyed_anchors);
    assert!(original
        .location_geography
        .iter()
        .all(|record| record.stable_key.starts_with("location.generated.")));
}

#[test]
fn settlement_archetype_prefers_freshwater_and_flatter_ground() {
    use crate::geography::locations::{anchor_candidate_score, LocationArchetype};

    let without_water = anchor_candidate_score(
        LocationArchetype::Settlement,
        20_000,
        10_000,
        true,
        false,
        2_000,
    );
    let with_water = anchor_candidate_score(
        LocationArchetype::Settlement,
        20_000,
        10_000,
        true,
        true,
        2_000,
    );
    let rough_ground = anchor_candidate_score(
        LocationArchetype::Settlement,
        20_000,
        10_000,
        true,
        false,
        20_000,
    );

    let water_nearby_without_full_footprint = anchor_candidate_score(
        LocationArchetype::Settlement,
        20_000,
        10_000,
        false,
        true,
        2_000,
    );

    assert!(with_water > without_water);
    assert!(without_water > rough_ground);
    assert!(without_water > water_nearby_without_full_footprint);
}

#[test]
fn landmark_archetype_prefers_elevated_distinctive_ground() {
    use crate::geography::locations::{anchor_candidate_score, LocationArchetype};

    let lowland = anchor_candidate_score(
        LocationArchetype::Landmark,
        5_000,
        5_000,
        true,
        false,
        1_000,
    );
    let highland = anchor_candidate_score(
        LocationArchetype::Landmark,
        40_000,
        30_000,
        true,
        false,
        1_000,
    );

    assert!(highland > lowland);
}

#[test]
fn wilderness_archetype_keeps_broad_environmental_preferences() {
    use crate::geography::locations::{anchor_candidate_score, LocationArchetype};

    let baseline = anchor_candidate_score(
        LocationArchetype::Wilderness,
        20_000,
        10_000,
        true,
        false,
        0,
    );
    let unrelated_context = anchor_candidate_score(
        LocationArchetype::Wilderness,
        20_000,
        10_000,
        true,
        true,
        50_000,
    );

    assert_eq!(baseline, unrelated_context);
}

#[test]
fn legacy_location_content_defaults_to_wilderness_archetype() {
    let location: crate::content::LocationContent = serde_json::from_str(
        r#"{"id":"location.legacy","name":"Legacy","description":"","dangerous":false}"#,
    )
    .unwrap();

    assert_eq!(
        location.archetype.unwrap_or_default(),
        crate::geography::locations::LocationArchetype::Wilderness
    );
}
