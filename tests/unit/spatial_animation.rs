use super::*;

#[test]
fn sheet_contract_matches_four_rows_of_idle_and_walk_frames() {
    assert_eq!(PLAYER_ANIMATION_FRAME_WIDTH, 16);
    assert_eq!(PLAYER_ANIMATION_FRAME_HEIGHT, 24);
    assert_eq!(PLAYER_ANIMATION_COLUMNS, 5);
    assert_eq!(PLAYER_ANIMATION_ROWS, 4);
    assert_eq!(PLAYER_ANIMATION_SHEET_WIDTH, 80);
    assert_eq!(PLAYER_ANIMATION_SHEET_HEIGHT, 96);
    assert_eq!(
        PLAYER_ANIMATION_SHEET_WIDTH,
        PLAYER_ANIMATION_FRAME_WIDTH * PLAYER_ANIMATION_COLUMNS
    );
    assert_eq!(
        PLAYER_ANIMATION_SHEET_HEIGHT,
        PLAYER_ANIMATION_FRAME_HEIGHT * PLAYER_ANIMATION_ROWS
    );
    assert_eq!(
        PLAYER_ANIMATION_SHEET_REFERENCE,
        "spatial/player_oathbound_animated.png"
    );
}

#[test]
fn default_animation_starts_on_the_south_idle_frame() {
    let animation = PlayerAnimation::default();
    assert_eq!(animation.facing(), PlayerFacing::South);
    assert_eq!(animation.frame_index(), 10);
}

#[test]
fn screen_direction_selects_the_matching_directional_row() {
    for (x, y, facing, idle_frame, walk_frame) in [
        (0.0, 1.0, PlayerFacing::North, 0, 1),
        (1.0, 0.0, PlayerFacing::East, 5, 6),
        (0.0, -1.0, PlayerFacing::South, 10, 11),
        (-1.0, 0.0, PlayerFacing::West, 15, 16),
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
fn stopping_returns_to_the_same_facing_idle_frame() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(-1.0, 0.0, true, 1.0);
    animation.advance(0.24);
    animation.stop();
    assert_eq!(animation.facing(), PlayerFacing::West);
    assert_eq!(animation.frame_index(), 15);
}

#[test]
fn blocked_movement_updates_facing_without_starting_the_walk_cycle() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(-1.0, 0.0, false, 1.0);
    assert_eq!(animation.facing(), PlayerFacing::West);
    assert_eq!(animation.frame_index(), 15);
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
    assert_eq!(sixty.frame_index(), 3);

    sixty.advance(0.12);
    assert_eq!(sixty.frame_index(), 4);
    sixty.advance(0.06);
    assert_eq!(sixty.frame_index(), 1);
}

#[test]
fn lower_movement_input_slows_the_walk_cycle() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(1.0, 0.0, true, 0.5);
    animation.advance(0.12);
    assert_eq!(animation.frame_index(), 6);
    animation.advance(0.12);
    assert_eq!(animation.frame_index(), 7);
}

#[test]
fn configurable_non_looping_clip_holds_its_last_frame() {
    let mut animation = PlayerAnimation::new(
        AnimationClip::new(0, 1, 0.2, false),
        AnimationClip::new(1, 3, 0.25, false),
    );
    animation.set_motion(-1.0, 0.0, true, 1.0);
    animation.advance(4.0);
    assert_eq!(animation.frame_index(), 18);
}

#[test]
fn invalid_frame_deltas_do_not_corrupt_playback() {
    let mut animation = PlayerAnimation::default();
    animation.set_motion(0.0, 1.0, true, 1.0);
    animation.advance(-1.0);
    animation.advance(f32::NAN);
    animation.advance(f32::INFINITY);
    assert_eq!(animation.frame_index(), 1);
}
