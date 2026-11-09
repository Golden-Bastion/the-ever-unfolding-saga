use super::*;

#[test]
fn developer_console_shortcut_is_available_from_gameplay_and_spatial_exploration() {
    assert!(console_shortcut_screen_active(Some(ScreenId::Gameplay)));
    assert!(console_shortcut_screen_active(Some(ScreenId::SpatialExploration)));
    assert!(!console_shortcut_screen_active(Some(ScreenId::Character)));
    assert!(!console_shortcut_screen_active(None));
}
