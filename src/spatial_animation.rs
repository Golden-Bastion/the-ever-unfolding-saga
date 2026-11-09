//! Renderer-independent player sprite animation state and sheet contract.
//!
//! The atlas is described by a mod-aware JSON manifest. Each direction has a
//! separate idle row and walk row; frame timing and playback behavior are data.

use serde::Deserialize;

pub(crate) const PLAYER_ANIMATION_SHEET_REFERENCE: &str =
    "spatial/player_oathbound_animated.png";
pub(crate) const PLAYER_ANIMATION_MANIFEST_REFERENCE: &str =
    "spatial/player_oathbound_animated.json";

const MANIFEST_SCHEMA_VERSION: u32 = 1;
const DEFAULT_IDLE_FRAME_DURATION_SECONDS: f32 = 0.25;
const DEFAULT_WALK_FRAME_DURATION_SECONDS: f32 = 0.12;
const MIN_FRAME_DURATION_MILLISECONDS: u32 = 10;
const MAX_FRAME_DURATION_MILLISECONDS: u32 = 10_000;
const MIN_WALK_SPEED_SCALE: f32 = 0.35;
const FRAME_BOUNDARY_EPSILON: f32 = 0.00001;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayerAnimationManifest {
    schema_version: u32,
    frame_width: u32,
    frame_height: u32,
    columns: u32,
    rows: u32,
    directions: DirectionAnimationManifest,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectionAnimationManifest {
    north: DirectionClipsManifest,
    east: DirectionClipsManifest,
    south: DirectionClipsManifest,
    west: DirectionClipsManifest,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectionClipsManifest {
    idle: AnimationClipManifest,
    walk: AnimationClipManifest,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnimationClipManifest {
    row: u32,
    first_column: u32,
    frame_count: u32,
    frame_duration_ms: u32,
    looping: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AnimationClip {
    row: usize,
    first_column: usize,
    frame_count: usize,
    frame_duration_seconds: f32,
    looping: bool,
}

impl AnimationClip {
    const fn new(
        row: usize,
        first_column: usize,
        frame_count: usize,
        frame_duration_seconds: f32,
        looping: bool,
    ) -> Self {
        Self { row, first_column, frame_count, frame_duration_seconds, looping }
    }

    fn from_manifest(
        definition: AnimationClipManifest,
        columns: u32,
        rows: u32,
        name: &str,
    ) -> Result<Self, String> {
        if definition.row >= rows {
            return Err(format!(
                "{name} animation row {} is outside the {rows}-row atlas",
                definition.row
            ));
        }
        if definition.frame_count == 0 {
            return Err(format!("{name} animation must contain at least one frame"));
        }
        let end_column = definition.first_column
            .checked_add(definition.frame_count)
            .ok_or_else(|| format!("{name} animation column range overflows"))?;
        if end_column > columns {
            return Err(format!(
                "{name} animation columns {}..{} exceed the {columns}-column atlas",
                definition.first_column, end_column
            ));
        }
        if !(MIN_FRAME_DURATION_MILLISECONDS..=MAX_FRAME_DURATION_MILLISECONDS)
            .contains(&definition.frame_duration_ms)
        {
            return Err(format!(
                "{name} frame duration must be between {MIN_FRAME_DURATION_MILLISECONDS} and {MAX_FRAME_DURATION_MILLISECONDS} milliseconds"
            ));
        }

        Ok(Self::new(
            definition.row as usize,
            definition.first_column as usize,
            definition.frame_count as usize,
            definition.frame_duration_ms as f32 / 1000.0,
            definition.looping,
        ))
    }

    fn frame_at_elapsed(self, elapsed_seconds: f32) -> usize {
        let position = elapsed_seconds / self.frame_duration_seconds;
        let elapsed_frames = (position + FRAME_BOUNDARY_EPSILON).floor() as usize;
        if self.looping {
            elapsed_frames % self.frame_count
        } else {
            elapsed_frames.min(self.frame_count - 1)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DirectionAnimationClips {
    idle: AnimationClip,
    walk: AnimationClip,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlayerAnimationConfig {
    frame_width: u32,
    frame_height: u32,
    columns: u32,
    rows: u32,
    directions: [DirectionAnimationClips; 4],
}

impl Default for PlayerAnimationConfig {
    fn default() -> Self {
        let direction = |idle_row, walk_row| DirectionAnimationClips {
            idle: AnimationClip::new(idle_row, 0, 4, DEFAULT_IDLE_FRAME_DURATION_SECONDS, true),
            walk: AnimationClip::new(walk_row, 0, 6, DEFAULT_WALK_FRAME_DURATION_SECONDS, true),
        };
        Self {
            frame_width: 32,
            frame_height: 48,
            columns: 6,
            rows: 8,
            directions: [
                direction(0, 1),
                direction(2, 3),
                direction(4, 5),
                direction(6, 7),
            ],
        }
    }
}

impl PlayerAnimationConfig {
    pub(crate) fn from_json(json: &str) -> Result<Self, String> {
        let manifest: PlayerAnimationManifest = serde_json::from_str(json)
            .map_err(|error| format!("could not parse player animation manifest: {error}"))?;
        Self::from_manifest(manifest)
    }

    fn from_manifest(manifest: PlayerAnimationManifest) -> Result<Self, String> {
        if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(format!(
                "unsupported player animation manifest schema {}; expected {}",
                manifest.schema_version, MANIFEST_SCHEMA_VERSION
            ));
        }
        if manifest.frame_width == 0 || manifest.frame_height == 0
            || manifest.columns == 0 || manifest.rows == 0
        {
            return Err("player animation frame and atlas dimensions must be non-zero".to_string());
        }
        manifest.frame_width.checked_mul(manifest.columns)
            .ok_or_else(|| "player animation sheet width overflows".to_string())?;
        manifest.frame_height.checked_mul(manifest.rows)
            .ok_or_else(|| "player animation sheet height overflows".to_string())?;

        let definitions = [
            ("north", manifest.directions.north),
            ("east", manifest.directions.east),
            ("south", manifest.directions.south),
            ("west", manifest.directions.west),
        ];
        let mut rows = Vec::with_capacity(8);
        let mut clips = Vec::with_capacity(4);
        for (direction, definitions) in definitions {
            let idle = AnimationClip::from_manifest(
                definitions.idle, manifest.columns, manifest.rows, &format!("{direction}.idle")
            )?;
            let walk = AnimationClip::from_manifest(
                definitions.walk, manifest.columns, manifest.rows, &format!("{direction}.walk")
            )?;
            rows.push(idle.row);
            rows.push(walk.row);
            clips.push(DirectionAnimationClips { idle, walk });
        }

        let mut unique_rows = rows.clone();
        unique_rows.sort_unstable();
        unique_rows.dedup();
        if unique_rows.len() != rows.len() {
            return Err("each directional idle and walk clip must use its own atlas row".to_string());
        }

        let directions: [DirectionAnimationClips; 4] = clips.try_into()
            .map_err(|_| "player animation manifest must define four directions".to_string())?;
        Ok(Self {
            frame_width: manifest.frame_width,
            frame_height: manifest.frame_height,
            columns: manifest.columns,
            rows: manifest.rows,
            directions,
        })
    }

    pub(crate) const fn frame_width(self) -> u32 { self.frame_width }
    pub(crate) const fn frame_height(self) -> u32 { self.frame_height }
    pub(crate) const fn columns(self) -> u32 { self.columns }
    pub(crate) const fn rows(self) -> u32 { self.rows }

    pub(crate) const fn sheet_dimensions(self) -> (u32, u32) {
        (self.frame_width * self.columns, self.frame_height * self.rows)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerFacing { North, East, South, West }

impl PlayerFacing {
    const fn index(self) -> usize {
        match self {
            Self::North => 0,
            Self::East => 1,
            Self::South => 2,
            Self::West => 3,
        }
    }

    fn from_screen_direction(x: f32, y: f32, previous: Self) -> Option<Self> {
        if !x.is_finite() || !y.is_finite() { return None; }
        let horizontal = x.abs();
        let vertical = y.abs();
        if horizontal == 0.0 && vertical == 0.0 { return None; }
        if horizontal > vertical {
            return Some(if x < 0.0 { Self::West } else { Self::East });
        }
        if vertical > horizontal {
            return Some(if y > 0.0 { Self::North } else { Self::South });
        }

        // Four-direction sheets have no diagonal pose. Preserve the previous
        // axis when a joystick direction is exactly diagonal to avoid jitter.
        let previous_matches_horizontal = match previous {
            Self::West => x < 0.0,
            Self::East => x > 0.0,
            _ => false,
        };
        let previous_matches_vertical = match previous {
            Self::North => y > 0.0,
            Self::South => y < 0.0,
            _ => false,
        };
        if previous_matches_horizontal || previous_matches_vertical {
            return Some(previous);
        }
        Some(if x < 0.0 { Self::West } else { Self::East })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlayerAnimation {
    facing: PlayerFacing,
    moving: bool,
    elapsed_seconds: f32,
    speed_scale: f32,
    config: PlayerAnimationConfig,
}

impl Default for PlayerAnimation {
    fn default() -> Self { Self::new(PlayerAnimationConfig::default()) }
}

impl PlayerAnimation {
    pub(crate) fn new(config: PlayerAnimationConfig) -> Self {
        Self {
            facing: PlayerFacing::South,
            moving: false,
            elapsed_seconds: 0.0,
            speed_scale: 1.0,
            config,
        }
    }

    /// Update screen-facing direction and whether resolved movement is occurring.
    /// speed_scale is the normalized movement-input magnitude (0.0 to 1.0).
    pub(crate) fn set_motion(&mut self, x: f32, y: f32, moving: bool, speed_scale: f32) {
        if let Some(facing) = PlayerFacing::from_screen_direction(x, y, self.facing) {
            if facing != self.facing {
                self.facing = facing;
                self.elapsed_seconds = 0.0;
            }
        }
        if self.moving != moving {
            self.moving = moving;
            self.elapsed_seconds = 0.0;
        }
        self.speed_scale = if self.moving && speed_scale.is_finite() {
            speed_scale.clamp(MIN_WALK_SPEED_SCALE, 1.0)
        } else {
            1.0
        };
    }

    /// Return to the last facing direction's idle cycle.
    pub(crate) fn stop(&mut self) {
        if self.moving {
            self.moving = false;
            self.elapsed_seconds = 0.0;
        }
        self.speed_scale = 1.0;
    }

    /// Advance the active idle or walk clip using elapsed wall-clock time.
    pub(crate) fn advance(&mut self, delta_seconds: f32) -> usize {
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        let clip = self.active_clip();
        if delta_seconds > 0.0 {
            let speed_scale = if self.moving { self.speed_scale } else { 1.0 };
            self.elapsed_seconds += delta_seconds * speed_scale;
            if !self.elapsed_seconds.is_finite() { self.elapsed_seconds = 0.0; }
            let cycle_duration = clip.frame_duration_seconds * clip.frame_count as f32;
            if clip.looping {
                self.elapsed_seconds = self.elapsed_seconds.rem_euclid(cycle_duration);
            } else {
                self.elapsed_seconds = self.elapsed_seconds.min(cycle_duration);
            }
        }
        self.frame_index()
    }

    /// Return the current row-major atlas frame without advancing playback.
    pub(crate) fn frame_index(&self) -> usize {
        let clip = self.active_clip();
        clip.row * self.config.columns as usize
            + clip.first_column
            + clip.frame_at_elapsed(self.elapsed_seconds)
    }

    fn active_clip(&self) -> AnimationClip {
        let direction = self.config.directions[self.facing.index()];
        if self.moving { direction.walk } else { direction.idle }
    }

    #[cfg(test)]
    fn facing(&self) -> PlayerFacing { self.facing }
}

#[cfg(test)]
#[path = "../tests/unit/spatial_animation.rs"]
mod tests;
