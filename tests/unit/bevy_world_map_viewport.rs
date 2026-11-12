use super::*;

#[test]
fn a_destination_tap_activates_on_release_over_the_same_node() {
    let mut world = World::new();
    let destination = world.spawn_empty().id();
    let mut gesture = ActiveWorldMapGesture::begin(Vec2::new(40.0, 60.0), Some(destination));

    assert_eq!(gesture.update(Vec2::new(43.0, 62.0)), Vec2::ZERO);
    assert_eq!(gesture.release_activation(true), Some(destination));
}

#[test]
fn pointer_drag_preserves_the_threshold_crossing_motion_and_cancels_destination_activation() {
    let mut world = World::new();
    let destination = world.spawn_empty().id();
    let mut gesture = ActiveWorldMapGesture::begin(Vec2::ZERO, Some(destination));

    assert_eq!(gesture.update(Vec2::new(5.0, 0.0)), Vec2::ZERO);
    assert_eq!(gesture.update(Vec2::new(13.0, 0.0)), Vec2::new(13.0, 0.0));
    assert_eq!(gesture.update(Vec2::new(20.0, 4.0)), Vec2::new(7.0, 4.0));
    assert_eq!(gesture.release_activation(true), None);
}

#[test]
fn pinch_gesture_cannot_turn_into_a_destination_tap_when_one_finger_remains() {
    let mut world = World::new();
    let destination = world.spawn_empty().id();
    let mut gesture = ActiveWorldMapGesture::begin(Vec2::ZERO, Some(destination));

    // This is the state transition used when the second pointer joins a touch.
    gesture.begin_drag(Vec2::new(10.0, 10.0));
    assert_eq!(gesture.update(Vec2::new(14.0, 11.0)), Vec2::new(4.0, 1.0));
    assert_eq!(gesture.release_activation(true), None);
}
