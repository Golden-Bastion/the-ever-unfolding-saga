//! Renderer-independent player sprite animation state and sheet contract.
//!
//! The atlas is row-major: north, east, south, west. Each directional row contains
//! one idle frame followed by four walk-cycle frames. The Bevy renderer owns only
//! texture loading and atlas selection; this module owns facing and clip playback.

pub(crate) const PLAYER_ANIMATION_FRAME_WIDTH: u32 = 16;
pub(crate) const PLAYER_ANIMATION_FRAME_HEIGHT: u32 = 24;
pub(crate) const PLAYER_ANIMATION_COLUMNS: u32 = 5;
pub(crate) const PLAYER_ANIMATION_ROWS: u32 = 4;
pub(crate) const PLAYER_ANIMATION_SHEET_WIDTH: u32 = 80;
pub(crate) const PLAYER_ANIMATION_SHEET_HEIGHT: u32 = 96;
pub(crate) const PLAYER_ANIMATION_SHEET_REFERENCE: &str = "spatial/player_ash_warden_animated.png";

const DEFAULT_IDLE_FRAME_DURATION_SECONDS: f32 = 0.16;
const DEFAULT_WALK_FRAME_DURATION_SECONDS: f32 = 0.12;
const MIN_FRAME_DURATION_SECONDS: f32 = 0.01;
const MAX_FRAME_DURATION_SECONDS: f32 = 10.0;
const MIN_WALK_SPEED_SCALE: f32 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq)]
struct AnimationClip {
    first_frame: usize,
    frame_count: usize,
    frame_duration_seconds: f32,
    looping: bool,
}

impl AnimationClip {
    const fn new(
        first_frame: usize,
        frame_count: usize,
        frame_duration_seconds: f32,
        looping: bool,
    ) -> Self {
        Self {
            first_frame,
            frame_count,
            frame_duration_seconds,
            looping,
        }
    }

    fn normalized(self) -> Self {
        let first_frame = self.first_frame.min(PLAYER_ANIMATION_COLUMNS as usize - 1);
        let available_frames = PLAYER_ANIMATION_COLUMNS as usize - first_frame;
        let frame_count = self.frame_count.clamp(1, available_frames);
        let frame_duration_seconds =
            if self.frame_duration_seconds.is_finite() && self.frame_duration_seconds > 0.0 {
                self.frame_duration_seconds
                    .clamp(MIN_FRAME_DURATION_SECONDS, MAX_FRAME_DURATION_SECONDS)
            } else {
                MIN_FRAME_DURATION_SECONDS
            };

        Self {
            first_frame,
            frame_count,
            frame_duration_seconds,
            looping: self.looping,
        }
    }
}

const PLAYER_IDLE_CLIP: AnimationClip =
    AnimationClip::new(0, 1, DEFAULT_IDLE_FRAME_DURATION_SECONDS, false);
const PLAYER_WALK_CLIP: AnimationClip =
    AnimationClip::new(1, 4, DEFAULT_WALK_FRAME_DURATION_SECONDS, true);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerFacing {
    North,
    East,
    South,
    West,
}

impl PlayerFacing {
    const fn atlas_row(self) -> usize {
        match self {
            Self::North => 0,
            Self::East => 1,
            Self::South => 2,
            Self::West => 3,
        }
    }

    fn from_screen_direction(x: f32, y: f32, previous: Self) -> Option<Self> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }

        let horizontal = x.abs();
        let vertical = y.abs();
        if horizontal == 0.0 && vertical == 0.0 {
            return None;
        }

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
    idle_clip: AnimationClip,
    walk_clip: AnimationClip,
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self::new(PLAYER_IDLE_CLIP, PLAYER_WALK_CLIP)
    }
}

impl PlayerAnimation {
    fn new(idle_clip: AnimationClip, walk_clip: AnimationClip) -> Self {
        Self {
            // The existing static Ash Warden sprite is the forward-facing pose.
            facing: PlayerFacing::South,
            moving: false,
            elapsed_seconds: 0.0,
            speed_scale: 1.0,
            idle_clip: idle_clip.normalized(),
            walk_clip: walk_clip.normalized(),
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

        if self.moving {
            self.speed_scale = if speed_scale.is_finite() {
                speed_scale.clamp(MIN_WALK_SPEED_SCALE, 1.0)
            } else {
                1.0
            };
        } else {
            self.speed_scale = 1.0;
            self.elapsed_seconds = 0.0;
        }
    }

    /// Return to the last facing direction's idle frame.
    pub(crate) fn stop(&mut self) {
        self.moving = false;
        self.elapsed_seconds = 0.0;
        self.speed_scale = 1.0;
    }

    /// Advance by elapsed wall-clock time and return the active atlas frame index.
    pub(crate) fn advance(&mut self, delta_seconds: f32) -> usize {
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };

        if self.moving && delta_seconds > 0.0 {
            self.elapsed_seconds += delta_seconds * self.speed_scale;
            if !self.elapsed_seconds.is_finite() {
                self.elapsed_seconds = 0.0;
            }

            let cycle_duration =
                self.walk_clip.frame_duration_seconds * self.walk_clip.frame_count as f32;
            if self.walk_clip.looping {
                self.elapsed_seconds = self.elapsed_seconds.rem_euclid(cycle_duration);
            } else {
                self.elapsed_seconds = self.elapsed_seconds.min(cycle_duration);
            }
        }

        self.frame_index()
    }

    /// Return the current row-major atlas index without advancing playback.
    pub(crate) fn frame_index(&self) -> usize {
        let clip = if self.moving {
            self.walk_clip
        } else {
            self.idle_clip
        };
        let elapsed_frames = (self.elapsed_seconds / clip.frame_duration_seconds).floor() as usize;
        let clip_frame = if clip.looping {
            elapsed_frames % clip.frame_count
        } else {
            elapsed_frames.min(clip.frame_count - 1)
        };

        self.facing.atlas_row() * PLAYER_ANIMATION_COLUMNS as usize + clip.first_frame + clip_frame
    }

    #[cfg(test)]
    fn facing(&self) -> PlayerFacing {
        self.facing
    }
}

#[cfg(test)]
#[path = "../tests/unit/spatial_animation.rs"]
mod tests;
