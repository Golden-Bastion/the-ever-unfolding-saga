use super::*;
use crate::content::load_campaign_content;
use std::collections::HashSet;

#[test]
fn same_seed_and_config_produce_identical_world_structure() {
    let config = WorldGenerationConfig {
        region_count: 4,
        location_count: 20,
        extra_edges: 10,
    };
    let a = generate_world("Ashen", 12345, config);
    let b = generate_world("Ashen", 12345, config);

    assert_eq!(a.regions.len(), b.regions.len());
    assert_eq!(a.locations.len(), b.locations.len());
    assert_eq!(
        a.regions
            .iter()
            .map(|region| (&region.name, &region.location_ids, region.coordinate))
            .collect::<Vec<_>>(),
        b.regions
            .iter()
            .map(|region| (&region.name, &region.location_ids, region.coordinate))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        a.locations
            .iter()
            .map(|location| (
                &location.name,
                location.region_id,
                location.dangerous,
                &location.exits
            ))
            .collect::<Vec<_>>(),
        b.locations
            .iter()
            .map(|location| (
                &location.name,
                location.region_id,
                location.dangerous,
                &location.exits
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn generated_world_is_connected_from_the_starting_location() {
    let world = generate_world("Ashen", 7, WorldGenerationConfig::default());
    let start = world
        .locations
        .first()
        .expect("generator creates a location")
        .id;
    let mut visited = HashSet::new();
    let mut stack = vec![start];

    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        if let Some(location) = world.location_by_id(id) {
            stack.extend(location.exits.iter().copied());
        }
    }

    assert_eq!(visited.len(), world.locations.len());
}

#[test]
fn generated_world_has_valid_regions_and_symmetric_exits() {
    let world = generate_world(
        "Ashen",
        99,
        WorldGenerationConfig {
            region_count: 5,
            location_count: 17,
            extra_edges: 12,
        },
    );
    let region_ids = world
        .regions
        .iter()
        .map(|region| region.id)
        .collect::<HashSet<_>>();
    let location_ids = world
        .locations
        .iter()
        .map(|location| location.id)
        .collect::<HashSet<_>>();

    assert!(!world.regions.is_empty());
    assert!(!world.locations.is_empty());
    assert!(world
        .locations
        .iter()
        .all(|location| region_ids.contains(&location.region_id)));
    assert!(world.regions.iter().all(|region| region
        .location_ids
        .iter()
        .all(|id| location_ids.contains(id))));
    let region_coordinates = world
        .regions
        .iter()
        .map(|region| {
            region
                .coordinate
                .expect("generated regions have coordinates")
        })
        .collect::<HashSet<_>>();
    assert_eq!(region_coordinates.len(), world.regions.len());
    assert!(world.locations.iter().all(|location| {
        location.exits.iter().all(|target_id| {
            world
                .location_by_id(*target_id)
                .map(|target| target.exits.contains(&location.id))
                .unwrap_or(false)
        })
    }));
}

#[test]
fn zero_sized_configuration_is_normalized_to_a_minimal_world() {
    let world = generate_world(
        "Ashen",
        1,
        WorldGenerationConfig {
            region_count: 0,
            location_count: 0,
            extra_edges: 0,
        },
    );

    assert_eq!(world.regions.len(), 1);
    assert_eq!(world.locations.len(), 1);
    assert_eq!(
        world.regions[0].coordinate,
        Some(crate::geography::RegionCoordinate::ORIGIN)
    );
    assert!(world.locations[0].exits.is_empty());
}

#[test]
fn authored_content_is_placed_deterministically_and_preserves_authored_relationships() {
    let content = load_campaign_content();
    let config = WorldGenerationConfig {
        region_count: 3,
        location_count: 12,
        extra_edges: 6,
    };
    let mut a = generate_world("Ashen", 2026, config);
    let mut b = generate_world("Ashen", 2026, config);

    let placed_a = place_authored_content(&mut a, &content);
    let placed_b = place_authored_content(&mut b, &content);

    assert_eq!(placed_a, content.world.locations.len());
    assert_eq!(placed_a, placed_b);
    assert_eq!(
        a.locations
            .iter()
            .map(|location| (
                &location.name,
                &location.description,
                location.region_id,
                location.dangerous,
                &location.exits
            ))
            .collect::<Vec<_>>(),
        b.locations
            .iter()
            .map(|location| (
                &location.name,
                &location.description,
                location.region_id,
                location.dangerous,
                &location.exits
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        a.regions
            .iter()
            .map(|region| (&region.name, &region.description, &region.location_ids))
            .collect::<Vec<_>>(),
        b.regions
            .iter()
            .map(|region| (&region.name, &region.description, &region.location_ids))
            .collect::<Vec<_>>()
    );

    assert_eq!(a.regions[0].name, content.world.region.name);

    let runtime_by_name = a
        .locations
        .iter()
        .map(|location| (location.name.as_str(), location.id))
        .collect::<std::collections::HashMap<_, _>>();
    for authored in &content.world.locations {
        let runtime_id = runtime_by_name[authored.name.as_str()];
        let runtime = a
            .location_by_id(runtime_id)
            .expect("authored location exists");
        assert_eq!(
            runtime.spatial_layout.as_deref(),
            authored.spatial_layout.as_deref()
        );
        for authored_exit in &authored.exits {
            let target_runtime_id = content
                .world
                .locations
                .iter()
                .find(|location| location.id == *authored_exit)
                .and_then(|location| runtime_by_name.get(location.name.as_str()))
                .copied()
                .expect("authored exit resolves to an authored location");
            assert!(runtime.exits.contains(&target_runtime_id));
        }
    }

    let authored_names = content
        .world
        .locations
        .iter()
        .map(|location| location.name.as_str())
        .collect::<HashSet<_>>();
    assert!(a
        .locations
        .iter()
        .take(content.world.locations.len())
        .all(|location| authored_names.contains(location.name.as_str())));
}
