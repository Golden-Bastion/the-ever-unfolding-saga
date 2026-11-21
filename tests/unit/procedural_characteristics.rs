use super::*;
use crate::model::WorldMode;
use crate::procedural::{generate_world, WorldGenerationConfig};

fn generated_world() -> World {
    let mut world = generate_world(
        "Ashen",
        2026,
        WorldGenerationConfig {
            region_count: 4,
            location_count: 16,
            extra_edges: 8,
        },
    );
    world.mode = WorldMode::New;
    world.generation = Some(crate::model::WorldGenerationMetadata {
        seed: 2026,
        region_count: 4,
        location_count: 16,
        extra_edges: 8,
        geography_ruleset_version: crate::geography::GEOGRAPHY_RULESET_VERSION,
    });
    world
}

#[test]
fn same_generated_world_produces_identical_characteristics() {
    let a = generated_world();
    let b = generated_world();

    assert_eq!(
        generate_world_characteristics(&a),
        generate_world_characteristics(&b)
    );
}

#[test]
fn characteristics_are_non_empty_and_contextual() {
    let world = generated_world();
    let characteristics = generate_world_characteristics(&world);

    assert_eq!(characteristics.regions.len(), world.regions.len());
    assert_eq!(characteristics.locations.len(), world.locations.len());
    assert!(characteristics.regions.iter().all(|region| {
        !region.resources.is_empty()
            && !region.tags.is_empty()
            && (1..=100).contains(&region.prosperity)
            && (1..=100).contains(&region.danger)
    }));
    assert!(characteristics.locations.iter().all(|location| {
        !location.resources.is_empty()
            && !location.tags.is_empty()
            && world
                .location_by_id(location.location_id)
                .map(|world_location| world_location.region_id == location.region_id)
                .unwrap_or(false)
    }));
    assert!(characteristics.locations.iter().any(|location| {
        characteristics
            .regions
            .iter()
            .find(|region| region.region_id == location.region_id)
            .map(|region| {
                location
                    .resources
                    .iter()
                    .any(|resource| region.resources.contains(resource))
            })
            .unwrap_or(false)
    }));
}

#[test]
fn characteristics_can_be_rebuilt_after_cloning_the_world() {
    let world = generated_world();
    let cloned = world.clone();

    assert_eq!(
        generate_world_characteristics(&world),
        generate_world_characteristics(&cloned)
    );
}
