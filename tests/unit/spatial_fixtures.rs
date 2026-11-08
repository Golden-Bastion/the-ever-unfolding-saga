use super::*;

use crate::spatial::{generate_local_map, LocalMapConfig, SpatialPropKind};

use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "teus-spatial-fixture-{label}-{}-{unique}",
        std::process::id()
    ))
}

#[test]
fn saved_fixture_round_trips_the_complete_local_map() {
    let directory = temporary_directory("round-trip");
    let mut map = generate_local_map(31, "developer.world_lab", LocalMapConfig::default());
    let free = (1..map.height - 1)
        .flat_map(|y| (1..map.width - 1).map(move |x| GridPosition { x, y }))
        .find(|position| {
            map.is_walkable(*position)
                && *position != map.entry
                && !map.points_of_interest.iter().any(|poi| poi.position == *position)
                && !map.exits.iter().any(|exit| exit.position == *position)
                && !map.interactables.iter().any(|actor| actor.position == *position)
        })
        .expect("generated map should have a free tile");
    assert!(map.set_terrain_for_editor(free, Terrain::Ash));
    let prop_position = (1..map.height - 1)
        .flat_map(|y| (1..map.width - 1).map(move |x| GridPosition { x, y }))
        .find(|position| {
            map.is_walkable(*position)
                && *position != map.entry
                && !map.points_of_interest.iter().any(|poi| poi.position == *position)
                && !map.exits.iter().any(|exit| exit.position == *position)
                && !map.interactables.iter().any(|actor| actor.position == *position)
        })
        .expect("generated map should have a free prop tile");
    assert!(map.place_prop("fixture.pillar", SpatialPropKind::Pillar, prop_position));

    let path = save_spatial_fixture(&directory, "Old Shrine Test", &map)
        .expect("fixture should be saved");
    let fixture = load_spatial_fixture(&path).expect("saved fixture should validate");

    assert_eq!(fixture.name, "Old Shrine Test");
    assert_eq!(fixture.map, map);
    assert!(path.file_name().unwrap().to_string_lossy().contains("old-shrine-test"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn duplicate_fixture_names_receive_suffixes_instead_of_overwriting() {
    let directory = temporary_directory("duplicate");
    let map = generate_local_map(9, "developer.world_lab", LocalMapConfig::default());
    let first = save_spatial_fixture(&directory, "Shared Name", &map).unwrap();
    let second = save_spatial_fixture(&directory, "Shared Name", &map).unwrap();

    assert_ne!(first, second);
    assert_eq!(load_spatial_fixture(&first).unwrap().name, "Shared Name");
    assert_eq!(load_spatial_fixture(&second).unwrap().name, "Shared Name (2)");
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invalid_maps_and_unknown_fixture_versions_are_rejected() {
    let directory = temporary_directory("invalid");
    let map = generate_local_map(14, "developer.world_lab", LocalMapConfig::default());
    let path = save_spatial_fixture(&directory, "Invalid Map", &map).unwrap();
    let mut fixture = load_spatial_fixture(&path).unwrap();
    fixture.map.terrain.pop();
    fs::write(
        &path,
        serde_json::to_vec(&fixture).expect("fixture should serialize"),
    )
    .unwrap();
    assert!(load_spatial_fixture(&path)
        .unwrap_err()
        .to_string()
        .contains("terrain has"));

    fixture.map = map;
    fixture.format_version = SPATIAL_FIXTURE_FORMAT_VERSION + 1;
    fs::write(
        &path,
        serde_json::to_vec(&fixture).expect("fixture should serialize"),
    )
    .unwrap();
    assert!(load_spatial_fixture(&path)
        .unwrap_err()
        .to_string()
        .contains("unsupported fixture format version"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn fixture_catalogue_keeps_invalid_files_visible_with_diagnostics() {
    let directory = temporary_directory("catalogue");
    let map = generate_local_map(5, "developer.world_lab", LocalMapConfig::default());
    let valid = save_spatial_fixture(&directory, "Valid Map", &map).unwrap();
    let invalid = directory.join("broken.json");
    fs::write(&invalid, b"{ not json").unwrap();

    let entries = list_spatial_fixtures(&[directory.as_path()]).unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| entry.path == valid && entry.map.is_some()));
    assert!(entries
        .iter()
        .any(|entry| entry.path == invalid && entry.error.is_some()));

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn fixture_names_cannot_create_paths_outside_the_fixture_directory() {
    let directory = temporary_directory("names");
    let map = generate_local_map(4, "developer.world_lab", LocalMapConfig::default());
    let path = save_spatial_fixture(&directory, "../../unsafe/name", &map).unwrap();
    assert_eq!(path.parent(), Some(directory.as_path()));
    assert!(path.file_name().unwrap().to_string_lossy().starts_with("unsafe-name"));
    fs::remove_dir_all(directory).unwrap();
}
