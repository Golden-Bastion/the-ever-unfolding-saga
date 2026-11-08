use super::*;

#[test]
fn character_tabs_have_a_stable_five_tab_order() {
    assert_eq!(CharacterTab::ALL.map(CharacterTab::index), [0, 1, 2, 3, 4]);
    assert_eq!(CharacterTab::from_index(0), Some(CharacterTab::General));
    assert_eq!(CharacterTab::from_index(4), Some(CharacterTab::Journal));
    assert_eq!(CharacterTab::from_index(5), None);
    assert_eq!(CharacterTab::Journal.next(), CharacterTab::General);
}

#[test]
fn closing_records_restores_spatial_context() {
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Character),
        return_screen: Some(ScreenId::SpatialExploration),
        selected: 2,
    };

    close_to_return_screen(&mut navigation);

    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.return_screen, None);
    assert_eq!(navigation.selected, 0);
}

#[test]
fn journal_does_not_repeat_the_note_as_a_chronicle_entry() {
    let notes = vec!["I should remember this.".to_string()];

    assert!(is_personal_note_history_entry(
        "Ada noted: I should remember this.",
        "Ada",
        &notes
    ));
    assert!(!is_personal_note_history_entry(
        "Ada noted: a different note.",
        "Ada",
        &notes
    ));
    assert!(!is_personal_note_history_entry(
        "Ada found a sealed letter.",
        "Ada",
        &notes
    ));
}

#[test]
fn character_tab_back_action_follows_tab_content() {
    assert_eq!(character_tab_back_index(CharacterTab::General, 3, 2, 4), 0);
    assert_eq!(
        character_tab_selection_count(CharacterTab::Reputation, 3, 2, 4),
        1
    );
    assert_eq!(
        character_tab_back_index(CharacterTab::Inventory, 3, 2, 4),
        3
    );
    assert_eq!(
        character_tab_selection_count(CharacterTab::Quests, 3, 2, 4),
        3
    );
    assert_eq!(character_tab_back_index(CharacterTab::Journal, 3, 2, 4), 5);
    assert_eq!(
        character_tab_selection_count(CharacterTab::Journal, 3, 2, 4),
        6
    );
}
