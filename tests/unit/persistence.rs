use super::*;
use crate::content::{EventConditionContent, EventEffectContent};
use crate::model::{create_new_state, EventCooldown, WorldMode};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be valid")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ever_unfolding_saga_save_tests_{}", stamp));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

#[test]
fn compressed_save_round_trips() {
    let dir = temp_dir();
    let path = character_save_path(&dir, "Ash Walker");
    let mut state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".to_string(),
        "Ash Walker".to_string(),
    );
    state.character.turn = 7;
    state.world.event_cooldowns.push(EventCooldown {
        event_id: "travel.ruined-road".to_string(),
        ready_at_turn: 11,
    });
    let mut map = crate::spatial::generate_local_map(
        77,
        "location.1",
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
    state.character.exploration_position = Some(crate::model::ExplorationPosition {
        x: f32::from(map.entry.x) + 0.73,
        y: f32::from(map.entry.y) + 0.29,
    });

    let generation = state.world.generation;
    let original_description = state.world.locations[0].description.clone();
    let original_exits = state.world.locations[0].exits.clone();
    state.world.locations[0].description = "A scar left by the first warden.".to_string();
    state.world.locations[0].dangerous = !state.world.locations[0].dangerous;
    state
        .world
        .record_history(7, "The world has changed.".to_string());
    let mutated_description = state.world.locations[0].description.clone();
    let history_len = state.world.history.len();

    save_game(&path, &state).expect("save should succeed");
    let bytes = fs::read(&path).expect("save should exist");
    assert!(is_gzip(&bytes));

    let loaded = load_game(&path).expect("load should succeed");
    assert_eq!(loaded.character.turn, 7);
    assert_eq!(loaded.character.name, "Tester");
    assert_eq!(loaded.world.event_cooldowns, state.world.event_cooldowns);
    assert_eq!(loaded.world.local_maps, vec![map.clone()]);
    assert!(loaded.world.local_maps[0].points_of_interest[0].interacted);
    assert!(loaded.world.local_maps[0].is_tile_discovered(map.entry));
    assert_eq!(
        loaded.character.exploration_session,
        state.character.exploration_session
    );
    assert_eq!(
        loaded.character.exploration_position,
        state.character.exploration_position
    );
    assert_eq!(loaded.rng_state, state.rng_state);
    assert_eq!(loaded.world.generation, generation);
    assert_eq!(loaded.world.locations[0].description, mutated_description);
    assert_ne!(loaded.world.locations[0].description, original_description);
    assert_eq!(loaded.world.locations[0].exits, original_exits);
    assert_eq!(loaded.world.history.len(), history_len);
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn exclusive_save_creation_never_overwrites_an_existing_file() {
    let dir = temp_dir();
    let path = dir.join("existing.save");
    fs::write(&path, b"keep this file").expect("existing file should be created");
    let state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".to_string(),
        "Import Collision".to_string(),
    );

    let error = write_save_file_exclusive(&path, &state, &[])
        .expect_err("exclusive creation must reject an existing destination");
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(
        fs::read(&path).expect("existing file should remain readable"),
        b"keep this file"
    );
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn save_list_metadata_exposes_character_world_and_format_details() {
    let dir = temp_dir();
    let path = character_save_path(&dir, "Metadata Test");
    let mut state = create_new_state(
        "The Cinder March",
        WorldMode::New,
        "Ash".to_string(),
        "Warden".to_string(),
    );
    state.character.level = 4;
    state.world.day = 12;
    let dependencies = vec![SavedModDependency {
        id: "campaign.example".into(),
        name: "Example Campaign".into(),
        version: "1.2.3".into(),
    }];

    save_game_with_dependencies(&path, &state, &dependencies)
        .expect("save with metadata should succeed");
    let metadata = read_save_metadata(&path).expect("save metadata should be readable");

    assert_eq!(metadata.character_display_name, "Ash the Warden");
    assert_eq!(metadata.character_level, 4);
    assert_eq!(metadata.world_name, "The Cinder March");
    assert_eq!(metadata.world_day, 12);
    assert_eq!(metadata.save_file_version, SAVE_FILE_VERSION);
    assert_eq!(metadata.mod_dependency_count, 1);
    assert_eq!(
        metadata.details_line(),
        "Level 4 | The Cinder March | Day 12 | Format v3 | 1 mod"
    );
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn save_metadata_details_line_pluralizes_multiple_mod_dependencies() {
    let metadata = SaveMetadata {
        character_display_name: "Ash the Warden".to_string(),
        character_level: 1,
        world_name: "The Cinder March".to_string(),
        world_day: 1,
        save_file_version: SAVE_FILE_VERSION,
        mod_dependency_count: 2,
    };

    assert_eq!(
        metadata.details_line(),
        "Level 1 | The Cinder March | Day 1 | Format v3 | 2 mods"
    );
}

#[test]
fn save_metadata_round_trips_mod_dependencies() {
    let dir = temp_dir();
    let path = character_save_path(&dir, "Dependency Test");
    let state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".to_string(),
        "Dependency Test".to_string(),
    );
    let dependencies = vec![
        SavedModDependency {
            id: "campaign.example".into(),
            name: "Example Campaign".into(),
            version: "1.2.3".into(),
        },
        SavedModDependency {
            id: "assets.example".into(),
            name: "Example Assets".into(),
            version: "0.4.0".into(),
        },
    ];

    save_game_with_dependencies(&path, &state, &dependencies)
        .expect("save with dependencies should succeed");
    let loaded = load_game_with_metadata(&path).expect("save metadata should load");
    assert_eq!(loaded.mod_dependencies, dependencies);
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn older_save_without_spatial_fields_loads_with_defaults() {
    let dir = temp_dir();
    let path = dir.join("older-save.json");
    let state = create_new_state(
        "Legacy Spatial World",
        WorldMode::New,
        "Legacy Warden".to_string(),
        "Ash Walker".to_string(),
    );
    let payload = SaveFile {
        save_file_version: SAVE_FILE_VERSION,
        game: &state,
        runtime_generated_events: Vec::new(),
        mod_dependencies: Vec::new(),
    };
    let mut json: serde_json::Value = serde_json::to_value(payload).expect("save should serialize");
    let world = json["game"]["world"]
        .as_object_mut()
        .expect("world should be an object");
    world.remove("local_maps");
    let character = json["game"]["character"]
        .as_object_mut()
        .expect("character should be an object");
    character.remove("exploration_session");
    character.remove("exploration_position");
    fs::write(
        &path,
        serde_json::to_vec(&json).expect("legacy JSON should serialize"),
    )
    .expect("legacy save should be written");

    let loaded = load_game(&path).expect("older save should remain loadable");
    assert!(loaded.world.local_maps.is_empty());
    assert!(loaded.character.exploration_session.is_none());
    assert!(loaded.character.exploration_position.is_none());
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn save_replacement_keeps_the_latest_complete_save_readable() {
    let dir = temp_dir();
    let path = character_save_path(&dir, "Replacement Test");
    let mut first = create_new_state(
        "Test World",
        WorldMode::New,
        "First".to_string(),
        "Ash Walker".to_string(),
    );
    first.character.turn = 3;
    save_game(&path, &first).expect("initial save should succeed");

    let mut second = first.clone();
    second.character.turn = 9;
    save_game(&path, &second).expect("replacement save should succeed");

    let loaded = load_game(&path).expect("replacement save should remain readable");
    assert_eq!(loaded.character.turn, 9);
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn runtime_generated_events_round_trip_separately_from_authored_content() {
    let dir = temp_dir();
    let path = character_save_path(&dir, "Generated Events");
    let mut state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".to_string(),
        "Ash Walker".to_string(),
    );
    let generated = EventContent {
        id: "generated.evolution.event.test".to_string(),
        trigger: "travel_arrival".to_string(),
        weight: 1,
        chance_percent: Some(100),
        cooldown_turns: Some(12),
        conditions: Some(EventConditionContent {
            dangerous: Some(true),
            ..Default::default()
        }),
        effects: vec![EventEffectContent::History {
            text: "A generated sign remains after the save.".to_string(),
        }],
    };
    let authored_id = state
        .campaign_content
        .as_ref()
        .unwrap()
        .events
        .first()
        .map(|event| event.id.clone())
        .expect("authored event should exist");
    state
        .campaign_content
        .as_mut()
        .unwrap()
        .events
        .push(generated.clone());

    save_game(&path, &state).expect("save should succeed");
    let loaded = load_game(&path).expect("load should succeed");
    let events = &loaded.campaign_content.as_ref().unwrap().events;
    assert!(events.iter().any(|event| event.id == authored_id));
    assert!(events.iter().any(|event| event.id == generated.id));
    assert_eq!(
        events
            .iter()
            .filter(|event| event.id == generated.id)
            .count(),
        1
    );
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn legacy_json_save_still_loads() {
    let dir = temp_dir();
    let path = legacy_save_path(&dir);
    let state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".to_string(),
        "Ash Walker".to_string(),
    );
    let payload = SaveFile {
        save_file_version: SAVE_FILE_VERSION,
        game: &state,
        runtime_generated_events: Vec::new(),
        mod_dependencies: Vec::new(),
    };
    fs::write(
        &path,
        serde_json::to_vec_pretty(&payload).expect("json should serialize"),
    )
    .expect("legacy save should be written");

    let loaded = load_game(&path).expect("legacy save should load");
    assert_eq!(loaded.character.name, state.character.name);
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn sanitize_filename_component_is_filesystem_safe() {
    assert_eq!(
        sanitize_filename_component("Ash/Walker:Night"),
        "Ash_Walker_Night"
    );
    assert_eq!(sanitize_filename_component("..."), "unnamed");
    assert_eq!(sanitize_filename_component("  Geralt  "), "Geralt");
}

#[test]
fn character_save_path_appends_character_name() {
    let path = character_save_path(Path::new("."), "Ash Walker");
    assert_eq!(
        path,
        PathBuf::from("./ever_unfolding_saga_save_Ash Walker.json.gz")
    );
}

#[test]
fn oversized_gzip_is_rejected_after_decompression() {
    let dir = temp_dir();
    let path = dir.join("oversized.json.gz");
    let file = File::create(&path).expect("save should be created");
    let mut encoder = GzEncoder::new(file, Compression::default());
    let chunk = vec![b'x'; 1024 * 1024];
    for _ in 0..9 {
        encoder
            .write_all(&chunk)
            .expect("test payload should write");
    }
    encoder.finish().expect("gzip should finish");

    let err = load_game(&path).expect_err("oversized gzip should fail");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("decompressed"));
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}

#[test]
fn invalid_gzip_is_reported_as_invalid_data() {
    let dir = temp_dir();
    let path = dir.join("broken.json.gz");
    fs::write(&path, [0x1f, 0x8b, 0x00, 0x01]).expect("broken gzip should be written");

    let err = load_game(&path).expect_err("invalid gzip should fail");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    fs::remove_dir_all(&dir).expect("temporary directory should be removed");
}
