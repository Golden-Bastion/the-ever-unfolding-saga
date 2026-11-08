use super::{has_reciprocal_location_connection, travel_to, MEDITATION_TARGETS};

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
    travel_to(&mut game, destination_id).unwrap();
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
