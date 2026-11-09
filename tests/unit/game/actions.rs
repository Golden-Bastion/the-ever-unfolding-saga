use super::{fast_travel_to, find_known_travel_path, has_reciprocal_location_connection, travel_to, MEDITATION_TARGETS};
use crate::model::{GameState, Location};
use crate::spatial::{GridPosition, LocalMap, Terrain};

#[test]
fn meditation_targets_advance_to_the_next_occurrence() {
    let current = 4;
    let morning = MEDITATION_TARGETS
        .iter()
        .find(|(slot, _)| *slot == 3)
        .map(|(slot, _)| *slot)
        .unwrap();
    let dusk = MEDITATION_TARGETS
        .iter()
        .find(|(slot, _)| *slot == 8)
        .map(|(slot, _)| *slot)
        .unwrap();
    assert_eq!((morning + 12 - current) % 12, 11);
    assert_eq!((dusk + 12 - current) % 12, 4);
}

#[test]
fn meditation_does_not_allow_a_zero_time_advance() {
    let current = 8;
    let dusk = MEDITATION_TARGETS
        .iter()
        .find(|(slot, _)| *slot == 8)
        .map(|(slot, _)| *slot)
        .unwrap();
    assert_eq!(((dusk + 12 - current) % 12).max(1), 1);
}

#[test]
fn travel_requires_reciprocal_world_graph_connections() {
    let mut game = crate::model::create_new_state(
        "Travel Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let source_id = game.character.location_id;
    let source = game
        .world
        .location_by_id(source_id)
        .expect("starting location should exist")
        .clone();
    let destination_id = game.world.allocate_id();
    game.world.locations.push(crate::model::Location {
        id: destination_id,
        name: "Distant Clearing".into(),
        description: "A clearing linked by a narrow path.".into(),
        region_id: source.region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: Vec::new(),
        spatial_layout: None,
    });
    game.world
        .location_by_id_mut(source_id)
        .unwrap()
        .exits
        .push(destination_id);

    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;
    let initial_history = game.world.history.len();
    assert!(!has_reciprocal_location_connection(&game, destination_id));
    assert!(travel_to(&mut game, destination_id).is_err());
    assert_eq!(game.character.location_id, source_id);
    assert_eq!(game.character.turn, initial_turn);
    assert_eq!(game.world.time_points, initial_time);
    assert_eq!(game.world.history.len(), initial_history);

    game.world
        .location_by_id_mut(destination_id)
        .unwrap()
        .exits
        .push(source_id);
    assert!(has_reciprocal_location_connection(&game, destination_id));
    travel_to(&mut game, destination_id).unwrap();
    assert_eq!(game.character.location_id, destination_id);
    assert_eq!(game.character.turn, initial_turn + 1);
}


fn fast_travel_fixture(
    dangerous_middle: bool,
    destination_known: bool,
    reciprocal_first_route: bool,
) -> (GameState, u64, u64) {
    let mut game = crate::model::create_new_state(
        "Fast Travel Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    game.campaign_content
        .as_mut()
        .expect("campaign content should be loaded")
        .events
        .retain(|event| event.trigger != "travel_arrival");

    let source_id = game.character.location_id;
    let source = game
        .world
        .location_by_id(source_id)
        .expect("starting location should exist")
        .clone();
    let middle_id = game.world.allocate_id();
    let destination_id = game.world.allocate_id();

    game.world.locations.push(Location {
        id: middle_id,
        name: "Known Waystation".into(),
        description: "A familiar stop along the road.".into(),
        region_id: source.region_id,
        dangerous: dangerous_middle,
        corpse_ids: Vec::new(),
        exits: if reciprocal_first_route {
            vec![source_id, destination_id]
        } else {
            vec![destination_id]
        },
        spatial_layout: None,
    });
    game.world.locations.push(Location {
        id: destination_id,
        name: "Known Destination".into(),
        description: "A place visited before.".into(),
        region_id: source.region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: vec![middle_id],
        spatial_layout: None,
    });
    game.world
        .location_by_id_mut(source_id)
        .expect("starting location should still exist")
        .exits = vec![middle_id];

    game.world.local_maps.push(known_map(middle_id));
    if destination_known {
        game.world.local_maps.push(known_map(destination_id));
    }

    (game, middle_id, destination_id)
}

fn known_map(location_id: u64) -> LocalMap {
    LocalMap {
        location_id: format!("location.{location_id}"),
        generator_version: 1,
        layout_id: None,
        seed: location_id,
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
fn fast_travel_follows_the_shortest_known_route_and_applies_normal_travel_per_hop() {
    let (mut game, middle_id, destination_id) = fast_travel_fixture(false, true, true);
    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;

    assert_eq!(
        find_known_travel_path(&game, destination_id),
        Some(vec![middle_id, destination_id])
    );

    let traveled = fast_travel_to(&mut game, destination_id).unwrap();

    assert_eq!(traveled, vec![middle_id, destination_id]);
    assert_eq!(game.character.location_id, destination_id);
    assert_eq!(game.character.turn, initial_turn + 2);
    assert_eq!(game.world.time_points, initial_time + 4);
    assert_eq!(
        game.world
            .history
            .iter()
            .filter(|entry| entry.text.contains("traveled to"))
            .count(),
        2
    );
}

#[test]
fn fast_travel_rejects_unvisited_destinations_without_changing_world_state() {
    let (mut game, _, destination_id) = fast_travel_fixture(false, false, true);
    let initial_location = game.character.location_id;
    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;
    let initial_history = game.world.history.len();

    assert!(find_known_travel_path(&game, destination_id).is_none());
    assert!(fast_travel_to(&mut game, destination_id).is_err());
    assert_eq!(game.character.location_id, initial_location);
    assert_eq!(game.character.turn, initial_turn);
    assert_eq!(game.world.time_points, initial_time);
    assert_eq!(game.world.history.len(), initial_history);
}

#[test]
fn fast_travel_cannot_use_one_way_connections() {
    let (game, _, destination_id) = fast_travel_fixture(false, true, false);

    assert!(find_known_travel_path(&game, destination_id).is_none());
}

#[test]
fn fast_travel_stops_at_an_intermediate_threat_instead_of_bypassing_it() {
    let (mut game, middle_id, destination_id) = fast_travel_fixture(true, true, true);
    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;

    let traveled = fast_travel_to(&mut game, destination_id).unwrap();

    assert_eq!(traveled, vec![middle_id]);
    assert_eq!(game.character.location_id, middle_id);
    assert!(game.threat.active);
    assert_eq!(game.character.turn, initial_turn + 1);
    assert_eq!(game.world.time_points, initial_time + 2);
}
