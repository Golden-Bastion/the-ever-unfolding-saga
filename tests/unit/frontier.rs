use crate::geography::frontier::{
    frontier_targets_for_location, materialize_frontier, FrontierDirection,
    FrontierMaterializationError,
};
use crate::geography::locations::frontier_anchor_candidates;
use crate::geography::routes::{plan_route, RoutePlanningStatus};
use crate::geography::{
    terrain, MacroCoordinate, RegionCoordinate, GEOGRAPHY_RULESET_VERSION,
};
use crate::model::{
    Location, LocationGeographyRecord, Region, World, WorldMode,
};

fn world_with_source(seed: u64, next_id: u64, source_y: i64) -> World {
    let mut world = World::new("Frontier Test", WorldMode::New);
    world.next_id = next_id;
    world.geography_seed = Some(seed);
    world.geography_ruleset_version = GEOGRAPHY_RULESET_VERSION;
    world.regions.push(Region {
        id: 2,
        name: "Origin".into(),
        description: String::new(),
        location_ids: vec![3],
        coordinate: Some(RegionCoordinate::ORIGIN),
    });
    world.locations.push(Location {
        id: 3,
        name: "Starting Place".into(),
        description: String::new(),
        region_id: 2,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: Vec::new(),
        spatial_layout: None,
    });
    world.location_geography.push(LocationGeographyRecord {
        location_id: 3,
        stable_key: "location.content.start".into(),
        anchor: MacroCoordinate::new(248, source_y),
        footprint_radius_samples: 0,
    });
    world
}

fn land_connection_fixture() -> (World, crate::geography::frontier::FrontierTarget, MacroCoordinate) {
    for seed in 1..=32_u64 {
        for source_y in (0..256_i64).step_by(8) {
            let source = MacroCoordinate::new(248, source_y);
            if !terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, source)
                .is_some_and(|sample| sample.is_land())
            {
                continue;
            }
            let world = world_with_source(seed, 20, source_y);
            let frontier = frontier_targets_for_location(&world, 3)
                .into_iter()
                .find(|frontier| frontier.direction == FrontierDirection::East)
                .expect("east frontier should be available");
            let candidates = frontier_anchor_candidates(
                seed,
                GEOGRAPHY_RULESET_VERSION,
                frontier.target_region,
                &frontier.location_key(),
                frontier.direction,
                16,
            );
            if let Some(anchor) = candidates.into_iter().find(|anchor| {
                plan_route(seed, GEOGRAPHY_RULESET_VERSION, source, *anchor).status
                    == RoutePlanningStatus::Planned
            }) {
                return (world, frontier, anchor);
            }
        }
    }
    panic!("expected at least one deterministic adjacent land frontier");
}

#[test]
fn frontier_listing_is_deterministic_and_does_not_materialize_world_state() {
    let (mut world, _, _) = land_connection_fixture();

    let original_regions = world.regions.len();
    let original_locations = world.locations.len();
    let original_next_id = world.next_id;
    let first = frontier_targets_for_location(&world, 3);
    let second = frontier_targets_for_location(&world, 3);

    assert_eq!(first, second);
    assert!(first.iter().any(|frontier| frontier.direction == FrontierDirection::East));
    assert!(first.iter().all(|frontier| {
        frontier.direction != FrontierDirection::East
            || frontier.direction.adjacent_region(RegionCoordinate::ORIGIN)
                == Some(frontier.target_region)
    }));
    assert_eq!(world.regions.len(), original_regions);
    assert_eq!(world.locations.len(), original_locations);
    assert_eq!(world.next_id, original_next_id);
}

#[test]
fn materializing_a_frontier_creates_one_adjacent_region_and_is_idempotent() {
    let (mut world, frontier, _) = land_connection_fixture();
    let original_regions = world.regions.len();
    let original_locations = world.locations.len();
    let original_next_id = world.next_id;

    let first = materialize_frontier(&mut world, 3, &frontier)
        .expect("selected frontier should have a land route");
    assert!(first.created_region);
    assert!(first.created_location);
    assert_eq!(world.regions.len(), original_regions + 1);
    assert_eq!(world.locations.len(), original_locations + 1);
    assert_eq!(world.next_id, original_next_id + 2);

    let destination = world
        .location_by_id(first.location_id)
        .expect("frontier destination should exist");
    let destination_region = world
        .regions
        .iter()
        .find(|region| region.id == destination.region_id)
        .expect("destination region should exist");
    assert_eq!(destination_region.coordinate, Some(frontier.target_region));
    assert_eq!(destination_region.location_ids, vec![first.location_id]);

    let route = world
        .route_by_id(&first.route_id)
        .expect("frontier route should be persisted");
    assert!(route.connects(3, first.location_id));
    assert_eq!(
        route.geography.as_ref().map(|geography| geography.status),
        Some(RoutePlanningStatus::Planned)
    );

    let repeated = materialize_frontier(&mut world, 3, &frontier)
        .expect("revisiting the same frontier should reuse its state");
    assert_eq!(repeated.region_id, first.region_id);
    assert_eq!(repeated.location_id, first.location_id);
    assert_eq!(repeated.route_id, first.route_id);
    assert!(!repeated.created_region);
    assert!(!repeated.created_location);
    assert_eq!(world.regions.len(), original_regions + 1);
    assert_eq!(world.locations.len(), original_locations + 1);
    assert_eq!(world.next_id, original_next_id + 2);

    let remaining = frontier_targets_for_location(&world, 3);
    assert!(!remaining.iter().any(|candidate| candidate.id == frontier.id));
}

#[test]
fn frontier_identity_and_geography_do_not_depend_on_runtime_allocator_position() {
    let (first_world, frontier, expected_anchor) = land_connection_fixture();
    let source_y = first_world.location_geography[0].anchor.y;
    let seed = first_world.geography_seed.unwrap();
    let mut first = world_with_source(seed, 20, source_y);
    let mut second = world_with_source(seed, 200, source_y);

    let first_target = frontier_targets_for_location(&first, 3)
        .into_iter()
        .find(|candidate| candidate.direction == FrontierDirection::East)
        .unwrap();
    let second_target = frontier_targets_for_location(&second, 3)
        .into_iter()
        .find(|candidate| candidate.direction == FrontierDirection::East)
        .unwrap();
    assert_eq!(first_target, second_target);
    assert_eq!(first_target, frontier);

    let first_result = materialize_frontier(&mut first, 3, &first_target).unwrap();
    let second_result = materialize_frontier(&mut second, 3, &second_target).unwrap();
    assert_eq!(first_result.route_id, second_result.route_id);
    assert_eq!(first_result.region_id, 20);
    assert_eq!(second_result.region_id, 200);
    assert_ne!(first_result.location_id, second_result.location_id);
    assert_eq!(
        first.location_geography.iter().find(|record| record.location_id == first_result.location_id).unwrap().anchor,
        expected_anchor
    );
    assert_eq!(
        first.location_geography.iter().find(|record| record.location_id == first_result.location_id).unwrap().anchor,
        second.location_geography.iter().find(|record| record.location_id == second_result.location_id).unwrap().anchor
    );
    assert_eq!(
        first.route_by_id(&first_result.route_id).unwrap().geography,
        second.route_by_id(&second_result.route_id).unwrap().geography
    );
}

#[test]
fn invalid_frontier_payload_cannot_create_a_region_or_consume_ids() {
    let (mut world, mut frontier, _) = land_connection_fixture();
    frontier.target_region.x += 1;
    let original = serde_json::to_value(&world).unwrap();
    let result = materialize_frontier(&mut world, 3, &frontier);

    assert_eq!(result, Err(FrontierMaterializationError::InvalidFrontier));
    assert_eq!(serde_json::to_value(&world).unwrap(), original);
}
