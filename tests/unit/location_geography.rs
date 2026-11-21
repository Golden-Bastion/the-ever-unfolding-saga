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
    world.local_maps.push(crate::spatial::generate_local_map(
        77,
        "location.10",
        crate::spatial::LocalMapConfig::default(),
    ));
    world
}

#[test]
fn legacy_embedding_is_deterministic_and_preserves_world_identity() {
    let mut first = legacy_world();
    let mut second = legacy_world();
    let original_history = first.history.clone();
    let original_maps = first.local_maps.clone();
    let original_region_ids = first.regions.iter().map(|region| region.id).collect::<Vec<_>>();
    let original_location_ids = first.locations.iter().map(|location| location.id).collect::<Vec<_>>();
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
    assert_eq!(first.history.len(), original_history.len());
    assert_eq!(first.history[0].id, original_history[0].id);
    assert_eq!(first.history[0].turn, original_history[0].turn);
    assert_eq!(first.history[0].text, original_history[0].text);
    assert_eq!(first.local_maps, original_maps);
    assert_eq!(first.regions.iter().map(|region| region.id).collect::<Vec<_>>(), original_region_ids);
    assert_eq!(first.locations.iter().map(|location| location.id).collect::<Vec<_>>(), original_location_ids);
    assert_eq!(first.geography_embedding_version, 1);
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
        assert_eq!(record.anchor.region_and_local().0, region.coordinate.unwrap());
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
    rekeyed.routes[0] = WorldRoute::new(
        "route.legacy",
        100,
        101,
        RouteDirection::Bidirectional,
    );

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
