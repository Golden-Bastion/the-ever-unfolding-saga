use super::*;

#[test]
fn developer_console_long_press_uses_a_deliberate_threshold() {
    assert!(!should_open_developer_console(10.64, 10.0, false));
    assert!(should_open_developer_console(10.66, 10.0, false));
    assert!(!should_open_developer_console(10.80, 10.0, true));
}

#[test]
fn pause_button_gesture_is_available_in_spatial_exploration_only() {
    assert!(menu_button_screen_active(Some(
        crate::bevy_presentation::ScreenId::SpatialExploration
    )));
    assert!(!menu_button_screen_active(Some(
        crate::bevy_presentation::ScreenId::Console
    )));
    assert!(!menu_button_screen_active(None));
}
