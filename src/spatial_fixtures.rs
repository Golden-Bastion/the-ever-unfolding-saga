//! Named, versioned JSON fixtures for the disposable spatial developer sandbox.

use std::collections::HashSet;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};

use crate::spatial::{GridPosition, LocalMap, Terrain};

pub const SPATIAL_FIXTURE_EXTENSION: &str = "json";
const SPATIAL_FIXTURE_FORMAT_VERSION: u32 = 1;
const MAX_SPATIAL_FIXTURE_BYTES: u64 = 8 * 1024 * 1024;
const MIN_MAP_DIMENSION: u16 = 8;
const MAX_MAP_DIMENSION: u16 = 128;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialMapFixture {
    format_version: u32,
    pub name: String,
    pub map: LocalMap,
}

#[derive(Debug, Clone)]
pub struct SpatialFixtureEntry {
    pub path: PathBuf,
    pub name: String,
    pub details: String,
    pub map: Option<LocalMap>,
    pub error: Option<String>,
}

/// Write a self-contained map fixture without touching campaign saves.
///
/// Repeated names receive a numeric suffix instead of overwriting an existing
/// fixture, so exporting an iteration never destroys an earlier experiment.
pub fn save_spatial_fixture(directory: &Path, name: &str, map: &LocalMap) -> io::Result<PathBuf> {
    validate_local_map(map).map_err(invalid_data)?;
    let display_name = name.trim();
    if display_name.is_empty() || display_name.chars().count() > 80 {
        return Err(invalid_input("fixture name must contain 1–80 characters"));
    }

    let file_stem = fixture_file_stem(display_name);
    if file_stem.is_empty() {
        return Err(invalid_input(
            "fixture name must contain at least one letter or number",
        ));
    }
    fs::create_dir_all(directory)?;

    for suffix in 1_u32.. {
        let unique_stem = if suffix == 1 {
            file_stem.clone()
        } else {
            format!("{file_stem}-{suffix}")
        };
        let path = directory.join(format!("{unique_stem}.teus-map.json"));
        if path.exists() {
            continue;
        }

        let unique_name = if suffix == 1 {
            display_name.to_string()
        } else {
            format!("{display_name} ({suffix})")
        };
        let fixture = SpatialMapFixture {
            format_version: SPATIAL_FIXTURE_FORMAT_VERSION,
            name: unique_name,
            map: map.clone(),
        };
        let json = serde_json::to_vec_pretty(&fixture)
            .map_err(|error| invalid_data(format!("could not serialize fixture: {error}")))?;
        if json.len() as u64 > MAX_SPATIAL_FIXTURE_BYTES {
            return Err(invalid_data("fixture exceeds the 8 MiB size limit"));
        }

        let mut file = AtomicWriteFile::open(&path)?;
        file.write_all(&json)?;
        file.commit()?;
        return Ok(path);
    }

    Err(io::Error::other("no unused fixture filename is available"))
}

/// Load and validate a fixture before exposing its map to the live sandbox.
pub fn load_spatial_fixture(path: &Path) -> io::Result<SpatialMapFixture> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(invalid_data("fixture path is not a regular file"));
    }
    if metadata.len() > MAX_SPATIAL_FIXTURE_BYTES {
        return Err(invalid_data("fixture exceeds the 8 MiB size limit"));
    }

    let input = fs::File::open(path)?;
    let mut json = Vec::new();
    input
        .take(MAX_SPATIAL_FIXTURE_BYTES + 1)
        .read_to_end(&mut json)?;
    if json.len() as u64 > MAX_SPATIAL_FIXTURE_BYTES {
        return Err(invalid_data("fixture exceeds the 8 MiB size limit"));
    }
    let fixture: SpatialMapFixture = serde_json::from_slice(&json)
        .map_err(|error| invalid_data(format!("fixture JSON is invalid: {error}")))?;
    if fixture.format_version != SPATIAL_FIXTURE_FORMAT_VERSION {
        return Err(invalid_data(format!(
            "unsupported fixture format version {} (supported: {})",
            fixture.format_version, SPATIAL_FIXTURE_FORMAT_VERSION
        )));
    }
    if fixture.name.trim().is_empty() {
        return Err(invalid_data("fixture name is empty"));
    }
    validate_local_map(&fixture.map).map_err(invalid_data)?;
    Ok(fixture)
}

/// Return valid fixtures and visible diagnostic entries for invalid JSON files.
/// Directories are scanned in order; duplicate paths are included only once.
pub fn list_spatial_fixtures(directories: &[&Path]) -> io::Result<Vec<SpatialFixtureEntry>> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for directory in directories {
        if !directory.exists() {
            continue;
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some(SPATIAL_FIXTURE_EXTENSION)
            {
                continue;
            }
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }

    let mut fixtures = paths
        .into_iter()
        .map(|path| match load_spatial_fixture(&path) {
            Ok(fixture) => SpatialFixtureEntry {
                path,
                name: fixture.name,
                details: format!(
                    "{} × {} · seed {} · {} props · {} discoveries · {} passages · {} actor placements",
                    fixture.map.width,
                    fixture.map.height,
                    fixture.map.seed,
                    fixture.map.props.len(),
                    fixture.map.points_of_interest.len(),
                    fixture.map.exits.len(),
                    fixture.map.interactables.len()
                ),
                map: Some(fixture.map),
                error: None,
            },
            Err(error) => {
                let fallback_name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Invalid fixture")
                    .to_string();
                SpatialFixtureEntry {
                    path,
                    name: fallback_name,
                    details: format!("Invalid fixture: {error}"),
                    map: None,
                    error: Some(error.to_string()),
                }
            }
        })
        .collect::<Vec<_>>();
    fixtures.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(fixtures)
}

fn validate_local_map(map: &LocalMap) -> Result<(), String> {
    if map.location_id.trim().is_empty() {
        return Err("map location ID is empty".to_string());
    }
    if !(MIN_MAP_DIMENSION..=MAX_MAP_DIMENSION).contains(&map.width)
        || !(MIN_MAP_DIMENSION..=MAX_MAP_DIMENSION).contains(&map.height)
    {
        return Err(format!(
            "map dimensions {} × {} are outside the supported 8–128 range",
            map.width, map.height
        ));
    }

    let tile_count = usize::from(map.width) * usize::from(map.height);
    if map.terrain.len() != tile_count {
        return Err(format!(
            "terrain has {} cells; {} × {} requires {tile_count}",
            map.terrain.len(),
            map.width,
            map.height
        ));
    }
    if !map.discovered_tiles.is_empty() && map.discovered_tiles.len() != tile_count {
        return Err(format!(
            "discovery data has {} cells; expected either zero or {tile_count}",
            map.discovered_tiles.len()
        ));
    }
    if !map.contains(map.entry) {
        return Err("entry position is outside the map".to_string());
    }
    if !map.terrain_at(map.entry).is_some_and(Terrain::is_walkable) {
        return Err("entry position is not on walkable terrain".to_string());
    }

    let mut reserved = HashSet::<GridPosition>::new();
    reserved.insert(map.entry);
    let mut poi_ids = HashSet::new();
    for poi in &map.points_of_interest {
        if poi.id.trim().is_empty() || !poi_ids.insert(poi.id.as_str()) {
            return Err("point-of-interest IDs must be non-empty and unique".to_string());
        }
        if !map.contains(poi.position)
            || !map
                .terrain_at(poi.position)
                .is_some_and(Terrain::is_walkable)
        {
            return Err(format!(
                "point of interest {} is outside the map or not walkable",
                poi.id
            ));
        }
        if !reserved.insert(poi.position) {
            return Err(format!(
                "point of interest {} overlaps a reserved cell",
                poi.id
            ));
        }
    }

    let source_location_id = map
        .location_id
        .strip_prefix("location.")
        .and_then(|id| id.parse::<u64>().ok());
    let mut exit_ids = HashSet::new();
    let mut exit_destinations = HashSet::new();
    for exit in &map.exits {
        if exit.id.trim().is_empty() || !exit_ids.insert(exit.id.as_str()) {
            return Err("passage IDs must be non-empty and unique".to_string());
        }
        if exit.target_location_id == 0
            || source_location_id == Some(exit.target_location_id)
            || !exit_destinations.insert(exit.target_location_id)
        {
            return Err(
                "passage destinations must be non-zero, unique, and different from the source location"
                    .to_string(),
            );
        }
        if source_location_id.is_none() {
            return Err("passages require a numeric location.<id> map identity".to_string());
        }
        if !map.contains(exit.position)
            || !map
                .terrain_at(exit.position)
                .is_some_and(Terrain::is_walkable)
        {
            return Err(format!(
                "passage {} is outside the map or not on walkable terrain",
                exit.id
            ));
        }
        if !reserved.insert(exit.position) {
            return Err(format!("passage {} overlaps a reserved cell", exit.id));
        }
    }

    let mut actor_ids = HashSet::new();
    for actor in &map.interactables {
        if actor.entity_id == 0 || !actor_ids.insert(actor.entity_id) {
            return Err("actor placements must have non-zero unique entity IDs".to_string());
        }
        if !map.contains(actor.position)
            || !map
                .terrain_at(actor.position)
                .is_some_and(Terrain::is_walkable)
        {
            return Err(format!(
                "actor {} is outside the map or not on walkable terrain",
                actor.entity_id
            ));
        }
        if !reserved.insert(actor.position) {
            return Err(format!(
                "actor {} overlaps a reserved cell",
                actor.entity_id
            ));
        }
    }

    for change in &map.terrain_changes {
        if !map.contains(change.position) {
            return Err("terrain change is outside the map".to_string());
        }
    }

    let mut prop_ids = HashSet::new();
    let mut prop_cells = HashSet::new();
    for prop in &map.props {
        if prop.id.trim().is_empty() || !prop_ids.insert(prop.id.as_str()) {
            return Err("prop IDs must be non-empty and unique".to_string());
        }
        for offset in 0..prop.kind.footprint_width() {
            let Some(x) = prop.position.x.checked_add(offset) else {
                return Err(format!("prop {} has an invalid footprint", prop.id));
            };
            let cell = GridPosition {
                x,
                y: prop.position.y,
            };
            if !map.contains(cell) {
                return Err(format!("prop {} extends outside the map", prop.id));
            }
            if !map.terrain_at(cell).is_some_and(Terrain::is_walkable) {
                return Err(format!("prop {} occupies non-walkable terrain", prop.id));
            }
            if reserved.contains(&cell) || !prop_cells.insert(cell) {
                return Err(format!("prop {} overlaps an occupied cell", prop.id));
            }
        }
    }

    Ok(())
}

fn fixture_file_stem(name: &str) -> String {
    let mut stem = String::new();
    let mut separator = false;
    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() {
            if separator && !stem.is_empty() {
                stem.push('-');
            }
            stem.push(character.to_ascii_lowercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    stem.trim_matches('-').to_string()
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[cfg(test)]
#[path = "../tests/unit/spatial_fixtures.rs"]
mod tests;
