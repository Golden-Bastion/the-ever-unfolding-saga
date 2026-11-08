use super::*;

#[test]
fn dismissing_feedback_preserves_the_parent_screen_return_target() {
    let mut feedback = FeedbackState {
        return_screen: Some(ScreenId::Combat),
        active: true,
        ..default()
    };
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Feedback),
        return_screen: Some(ScreenId::SpatialExploration),
        selected: 2,
    };

    finish_or_advance(&mut feedback, &mut navigation);

    assert!(!feedback.active);
    assert_eq!(navigation.current_screen, Some(ScreenId::Combat));
    assert_eq!(
        navigation.return_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.selected, 0);
}
