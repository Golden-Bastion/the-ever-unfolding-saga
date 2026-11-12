use super::*;

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
        Some(ScreenId::Character),
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
