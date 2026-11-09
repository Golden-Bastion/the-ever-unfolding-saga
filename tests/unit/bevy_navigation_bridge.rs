use super::*;

#[test]
fn gameplay_reentry_does_not_reset_pause_selection() {
    assert!(!should_reset_gameplay_selection(
        Some(ScreenId::Options),
        Some(ScreenId::Gameplay),
        GameplayScreen::Pause
    ));
}

#[test]
fn gameplay_reentry_resets_selection_for_normal_gameplay() {
    assert!(should_reset_gameplay_selection(
        Some(ScreenId::Character),
        Some(ScreenId::Gameplay),
        GameplayScreen::Dashboard
    ));
}


#[test]
fn completed_new_or_loaded_session_enters_spatial_exploration() {
    assert!(should_enter_spatial_exploration(
        Some(ScreenId::Lifecycle),
        true,
        false,
        None,
    ));
}

#[test]
fn lifecycle_handoff_waits_for_save_warning_confirmation() {
    assert!(!should_enter_spatial_exploration(
        Some(ScreenId::Lifecycle),
        true,
        true,
        None,
    ));
}

#[test]
fn lifecycle_handoff_requires_a_completed_session_on_lifecycle_screen() {
    assert!(!should_enter_spatial_exploration(
        Some(ScreenId::Gameplay),
        true,
        false,
        None,
    ));
    assert!(!should_enter_spatial_exploration(
        Some(ScreenId::Lifecycle),
        false,
        false,
        None,
    ));
    assert!(!should_enter_spatial_exploration(
        Some(ScreenId::Lifecycle),
        true,
        false,
        Some(ScreenId::SpatialExploration),
    ));
}
