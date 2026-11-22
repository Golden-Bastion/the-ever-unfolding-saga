use crate::geography::caves::{connection_targets_for_location, materialize_connection, CaveConnectionKind};
use crate::geography::{MacroCoordinate, RegionCoordinate};
use crate::model::{Location, LocationGeographyRecord, Region, World, WorldMode};

fn surface_world(seed: u64, anchor: MacroCoordinate) -> World {
    let mut world = World::new("Cave topology test", WorldMode::New);
    world.geography_seed = Some(seed);
    world.geography_ruleset_version = 2;
    world.next_id = 4;
    world.regions.push(Region {
        id: 2,
        name: "Surface".into(),
        description: String::new(),
        location_ids: vec![3],
        coordinate: Some(anchor.region_and_local().0),
    });
    world.locations.push(Location {
        id: 3,
        name: "Surface approach".into(),
        description: String::new(),
        region_id: 2,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: Vec::new(),
        spatial_layout: None,
    });
    world.location_geography.push(LocationGeographyRecord {
        location_id: 3,
        stable_key: "location.test.surface".into(),
        anchor,
        footprint_radius_samples: 0,
    });
    world
}

#[test]
fn surface_shaft_materializes_one_persistent_cave_chamber_and_route() {
    let mut found = None;
    'search: for seed in 1..64 {
        for y in (-64..=64).step_by(16) {
            for x in (-64..=64).step_by(16) {
                let world = surface_world(seed, MacroCoordinate::new(x, y));
                if let Some(target) = connection_targets_for_location(&world, 3).into_iter()
                    .find(|target| target.kind == CaveConnectionKind::SurfaceEntrance)
                {
                    found = Some((world, target));
                    break 'search;
                }
            }
        }
    }
    let (mut world, target) = found.expect("search window should contain a deterministic surface shaft");
    let first = materialize_connection(&mut world, 3, &target).unwrap();
    assert!(first.created_location);
    assert!(first.created_region);
    assert_eq!(world.location_geography.iter()
        .find(|record| record.location_id == first.location_id).unwrap().anchor,
        target.destination_anchor);
    assert_eq!(target.destination_anchor.stratum, -1);
    assert!(world.route_by_id(&first.route_id)
        .is_some_and(|route| route.connects(3, first.location_id)));

    let second = materialize_connection(&mut world, 3, &target).unwrap();
    assert_eq!(first.location_id, second.location_id);
    assert_eq!(first.route_id, second.route_id);
    assert!(!second.created_location);
    assert!(!second.created_region);
    assert_eq!(world.locations.iter().filter(|location| location.id == first.location_id).count(), 1);
}

#[test]
fn cave_chamber_targets_are_stable_and_only_follow_real_passages() {
    let mut found = None;
    'search: for seed in 1..64 {
        for stratum in [-1, -2, -3] {
            for y in (-128..=128).step_by(16) {
                for x in (-128..=128).step_by(16) {
                    let anchor = MacroCoordinate::in_stratum(x, y, stratum);
                    let Some(sample) = crate::geography::underground::sample_underground(seed, 2, anchor) else {
                        continue;
                    };
                    let Some(chamber) = sample.network_cell() else {
                        continue;
                    };
                    if chamber != anchor || sample.connections() == 0 {
                        continue;
                    }
                    let mut world = surface_world(seed, MacroCoordinate::new(0, 0));
                    world.regions[0].coordinate = Some(chamber.region_and_local().0);
                    world.locations[0].name = "Cave".into();
                    world.location_geography[0].anchor = chamber;
                    world.location_geography[0].stable_key = format!("location.cave.{}.{}.{}", chamber.stratum, chamber.x, chamber.y);
                    found = Some((world, chamber));
                    break 'search;
                }
            }
        }
    }
    let (world, chamber) = found.expect("search window should contain a connected cave chamber");
    let targets = connection_targets_for_location(&world, 3);
    assert!(!targets.is_empty());
    assert!(targets.iter().all(|target| target.destination_anchor.stratum < 0));
    assert!(targets.iter().any(|target| target.kind == CaveConnectionKind::HorizontalPassage));
    assert_eq!(targets, connection_targets_for_location(&world, 3));
    assert_eq!(targets.iter().filter(|target| target.destination_anchor == chamber).count(), 0);
}
