use crate::content::EventContent;
use crate::model::GameState;
use atomic_write_file::AtomicWriteFile;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const SAVE_FILE_VERSION: u32 = 3;
pub const SAVE_BASE_NAME: &str = "ever_unfolding_saga_save";
const SAVE_EXTENSION: &str = "json.gz";
const LEGACY_SAVE_FILE_NAME: &str = "ever_unfolding_saga_save.json";
const GENERATED_EVENT_PREFIXES: &[&str] = &["generated.event.", "generated.evolution.event."];

#[derive(Debug, Serialize)]
struct SaveFile<'a> {
    save_file_version: u32,
    game: &'a GameState,
    #[serde(default)]
    runtime_generated_events: Vec<EventContent>,
    #[serde(default)]
    mod_dependencies: Vec<SavedModDependency>,
}

#[derive(Debug, Deserialize)]
struct LoadedSaveFile {
    save_file_version: u32,
    game: GameState,
    #[serde(default)]
    runtime_generated_events: Vec<EventContent>,
    #[serde(default)]
    mod_dependencies: Vec<SavedModDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedModDependency {
    pub id: String,
    pub name: String,
    pub version: String,
}

pub struct LoadedSave {
    pub game: GameState,
    pub mod_dependencies: Vec<SavedModDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveMetadata {
    pub character_display_name: String,
    pub character_level: u32,
    pub world_name: String,
    pub world_day: u32,
    pub save_file_version: u32,
    pub mod_dependency_count: usize,
}

impl SaveMetadata {
    pub fn details_line(&self) -> String {
        let mod_summary = if self.mod_dependency_count == 1 {
            "1 mod".to_string()
        } else {
            format!("{} mods", self.mod_dependency_count)
        };

        format!(
            "Level {} | {} | Day {} | Format v{} | {}",
            self.character_level,
            self.world_name,
            self.world_day,
            self.save_file_version,
            mod_summary,
        )
    }
}

const MAX_SAVE_BYTES: u64 = 8 * 1024 * 1024;

pub fn save_game(path: &Path, state: &GameState) -> io::Result<()> {
    let dependencies: Vec<SavedModDependency> = state
        .campaign_content
        .as_ref()
        .map(|content| {
            content
                .loaded_mods
                .iter()
                .map(|manifest| SavedModDependency {
                    id: manifest.id.clone(),
                    name: manifest.name.clone(),
                    version: manifest.version.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    save_game_with_dependencies(path, state, &dependencies)
}

pub fn save_game_with_dependencies(
    path: &Path,
    state: &GameState,
    mod_dependencies: &[SavedModDependency],
) -> io::Result<()> {
    let json = serialize_save(state, mod_dependencies)?;
    let file = AtomicWriteFile::open(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&json)?;
    let file = encoder.finish()?;
    file.commit()?;
    Ok(())
}

fn serialize_save(
    state: &GameState,
    mod_dependencies: &[SavedModDependency],
) -> io::Result<Vec<u8>> {
    let payload = SaveFile {
        save_file_version: SAVE_FILE_VERSION,
        game: state,
        runtime_generated_events: state
            .campaign_content
            .as_ref()
            .map(|content| {
                content
                    .events
                    .iter()
                    .filter(|event| is_runtime_generated_event(&event.id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default(),
        mod_dependencies: mod_dependencies.to_vec(),
    };
    serde_json::to_vec_pretty(&payload).map_err(|error| io::Error::other(error.to_string()))
}

fn write_save_file_exclusive(
    path: &Path,
    state: &GameState,
    mod_dependencies: &[SavedModDependency],
) -> io::Result<()> {
    let json = serialize_save(state, mod_dependencies)?;
    // create_new is the no-overwrite guarantee, unlike a racy existence check.
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let result = (|| {
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(&json)?;
        let file = encoder.finish()?;
        file.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}

#[cfg(test)]
fn load_game(path: &Path) -> io::Result<GameState> {
    load_game_with_metadata(path).map(|loaded| loaded.game)
}

pub fn read_save_metadata(path: &Path) -> io::Result<SaveMetadata> {
    let LoadedSaveFile {
        save_file_version,
        game,
        mod_dependencies,
        ..
    } = read_save_file(path)?;
    let game = normalize_loaded_game(game, save_file_version);

    Ok(SaveMetadata {
        character_display_name: game.character.display_name(),
        character_level: game.character.level,
        world_name: game.world.name.clone(),
        world_day: game.world.day,
        save_file_version,
        mod_dependency_count: mod_dependencies.len(),
    })
}

pub fn load_game_with_metadata(path: &Path) -> io::Result<LoadedSave> {
    let LoadedSaveFile {
        save_file_version,
        game,
        runtime_generated_events,
        mod_dependencies,
    } = read_save_file(path)?;
    let mut game = normalize_loaded_game(game, save_file_version);

    let mut content = crate::content::load_campaign_content();
    for event in runtime_generated_events {
        if content
            .events
            .iter()
            .all(|candidate| candidate.id != event.id)
        {
            content.events.push(event);
        }
    }
    game.campaign_content = Some(content);
    Ok(LoadedSave {
        game,
        mod_dependencies,
    })
}

fn read_save_file(path: &Path) -> io::Result<LoadedSaveFile> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_SAVE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save file exceeds the maximum supported size",
        ));
    }
    let mut data = Vec::new();
    file.take(MAX_SAVE_BYTES + 1).read_to_end(&mut data)?;
    if data.len() as u64 > MAX_SAVE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save file exceeds the maximum supported size",
        ));
    }
    let json = if is_gzip(&data) {
        let decoder = GzDecoder::new(data.as_slice());
        let mut decoded = Vec::new();
        decoder
            .take(MAX_SAVE_BYTES + 1)
            .read_to_end(&mut decoded)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
        if decoded.len() as u64 > MAX_SAVE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "decompressed save exceeds the maximum supported size",
            ));
        }
        decoded
    } else {
        // Backward compatibility for the pre-compression JSON save format.
        data
    };

    let parsed: LoadedSaveFile = serde_json::from_slice(&json)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    if parsed.save_file_version > SAVE_FILE_VERSION || parsed.save_file_version == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Save file version {} is not supported by this build",
                parsed.save_file_version
            ),
        ));
    }
    Ok(parsed)
}

fn normalize_loaded_game(mut game: GameState, save_file_version: u32) -> GameState {
    if save_file_version < 2 {
        game.world.time_points = 3;
        game.world.day = 1;
        if game.character.level == 0 {
            game.character.level = 1;
        }
        if game.character.attributes.might == 0
            && game.character.attributes.insight == 0
            && game.character.attributes.endurance == 0
        {
            game.character.attributes.might = 1;
            game.character.attributes.insight = 1;
            game.character.attributes.endurance = 1;
        }
    }
    game
}

fn is_runtime_generated_event(event_id: &str) -> bool {
    GENERATED_EVENT_PREFIXES
        .iter()
        .any(|prefix| event_id.starts_with(prefix))
}

pub fn install_imported_save(source: &Path) -> io::Result<PathBuf> {
    let loaded = load_game_with_metadata(source)?;
    let paths = crate::game_paths::GamePaths::get();
    let canonical = character_save_path(&paths.saves_dir, &loaded.game.character.name);
    let safe_name = sanitize_filename_component(&loaded.game.character.name);
    let mut suffix = 0u32;

    loop {
        let target = if suffix == 0 {
            canonical.clone()
        } else {
            paths.saves_dir.join(format!(
                "{}_{}_imported_{}.{}",
                SAVE_BASE_NAME, safe_name, suffix, SAVE_EXTENSION
            ))
        };

        match write_save_file_exclusive(&target, &loaded.game, &loaded.mod_dependencies) {
            Ok(()) => return Ok(target),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                suffix = suffix.checked_add(1).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "no unused save filename is available",
                    )
                })?;
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn validate_save(path: &Path) -> io::Result<()> {
    load_game_with_metadata(path).map(|_| ())
}

#[cfg(not(target_os = "android"))]
pub fn export_save(source: &Path, destination: &Path) -> io::Result<()> {
    let _ = load_game_with_metadata(source)?;
    let mut input = File::open(source)?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let result = io::copy(&mut input, &mut output).and_then(|_| output.sync_all());
    if let Err(error) = result {
        drop(output);
        let _ = fs::remove_file(destination);
        return Err(error);
    }
    Ok(())
}

pub fn character_save_path(directory: &Path, character_name: &str) -> PathBuf {
    directory.join(format!(
        "{}_{}.{}",
        SAVE_BASE_NAME,
        sanitize_filename_component(character_name),
        SAVE_EXTENSION
    ))
}

pub fn find_save_files(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut saves = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with(&format!("{}_", SAVE_BASE_NAME))
            && file_name.ends_with(&format!(".{}", SAVE_EXTENSION))
        {
            saves.push(path);
        }
    }
    saves.sort();
    Ok(saves)
}

pub fn legacy_save_path(directory: &Path) -> PathBuf {
    directory.join(LEGACY_SAVE_FILE_NAME)
}

pub fn sanitize_filename_component(name: &str) -> String {
    let mut sanitized = String::new();
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ' ') {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    let sanitized = sanitized.trim().trim_matches('.').to_string();
    let has_safe_name_char = sanitized.chars().any(|ch| ch.is_ascii_alphanumeric());
    if sanitized.is_empty() || !has_safe_name_char {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

fn is_gzip(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}

#[cfg(test)]
#[path = "../tests/unit/persistence.rs"]
mod tests;
