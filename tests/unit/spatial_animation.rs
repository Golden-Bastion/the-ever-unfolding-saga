use super::*;

const PLAYER_ANIMATION_MANIFEST: &str =
    include_str!("../../data/assets/spatial/player_oathbound_animated.json");

#[test]
fn manifest_defines_the_approved_frame_grid_and_animation_clips() {
    let config = PlayerAnimationConfig::from_json(PLAYER_ANIMATION_MANIFEST).unwrap();
    assert_eq!(config.frame_width(), 32);
    assert_eq!(config.frame_height(), 48);
    assert_eq!(config.columns(), 6);
    assert_eq!(config.rows(), 8);
    assert_eq!(config.sheet_dimensions(), (192, 384));
    assert_eq!(
        PLAYER_ANIMATION_SHEET_REFERENCE,
        "spatial/player_oathbound_animated.png"
    );
    assert_eq!(
        PLAYER_ANIMATION_MANIFEST_REFERENCE,
        "spatial/player_oathbound_animated.json"
    );

    for (direction_index, idle_row, walk_row) in [(0, 0, 1), (1, 2, 3), (2, 4, 5), (3, 6, 7)] {
        let clips = config.directions[direction_index];
        assert_eq!(clips.idle.row, idle_row);
        assert_eq!(clips.idle.first_column, 0);
        assert_eq!(clips.idle.frame_count, 4);
        assert_eq!(clips.idle.frame_duration_seconds, 0.25);
        assert!(clips.idle.looping);
        assert_eq!(clips.walk.row, walk_row);
        assert_eq!(clips.walk.first_column, 0);
        assert_eq!(clips.walk.frame_count, 6);
        assert_eq!(clips.walk.frame_duration_seconds, 0.12);
        assert!(clips.walk.looping);
    }
}

#[test]
fn default_animation_starts_on_the_south_idle_frame() {
    let animation = PlayerAnimation::default();
    assert_eq!(animation.facing(), PlayerFacing::South);
    assert_eq!(animation.frame_index(), 24);
}

#[test]
fn screen_direction_selects_the_matching_directional_rows() {
    for (x, y, facing, idle_frame, walk_frame) in [
        (0.0, 1.0, PlayerFacing::North, 0, 6),
        (1.0, 0.0, PlayerFacing::East, 12, 18),
        (0.0, -1.0, PlayerFacing::South, 24, 30),
        (-1.0, 0.0, PlayerFacing::West, 36, 42),
    ] {
        let mut animation = PlayerAnimation::default();
        animation.set_motion(x, y, false, 1.0);
        assert_eq!(animation.facing(), facing);
        assert_eq!(animation.frame_index(), idle_frame);
        animation.set_motion(x, y, true, 1.0);
        assert_eq!(animation.frame_index(), walk_frame);
    }
}

#[test]
fn stopping_returns_to_the_same_facing_idle_cycle() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(-1.0, 0.0, true, 1.0);
    animation.advance(0.24);
    animation.stop();
    assert_eq!(animation.facing(), PlayerFacing::West);
    assert_eq!(animation.frame_index(), 36);
}

#[test]
fn blocked_movement_updates_facing_without_starting_the_walk_cycle() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(-1.0, 0.0, false, 1.0);
    assert_eq!(animation.facing(), PlayerFacing::West);
    assert_eq!(animation.frame_index(), 36);
}

#[test]
fn diagonal_input_preserves_an_aligned_previous_facing() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(0.0, 1.0, false, 1.0);
    animation.set_motion(1.0, 1.0, true, 1.0);
    assert_eq!(animation.facing(), PlayerFacing::North);
}

#[test]
fn walk_cycle_is_independent_of_update_frequency_and_loops() {
    let mut sixty = PlayerAnimation::default();
    let mut thirty = PlayerAnimation::default();
    sixty.set_motion(0.0, 1.0, true, 1.0);
    thirty.set_motion(0.0, 1.0, true, 1.0);
    for _ in 0..60 {
        sixty.advance(0.005);
    }
    for _ in 0..30 {
        thirty.advance(0.01);
    }
    assert_eq!(sixty.frame_index(), thirty.frame_index());
    assert_eq!(sixty.frame_index(), 8);
    sixty.advance(0.42);
    assert_eq!(sixty.frame_index(), 6);
}

#[test]
fn idle_cycle_advances_while_stationary_and_loops() {
    let mut animation = PlayerAnimation::default();
    assert_eq!(animation.frame_index(), 24);
    animation.advance(0.25);
    assert_eq!(animation.frame_index(), 25);
    animation.advance(0.75);
    assert_eq!(animation.frame_index(), 24);
}

#[test]
fn idle_cycle_does_not_restart_when_stop_is_called_repeatedly() {
    let mut animation = PlayerAnimation::default();
    animation.advance(0.5);
    assert_eq!(animation.frame_index(), 26);
    animation.stop();
    animation.advance(0.25);
    assert_eq!(animation.frame_index(), 27);
    animation.stop();
    assert_eq!(animation.frame_index(), 27);
}

#[test]
fn lower_movement_input_slows_the_walk_cycle() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(1.0, 0.0, true, 0.5);
    animation.advance(0.12);
    assert_eq!(animation.frame_index(), 18);
    animation.advance(0.12);
    assert_eq!(animation.frame_index(), 19);
}

#[test]
fn configurable_non_looping_clip_holds_its_last_frame() {
    let mut config = PlayerAnimationConfig::default();
    config.directions[3].walk = AnimationClip::new(7, 1, 3, 0.25, false);
    let mut animation = PlayerAnimation::new(config);
    animation.set_motion(-1.0, 0.0, true, 1.0);
    animation.advance(4.0);
    assert_eq!(animation.frame_index(), 45);
}

#[test]
fn invalid_manifest_clips_outside_the_atlas_are_rejected() {
    let invalid = PLAYER_ANIMATION_MANIFEST.replace("\"row\": 7", "\"row\": 8");
    assert!(PlayerAnimationConfig::from_json(&invalid).is_err());
}

#[test]
fn invalid_frame_deltas_do_not_corrupt_playback() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(0.0, 1.0, true, 1.0);
    animation.advance(-1.0);
    animation.advance(f32::NAN);
    animation.advance(f32::INFINITY);
    assert_eq!(animation.frame_index(), 6);
}
