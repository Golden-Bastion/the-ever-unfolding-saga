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
    assert_eq!(
        generation.geography_ruleset_version,
        crate::geography::GEOGRAPHY_RULESET_VERSION
    );
    assert!(state
        .world
        .regions
        .iter()
        .all(|region| region.coordinate.is_some()));
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
    state
        .character
        .record_evidence(map.points_of_interest[0].id.clone());
    state.character.record_evidence("evidence.previous_life");
    state.character.tracked_quest_id = Some(701);
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
    assert!(
        inherited.character.known_evidence_ids.is_empty(),
        "new characters must not inherit the previous life’s individual knowledge"
    );
    assert_eq!(inherited.character.tracked_quest_id, None);
    assert_eq!(
        inherited.world.local_maps[0].terrain_at(map.entry),
        Some(crate::spatial::Terrain::Rubble)
    );
    assert!(inherited.character.exploration_session.is_none());
    assert!(inherited.character.exploration_position.is_none());
}

#[test]
fn character_evidence_defaults_empty_for_old_saves() {
    let mut state = create_new_state(
        "Legacy Character",
        WorldMode::New,
        "First Warden".into(),
        "Ash Walker".into(),
    );
    state.character.record_evidence("evidence.known");
    let serialized = serde_json::to_value(&state.character).unwrap();
    let restored: Character = serde_json::from_value(serialized.clone())
        .expect("current character evidence should survive save serialization");
    assert_eq!(restored.known_evidence_ids, vec!["evidence.known"]);

    let mut legacy = serialized;
    legacy.as_object_mut().unwrap().remove("known_evidence_ids");
    let restored: Character = serde_json::from_value(legacy)
        .expect("older characters deserialize without evidence knowledge");
    assert!(restored.known_evidence_ids.is_empty());
}

#[test]
fn tracked_quest_defaults_to_none_and_round_trips_without_breaking_old_saves() {
    let mut state = create_new_state(
        "Quest Tracking Save",
        WorldMode::New,
        "First Warden".into(),
        "Ash Walker".into(),
    );
    assert_eq!(state.character.tracked_quest_id, None);
    state.character.tracked_quest_id = Some(704);
    let serialized = serde_json::to_value(&state.character).unwrap();
    let restored: Character = serde_json::from_value(serialized.clone())
        .expect("tracked quest should survive character serialization");
    assert_eq!(restored.tracked_quest_id, Some(704));

    let mut legacy = serialized;
    legacy.as_object_mut().unwrap().remove("tracked_quest_id");
    let restored: Character = serde_json::from_value(legacy)
        .expect("older characters deserialize without quest tracking");
    assert_eq!(restored.tracked_quest_id, None);
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
    assert!(objective.target_label.is_none());

    let evidence = QuestObjective::discover_evidence("evidence.altar", "Black Altar");
    let serialized = serde_json::to_value(&evidence).expect("evidence objective should serialize");
    let restored: QuestObjective =
        serde_json::from_value(serialized).expect("evidence objective should deserialize");
    assert_eq!(restored.kind, QuestObjectiveKind::DiscoverEvidence);
    assert_eq!(restored.target, "evidence.altar");
    assert_eq!(restored.target_label.as_deref(), Some("Black Altar"));
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
    assert_eq!(
        inherited.character.appearance_parts,
        state.character.appearance_parts
    );

    let mut legacy = serde_json::to_value(&state.character).expect("character should serialize");
    legacy
        .as_object_mut()
        .expect("character should serialize as an object")
        .remove("appearance_parts");
    let legacy: Character = serde_json::from_value(legacy)
        .expect("legacy character should load without appearance data");
    assert!(legacy.appearance_parts.is_empty());
    assert!(legacy.known_evidence_ids.is_empty());
}

#[test]
fn world_routes_canonicalize_endpoint_order_and_reverse_direction() {
    let route = WorldRoute::new("route.test.descending", 22, 11, RouteDirection::AtoB);

    assert_eq!(route.location_a_id, 11);
    assert_eq!(route.location_b_id, 22);
    assert_eq!(route.direction, RouteDirection::BtoA);
    assert!(route.allows_travel(22, 11));
    assert!(!route.allows_travel(11, 22));
    assert!(!route.allows_travel(11, 33));
}

#[test]
fn legacy_route_migration_preserves_missing_directions_and_is_idempotent() {
    let mut world = World::new("Legacy Route Migration", WorldMode::New);
    world.locations = vec![
        Location {
            id: 10,
            name: "First".into(),
            description: String::new(),
            region_id: 1,
            dangerous: false,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
            spatial_layout: None,
        },
        Location {
            id: 20,
            name: "Second".into(),
            description: String::new(),
            region_id: 1,
            dangerous: false,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
            spatial_layout: None,
        },
    ];
    assert!(world.add_route(WorldRoute::new(
        "route.content.forward",
        10,
        20,
        RouteDirection::AtoB,
    )));
    world.location_by_id_mut(20).unwrap().exits.push(10);
    let expected_next_id = world.next_id;

    world.migrate_legacy_exits();

    let compatibility_route = world
        .route_by_id("route.compat.legacy.10.20")
        .expect("the missing legacy direction should receive a compatibility route");
    assert_eq!(compatibility_route.direction, RouteDirection::BtoA);
    assert!(world.can_travel_between(10, 20));
    assert!(world.can_travel_between(20, 10));
    assert_eq!(world.next_id, expected_next_id);
    let migrated_routes = world.routes.clone();

    world.migrate_legacy_exits();

    assert_eq!(world.routes, migrated_routes);
    assert_eq!(world.next_id, expected_next_id);
}

#[test]
fn parallel_world_routes_keep_distinct_identity_and_update_the_derived_exit_cache() {
    let mut state = create_new_state(
        "Parallel Routes",
        WorldMode::New,
        "First Warden".to_string(),
        "Ash Walker".to_string(),
    );
    state.world.routes.clear();
    state.world.rebuild_location_exit_cache();
    let source_id = state.character.location_id;
    let source = state.world.location_by_id(source_id).unwrap().clone();
    let destination_id = state.world.allocate_id();
    state.world.locations.push(Location {
        id: destination_id,
        name: "Route Test Destination".into(),
        description: "A place joined by two distinct routes.".into(),
        region_id: source.region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: Vec::new(),
        spatial_layout: None,
    });

    assert!(state.world.add_route(WorldRoute::new(
        "route.test.trail",
        source_id,
        destination_id,
        RouteDirection::AtoB,
    )));
    assert!(state.world.add_route(WorldRoute::new(
        "route.test.bridge",
        source_id,
        destination_id,
        RouteDirection::Bidirectional,
    )));
    assert_eq!(state.world.routes.len(), 2);
    assert!(state.world.can_travel_between(source_id, destination_id));
    assert!(state.world.can_travel_between(destination_id, source_id));
    assert_eq!(
        state.world.location_by_id(source_id).unwrap().exits,
        vec![destination_id]
    );

    assert!(state.world.remove_route("route.test.bridge"));
    assert_eq!(state.world.routes.len(), 1);
    assert!(state.world.can_travel_between(source_id, destination_id));
    assert!(!state.world.can_travel_between(destination_id, source_id));

    assert!(state.world.remove_route("route.test.trail"));
    assert!(state.world.routes.is_empty());
    assert!(state
        .world
        .location_by_id(source_id)
        .unwrap()
        .exits
        .is_empty());
}

#[test]
fn waterbody_modifications_are_generic_deterministic_and_idempotent() {
    let mut world = World::new("Waterbody State", WorldMode::New);
    assert!(world.set_waterbody_property("lake.12.34.abcd", "name", "Mirror Lake"));
    assert!(world.set_waterbody_property("lake.12.34.abcd", "water_level", "0.42"));
    assert!(world.set_waterbody_property("lake.2.3.ef01", "discovered", "true"));
    assert!(!world.set_waterbody_property("lake.12.34.abcd", "name", "Mirror Lake"));
    assert!(!world.set_waterbody_property("", "name", "Invalid"));
    assert!(!world.set_waterbody_property("lake.12.34.abcd", "", "Invalid"));

    assert_eq!(
        world.waterbody_modifications[0].waterbody_id,
        "lake.12.34.abcd"
    );
    assert_eq!(
        world
            .waterbody_modification("lake.12.34.abcd")
            .unwrap()
            .properties["name"],
        "Mirror Lake"
    );

    let serialized = serde_json::to_value(&world).unwrap();
    let restored: World = serde_json::from_value(serialized.clone()).unwrap();
    assert_eq!(
        restored.waterbody_modifications,
        world.waterbody_modifications
    );

    let mut legacy = serialized;
    legacy
        .as_object_mut()
        .unwrap()
        .remove("waterbody_modifications");
    let restored: World = serde_json::from_value(legacy)
        .expect("worlds saved before waterbody changes should still load");
    assert!(restored.waterbody_modifications.is_empty());
}
