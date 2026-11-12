use super::*;

#[test]
fn a_destination_tap_activates_only_the_node_that_received_the_press() {
    let mut world = World::new();
    let destination = world.spawn_empty().id();

    assert_eq!(
        world_map_destination_activation(Some(destination), false, true),
        Some(destination)
    );
    assert_eq!(
        world_map_destination_activation(Some(destination), false, false),
        None
    );
}

#[test]
fn dragging_or_pinching_over_a_destination_never_activates_fast_travel() {
    let mut world = World::new();
    let destination = world.spawn_empty().id();

    // A drag can finish over the same node where it began; it is still navigation.
    assert_eq!(
        world_map_destination_activation(Some(destination), true, true),
        None
    );

    // The same suppression applies when a second pointer turns a press into a pinch.
    assert_eq!(
        world_map_destination_activation(Some(destination), true, false),
        None
    );
}
