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
    world
}

#[test]
fn legacy_embedding_is_deterministic_and_preserves_world_identity() {
    let mut first = legacy_world();
    let mut second = legacy_world();
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
    assert_eq!(first.next_id, 20);
    assert_eq!(first.routes.len(), 1);
    assert!(first.history.is_empty());
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
fn generated_location_keys_are_independent_of_runtime_entity_ids() {
    let mut world = legacy_world();
    world.generation = Some(crate::model::WorldGenerationMetadata {
        seed: 77,
        region_count: 2,
        location_count: 2,
        extra_edges: 0,
        geography_ruleset_version: crate::geography::GEOGRAPHY_RULESET_VERSION,
    });
    ensure_location_geography(&mut world, None);

    assert_eq!(world.geography_seed, Some(77));
    assert!(world
        .location_geography
        .iter()
        .all(|record| record.stable_key.starts_with("location.generated.")));
}
