use std::fs;
use std::io::{self, Write};

use atomic_write_file::AtomicWriteFile;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const SETTINGS_FILE_NAME: &str = "audio_settings.json";
const MIN_VOLUME_PERCENT: u8 = 0;
const MAX_VOLUME_PERCENT: u8 = 100;
pub(crate) const VOLUME_STEP: i16 = 5;

#[derive(Resource, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(crate) struct AudioSettings {
    music_volume_percent: u8,
    sfx_volume_percent: u8,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume_percent: 70,
            sfx_volume_percent: 80,
        }
    }
}

impl AudioSettings {
    pub(crate) fn load() -> Self {
        match fs::read(settings_path()) {
            Ok(data) => match serde_json::from_slice::<Self>(&data) {
                Ok(mut settings) => {
                    settings.normalize();
                    settings
                }
                Err(error) => {
                    warn!("Could not read audio settings; using defaults: {error}");
                    Self::default()
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                warn!("Could not load audio settings; using defaults: {error}");
                Self::default()
            }
        }
    }

    pub(crate) fn save(&self) -> io::Result<()> {
        let data = serde_json::to_vec_pretty(self).map_err(|error| {
            io::Error::other(format!("could not serialize audio settings: {error}"))
        })?;
        let mut file = AtomicWriteFile::open(settings_path())?;
        file.write_all(&data)?;
        file.commit()
    }

    pub(crate) fn music_volume_percent(self) -> u8 {
        self.music_volume_percent
    }

    pub(crate) fn sfx_volume_percent(self) -> u8 {
        self.sfx_volume_percent
    }

    pub(crate) fn music_gain(self) -> f32 {
        f32::from(self.music_volume_percent) / 100.0
    }

    pub(crate) fn sfx_gain(self) -> f32 {
        f32::from(self.sfx_volume_percent) / 100.0
    }

    pub(crate) fn adjust_music_volume(&mut self, delta: i16) -> bool {
        adjust_volume(&mut self.music_volume_percent, delta)
    }

    pub(crate) fn adjust_sfx_volume(&mut self, delta: i16) -> bool {
        adjust_volume(&mut self.sfx_volume_percent, delta)
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    fn normalize(&mut self) {
        self.music_volume_percent = self
            .music_volume_percent
            .clamp(MIN_VOLUME_PERCENT, MAX_VOLUME_PERCENT);
        self.sfx_volume_percent = self
            .sfx_volume_percent
            .clamp(MIN_VOLUME_PERCENT, MAX_VOLUME_PERCENT);
    }
}

fn adjust_volume(current: &mut u8, delta: i16) -> bool {
    let next = i16::from(*current)
        .saturating_add(delta)
        .clamp(i16::from(MIN_VOLUME_PERCENT), i16::from(MAX_VOLUME_PERCENT)) as u8;
    if next == *current {
        return false;
    }
    *current = next;
    true
}

fn settings_path() -> std::path::PathBuf {
    crate::game_paths::GamePaths::get()
        .config_dir
        .join(SETTINGS_FILE_NAME)
}

pub(crate) fn install(app: &mut App) {
    app.insert_resource(AudioSettings::load());
}

#[cfg(test)]
#[path = "../tests/unit/audio_settings.rs"]
mod tests;
