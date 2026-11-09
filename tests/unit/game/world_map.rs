use super::*;
use crate::model::{Location, Region, WorldMode};
use crate::spatial::{GridPosition, LocalMap, Terrain};

fn test_state() -> GameState {
    let mut state = crate::model::create_new_state(
        "World Map Test",
        WorldMode::New,
        "Tester".to_string(),
        "Wanderer".to_string(),
    );
    state.world.regions = vec![Region {
        id: 1,
        name: "Ashen Vale".to_string(),
        description: "A valley of old roads.".to_string(),
        location_ids: vec![10, 11, 12, 13, 14],
    }];
    state.world.locations = vec![
        location(10, "Crossroads", vec![11, 12]),
        location(11, "Old Shrine", vec![10, 13]),
        location(12, "Cinder Hamlet", vec![10]),
        location(13, "Sunken Watch", vec![11]),
        location(14, "Far Ruin", vec![]),
    ];
    state.world.local_maps.clear();
    state.character.location_id = 10;
    state
}

fn location(id: u64, name: &str, exits: Vec<u64>) -> Location {
    Location {
        id,
        name: name.to_string(),
        description: format!("A place called {name}."),
        region_id: 1,
        dangerous: id == 13,
        corpse_ids: Vec::new(),
        exits,
        spatial_layout: None,
    }
}

fn explored_map(location_id: u64) -> LocalMap {
    LocalMap {
        location_id: format!("location.{location_id}"),
        generator_version: 1,
        layout_id: None,
        seed: 0,
        width: 8,
        height: 8,
        terrain: vec![Terrain::Floor; 64],
        entry: GridPosition { x: 1, y: 1 },
        points_of_interest: Vec::new(),
        exits: Vec::new(),
        discovered_tiles: Vec::new(),
        terrain_changes: Vec::new(),
        props: Vec::new(),
        interactables: Vec::new(),
    }
}

#[test]
fn map_shows_current_location_and_its_immediate_routes_without_revealing_the_world() {
    let state = test_state();

    let view = build_view(&state);

    let ids = view
        .nodes
        .iter()
        .map(|node| node.location.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![10, 11, 12]);
    assert!(view.nodes.iter().any(|node| node.current && node.explored));
    assert!(view.nodes.iter().filter(|node| node.reachable).count() == 2);
    assert!(!ids.contains(&13));
    assert!(!ids.contains(&14));
    assert_eq!(
        view.destinations
            .iter()
            .map(|location| location.id)
            .collect::<Vec<_>>(),
        vec![11, 12]
    );
}

#[test]
fn explored_locations_extend_the_map_by_one_route_without_exposing_the_frontier_beyond_it() {
    let mut state = test_state();
    state.world.local_maps.push(explored_map(11));

    let view = build_view(&state);

    let ids = view
        .nodes
        .iter()
        .map(|node| node.location.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![10, 11, 12, 13]);
    assert!(
        view.nodes
            .iter()
            .find(|node| node.location.id == 11)
            .unwrap()
            .explored
    );
    assert!(
        !view
            .nodes
            .iter()
            .find(|node| node.location.id == 13)
            .unwrap()
            .explored
    );
    assert!(!ids.contains(&14));
}

#[test]
fn map_routes_are_derived_only_from_established_graph_edges() {
    let mut state = test_state();
    state.world.local_maps.push(explored_map(11));

    let view = build_view(&state);
    let mut routes = view
        .routes
        .iter()
        .map(|route| (route.from_id, route.to_id, route.bidirectional))
        .collect::<Vec<_>>();
    routes.sort_unstable();

    assert_eq!(routes, vec![(10, 11, true), (10, 12, true), (11, 13, true)]);
    assert_eq!(
        view.nodes
            .iter()
            .find(|node| node.location.id == 13)
            .unwrap()
            .location
            .region_name,
        "Ashen Vale"
    );
}

#[test]
fn map_does_not_invent_reciprocal_connections_for_one_way_routes() {
    let mut state = test_state();
    state.world.locations[1].exits = vec![13];
    state.world.locations[3].exits.clear();
    state.world.local_maps.push(explored_map(11));

    let view = build_view(&state);
    let route = view
        .routes
        .iter()
        .find(|route| route.from_id == 11 && route.to_id == 13)
        .expect("the one-way route should be present");

    assert!(!route.bidirectional);
}
