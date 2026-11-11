use super::*;

#[test]
fn developer_console_shortcut_is_available_from_spatial_exploration() {
    assert!(console_shortcut_screen_active(Some(
        ScreenId::SpatialExploration
    )));
    assert!(!console_shortcut_screen_active(Some(ScreenId::Lifecycle)));
    assert!(!console_shortcut_screen_active(Some(ScreenId::Character)));
    assert!(!console_shortcut_screen_active(None));
}
