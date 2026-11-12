use super::*;

#[test]
fn cancelling_combat_restores_spatial_exploration() {
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::SpatialExploration),
        return_screen: Some(ScreenId::SpatialExploration),
        selected: 0,
    };
    navigation.open_screen(ScreenId::Combat);

    return_from_combat(&mut navigation);

    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.return_screen, None);
    assert_eq!(navigation.selected, 0);
}
