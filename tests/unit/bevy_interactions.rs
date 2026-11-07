use super::*;

#[test]
fn interaction_screen_set_matches_navigation_targets() {
    assert!(is_interaction_screen(Some(ScreenId::Talk)));
    assert!(is_interaction_screen(Some(ScreenId::Conversation)));
    assert!(is_interaction_screen(Some(ScreenId::RemainsResult)));
    assert!(!is_interaction_screen(Some(ScreenId::Gameplay)));
    assert!(!is_interaction_screen(Some(ScreenId::SpatialExploration)));
}

#[test]
fn spatial_interaction_overlay_keeps_the_exploration_screen_active() {
    let mut state = BevyInteractionState::default();
    assert!(!spatial_overlay_open(&state));
    state.spatial_overlay = true;

    assert!(spatial_overlay_open(&state));
    assert!(!is_interaction_screen(Some(ScreenId::SpatialExploration)));
}
