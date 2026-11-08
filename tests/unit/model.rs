use super::*;

#[test]
fn world_history_is_bounded_to_recent_entries() {
    let mut state = create_new_state(
        "Test World",
        WorldMode::New,
        "First Warden".to_string(),
        "Ash Walker".to_string(),
    );
    for turn in 0..550 {
        state.world.record_history(turn, format!("entry {turn}"));
    }
    assert_eq!(state.world.history.len(), MAX_HISTORY_ENTRIES);
    assert_eq!(state.world.history.first().unwrap().text, "entry 50");
    assert_eq!(state.world.history.last().unwrap().text, "entry 549");
}

#[test]
fn new_worlds_record_generation_metadata() {
    let state = create_new_state(
        "Test World",
        WorldMode::New,
        "First Warden".to_string(),
        "Ash Walker".to_string(),
    );

    let generation = state
        .world
        .generation
        .expect("new worlds should record generation metadata");
    assert_eq!(generation.region_count, 3);
    assert_eq!(generation.location_count, 12);
    assert_eq!(generation.extra_edges, 8);
    let expected_location_count = generation.location_count.max(state.world.locations.len());
    assert_eq!(state.world.locations.len(), expected_location_count);
    assert_eq!(
        state
            .world
            .location_by_name("Ashen Gate")
            .map(|location| location.id),
        Some(state.character.location_id)
    );
}

#[test]
fn inherited_world_preserves_event_cooldowns() {
    let mut state = create_new_state(
        "Test World",
        WorldMode::New,
        "First Warden".to_string(),
        "Ash Walker".to_string(),
    );
    state.character.turn = 9;
    state.world.event_cooldowns.push(EventCooldown {
        event_id: "travel.ruined-road".to_string(),
        ready_at_turn: 14,
    });
    state
        .world
        .record_history(9, "A life ended here.".to_string());

    let inherited = create_inherited_state(
        &state,
        "Second Warden".to_string(),
        "Ash Walker".to_string(),
    );

    assert!(matches!(inherited.world.mode, WorldMode::Inherited));
    assert_eq!(inherited.world.event_cooldowns, state.world.event_cooldowns);
    assert_eq!(inherited.world.event_cooldowns[0].ready_at_turn, 14);
    assert_eq!(inherited.world.generation, state.world.generation);
}

#[test]
fn inherited_world_keeps_spatial_maps_but_resets_character_session() {
    let mut state = create_new_state(
        "Spatial Inheritance",
        WorldMode::New,
        "First Warden".to_string(),
        "Ash Walker".to_string(),
    );
    let mut map = crate::spatial::generate_local_map(
        91,
        "location.old_shrine",
        crate::spatial::LocalMapConfig::default(),
    );
    map.mark_tile_discovered(map.entry);
    map.points_of_interest[0].discovered = true;
    map.points_of_interest[0].interacted = true;
    map.set_terrain(map.entry, crate::spatial::Terrain::Rubble);
    state.world.local_maps.push(map.clone());
    state.character.exploration_session = Some(crate::spatial::ExplorationSession {
        location_id: map.location_id.clone(),
        position: map.entry,
    });
    state.character.exploration_position = Some(ExplorationPosition {
        x: f32::from(map.entry.x) + 0.7,
        y: f32::from(map.entry.y) + 0.2,
    });

    let inherited = create_inherited_state(
        &state,
        "Second Warden".to_string(),
        "Ash Walker".to_string(),
    );

    assert_eq!(inherited.world.local_maps, vec![map.clone()]);
    assert!(inherited.world.local_maps[0].is_tile_discovered(map.entry));
    assert!(inherited.world.local_maps[0].points_of_interest[0].interacted);
    assert_eq!(
        inherited.world.local_maps[0].terrain_at(map.entry),
        Some(crate::spatial::Terrain::Rubble)
    );
    assert!(inherited.character.exploration_session.is_none());
    assert!(inherited.character.exploration_position.is_none());
}

#[test]
fn objective_defaults_are_safe_for_old_saves() {
    let json = r#"{
        "kind": "acquire_item",
        "target": "Old Token"
    }"#;
    let objective: QuestObjective =
        serde_json::from_str(json).expect("legacy objective should load");
    assert_eq!(objective.required, 1);
    assert_eq!(objective.progress, 0);
    assert!(!objective.completed);
}

#[test]
fn character_appearance_is_persisted_and_inherited() {
    let mut state = create_new_state(
        "Appearance Test",
        WorldMode::New,
        "First Warden".to_string(),
        "Ash Walker".to_string(),
    );
    state.character.appearance_parts = vec!["face.custom".to_string(), "hair.custom".to_string()];

    let inherited = create_inherited_state(
        &state,
        "Second Warden".to_string(),
        "Ash Walker".to_string(),
    );
    assert_eq!(inherited.character.appearance_parts, state.character.appearance_parts);

    let mut legacy = serde_json::to_value(&state.character).expect("character should serialize");
    legacy
        .as_object_mut()
        .expect("character should serialize as an object")
        .remove("appearance_parts");
    let legacy: Character =
        serde_json::from_value(legacy).expect("legacy character should load without appearance data");
    assert!(legacy.appearance_parts.is_empty());
}
