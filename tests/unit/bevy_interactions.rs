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


#[test]
fn portrait_layers_are_composited_in_stable_draw_order() {
    let portrait = CharacterPortraitView {
        layers: vec![
            CharacterPortraitLayerView { id: "body".to_string(), slot: "body".to_string(), asset: "body.png".to_string(), layer: 3 },
            CharacterPortraitLayerView { id: "hair-z".to_string(), slot: "hair".to_string(), asset: "hair-z.png".to_string(), layer: 1 },
            CharacterPortraitLayerView { id: "face-b".to_string(), slot: "face".to_string(), asset: "face-b.png".to_string(), layer: 1 },
            CharacterPortraitLayerView { id: "face-a".to_string(), slot: "face".to_string(), asset: "face-a.png".to_string(), layer: 1 },
        ],
    };

    let ordered = ordered_portrait_layers(&portrait)
        .into_iter()
        .map(|layer| layer.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ordered, vec!["face-a", "face-b", "hair-z", "body"]);
}

#[test]
fn closing_spatial_dialogue_keeps_movement_suspended_during_transition() {
    let mut state = BevyInteractionState::default();
    let mut navigation = NavigationState::default();
    state.spatial_overlay = true;

    close_spatial_overlay(&mut state, &mut navigation);

    assert!(state.spatial_overlay_closing);
    assert!(spatial_overlay_open(&state));
    assert!(!state.dirty);
}
