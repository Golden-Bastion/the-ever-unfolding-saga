#[test]
fn start_menu_indices_account_for_developer_tools_and_load_slot() {
    let mut lifecycle = super::LifecycleState::default();
    assert_eq!(super::start_option_count(&lifecycle), 4);
    assert_eq!(super::start_developer_tools_index(&lifecycle), 1);
    assert_eq!(super::start_options_index(&lifecycle), 2);

    lifecycle
        .save_files
        .push(std::path::PathBuf::from("save.json"));
    assert_eq!(super::start_option_count(&lifecycle), 5);
    assert_eq!(super::start_developer_tools_index(&lifecycle), 2);
    assert_eq!(super::start_options_index(&lifecycle), 3);
}

use super::*;

#[test]
fn display_version_uses_package_version_when_no_suffix_is_supplied() {
    assert_eq!(format_display_version("0.52.3", None), "v0.52.3");
    assert_eq!(format_display_version("0.52.3", Some("  ")), "v0.52.3");
}

#[test]
fn display_version_appends_custom_suffix_with_a_single_separator() {
    assert_eq!(
        format_display_version("0.52.3", Some("custom-preview")),
        "v0.52.3-custom-preview"
    );
    assert_eq!(
        format_display_version("0.52.3", Some("-custom-preview")),
        "v0.52.3-custom-preview"
    );
}

#[test]
fn quit_confirmation_can_return_to_gameplay() {
    let mut lifecycle = LifecycleState::default();
    let state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".to_string(),
        "Ash Walker".to_string(),
    );
    lifecycle.session = Some(GameSession {
        state,
        save_path: PathBuf::from("saves/test.json.gz"),
    });
    lifecycle.phase = LifecyclePhase::QuitConfirm;
    lifecycle.selected = 1;

    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Lifecycle),
        return_screen: Some(ScreenId::SpatialExploration),
        selected: 1,
    };

    lifecycle.cancel_quit_confirmation(&mut navigation);

    assert_eq!(lifecycle.phase, LifecyclePhase::Complete);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.return_screen, None);
    assert_eq!(lifecycle.selected, 0);
    assert!(!lifecycle.dirty);
}

#[test]
fn quit_confirmation_can_return_to_spatial_exploration() {
    let mut lifecycle = LifecycleState::default();
    let state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".to_string(),
        "The Oathbound".to_string(),
    );
    lifecycle.session = Some(GameSession {
        state,
        save_path: PathBuf::from("saves/test.json.gz"),
    });
    lifecycle.phase = LifecyclePhase::QuitConfirm;
    lifecycle.selected = 1;

    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Lifecycle),
        return_screen: Some(ScreenId::SpatialExploration),
        selected: 1,
    };

    lifecycle.cancel_quit_confirmation(&mut navigation);

    assert_eq!(lifecycle.phase, LifecyclePhase::Complete);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.return_screen, None);
    assert_eq!(lifecycle.selected, 0);
    assert!(!lifecycle.dirty);
}

#[test]
fn options_activation_keeps_navigation_on_the_options_screen() {
    let mut lifecycle = LifecycleState::default();
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Lifecycle),
        return_screen: None,
        selected: 0,
    };

    lifecycle.phase = LifecyclePhase::Start;
    lifecycle.selected = 2;

    super::activate_selection(&mut lifecycle, &mut navigation);

    assert_eq!(navigation.current_screen, Some(ScreenId::Options));
    assert_eq!(navigation.return_screen, Some(ScreenId::Lifecycle));
}

#[test]
fn developer_tools_activation_opens_without_starting_a_campaign() {
    let mut lifecycle = LifecycleState::default();
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Lifecycle),
        return_screen: None,
        selected: 0,
    };

    lifecycle.phase = LifecyclePhase::Start;
    lifecycle.selected = super::start_developer_tools_index(&lifecycle);

    let transition = super::activate_selection(&mut lifecycle, &mut navigation);

    assert_eq!(transition, None);
    assert_eq!(lifecycle.phase, LifecyclePhase::Start);
    assert_eq!(navigation.current_screen, Some(ScreenId::DeveloperTools));
    assert_eq!(navigation.return_screen, Some(ScreenId::Lifecycle));
}

#[test]
fn load_back_returns_to_the_start_screen() {
    let mut lifecycle = LifecycleState {
        phase: LifecyclePhase::Load,
        ..LifecycleState::default()
    };
    lifecycle
        .save_files
        .push(std::path::PathBuf::from("saves/test.json.gz"));
    lifecycle.selected = lifecycle.save_files.len();
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Lifecycle),
        return_screen: None,
        selected: lifecycle.selected,
    };

    super::activate_selection(&mut lifecycle, &mut navigation);

    assert_eq!(lifecycle.phase, LifecyclePhase::Start);
    assert_eq!(lifecycle.selected, 0);
    assert_eq!(navigation.current_screen, Some(ScreenId::Lifecycle));
    assert_eq!(navigation.selected, 0);
}

#[test]
fn lifecycle_renderer_allows_initial_screen_but_not_dedicated_screen_overwrite() {
    assert!(super::should_render_lifecycle(None));
    assert!(super::should_render_lifecycle(Some(ScreenId::Lifecycle)));
    assert!(!super::should_render_lifecycle(Some(ScreenId::Options)));
    assert!(!super::should_render_lifecycle(Some(
        ScreenId::SpatialExploration
    )));
}

#[test]
fn start_screen_has_expected_options_without_saves() {
    let state = LifecycleState::default();
    assert_eq!(start_option_count(&state), 4);
    assert_eq!(start_developer_tools_index(&state), 1);
    assert_eq!(start_options_index(&state), 2);
    assert!(!start_has_load(&state));
}

#[test]
fn character_save_paths_use_the_dedicated_saves_directory() {
    let saves_dir = PathBuf::from("saves");
    let path = character_save_path(&saves_dir, "Ash Walker");
    assert_eq!(path.parent(), Some(saves_dir.as_path()));
    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some("ever_unfolding_saga_save_Ash Walker.json.gz")
    );
}

#[test]
fn android_text_snapshots_replace_the_buffer_and_reset_the_cursor() {
    let mut field = EditableText::new("The Ashen Crown");

    field.queue_edit(TextEdit::Insert("stale pending edit".into()));
    super::replace_text_snapshot(&mut field, "Málaga 🜏");

    assert_eq!(field.value().to_string(), "Málaga 🜏");
    assert_eq!(field.pending_edits.len(), 1);
    assert_eq!(field.pending_edits[0], TextEdit::TextEnd(false));

    super::replace_text_snapshot(&mut field, "Short");

    assert_eq!(field.value().to_string(), "Short");
    assert_eq!(field.pending_edits.len(), 1);
    assert_eq!(field.pending_edits[0], TextEdit::TextEnd(false));
}

#[test]
fn default_creation_values_match_legacy_flow() {
    let state = LifecycleState::default();
    assert_eq!(state.world_name, "The Ashen Crown");
    assert_eq!(state.creation_tab, 0);
    assert!(state.character_name.is_empty());
    assert_eq!(state.character_title, "The Oathbound");
}

#[test]
fn creation_tabs_have_independent_selection_ranges() {
    assert_eq!(creation_selection_bounds(0), (0, 0));
    assert_eq!(creation_selection_bounds(1), (1, 2));
    assert_eq!(creation_selection_bounds(2), (3, 6));
    assert_eq!(creation_selection_bounds(3), (7, 7));
    assert_eq!(creation_tab_for_field(0), 0);
    assert_eq!(creation_tab_for_field(1), 1);
    assert_eq!(creation_tab_for_field(2), 1);
}

#[test]
fn selecting_a_creation_tab_updates_the_active_selection() {
    let mut lifecycle = LifecycleState {
        phase: LifecyclePhase::CreateCharacter,
        selected: 2,
        dirty: false,
        ..LifecycleState::default()
    };
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Lifecycle),
        selected: 2,
        ..NavigationState::default()
    };

    select_creation_tab(&mut lifecycle, &mut navigation, 2);

    assert_eq!(lifecycle.creation_tab, 2);
    assert_eq!(lifecycle.selected, 3);
    assert_eq!(navigation.selected, 3);
    assert!(lifecycle.dirty);
}

#[test]
fn creation_confirmation_advances_through_world_character_and_start_tabs() {
    let mut lifecycle = LifecycleState {
        phase: LifecyclePhase::CreateCharacter,
        ..LifecycleState::default()
    };
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Lifecycle),
        ..NavigationState::default()
    };

    activate_selection(&mut lifecycle, &mut navigation);
    assert_eq!(lifecycle.creation_tab, 1);
    assert_eq!(lifecycle.selected, 1);

    activate_selection(&mut lifecycle, &mut navigation);
    assert_eq!(lifecycle.creation_tab, 1);
    assert_eq!(lifecycle.selected, 2);

    activate_selection(&mut lifecycle, &mut navigation);
    assert_eq!(lifecycle.creation_tab, 2);
    assert_eq!(lifecycle.selected, 3);
}

#[test]
fn empty_creation_values_keep_the_existing_effective_defaults() {
    let mut lifecycle = LifecycleState::default();
    assert_eq!(lifecycle.effective_world_name(), "The Ashen Crown");
    assert!(lifecycle.character_name.is_empty());
    assert_eq!(lifecycle.effective_character_name(), "Elias");
    assert_eq!(lifecycle.effective_character_title(), "The Oathbound");

    lifecycle.world_name = "  My World  ".to_string();
    lifecycle.character_name = "  My Character  ".to_string();
    lifecycle.character_title = "  My Title  ".to_string();
    assert_eq!(lifecycle.effective_world_name(), "My World");
    assert_eq!(lifecycle.effective_character_name(), "My Character");
    assert_eq!(lifecycle.effective_character_title(), "My Title");
}

#[test]
fn lifecycle_buttons_reference_existing_ui_icon_assets() {
    let mut icons = vec![SAVE_SLOT_ICON, BEGIN_LIFE_ICON];
    icons.extend(
        start_action_visuals(true)
            .iter()
            .chain(start_action_visuals(false).iter())
            .map(|(icon, _)| *icon),
    );
    icons.extend(
        quit_confirmation_visuals(&QuitVariant {
            question: "Question",
            leave: "Leave",
            stay: "Stay",
            art: "",
        })
        .iter()
        .map(|(icon, _)| *icon),
    );
    icons.extend(death_action_visuals().iter().map(|(icon, _)| *icon));

    for icon in icons {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/assets/icons/ui")
            .join(format!("{icon}.png"));
        assert!(
            path.is_file(),
            "missing bundled UI icon asset: {}",
            path.display()
        );
    }
}

#[test]
fn creation_field_focus_is_explicit_and_stops_at_the_review_tab() {
    assert!(should_focus_creation_field(0, None));
    assert!(should_focus_creation_field(1, Some(0)));
    assert!(!should_focus_creation_field(1, Some(1)));
    assert!(!should_focus_creation_field(3, None));
    assert!(!should_focus_creation_field(7, None));
}

#[test]
fn appearance_creation_uses_two_controls_per_slot_and_persists_the_selection() {
    assert_eq!(creation_selection_bounds(2, 0), (3, 3));
    assert_eq!(creation_selection_bounds(2, 5), (3, 12));
    assert_eq!(creation_selection_bounds(3, 5), (7, 7));

    let mut lifecycle = LifecycleState::default();
    let mut navigation = NavigationState::default();
    lifecycle.start_new_game(&mut navigation);
    let slot_count = lifecycle.appearance_slots.len();
    assert!(slot_count > 0);
    assert_eq!(
        creation_selection_bounds(2, slot_count),
        (3, 3 + slot_count * 2 - 1)
    );

    select_creation_tab(&mut lifecycle, &mut navigation, 2);
    let initial_part = lifecycle.appearance_slots[0].parts[0].id.clone();
    let part_count = lifecycle.appearance_slots[0].parts.len();
    cycle_appearance_part(&mut lifecycle, 1);
    if part_count > 1 {
        assert_ne!(
            lifecycle.appearance_slots[0].parts[lifecycle.appearance_slots[0].selected_part].id,
            initial_part
        );
    }

    let selected = selected_appearance_part_ids(&lifecycle);
    assert_eq!(selected.len(), slot_count);

    select_creation_tab(&mut lifecycle, &mut navigation, 3);
    assert_eq!(lifecycle.selected, 7);
    let mut state = crate::model::create_new_state(
        "Appearance Test",
        crate::model::WorldMode::New,
        "Wanderer".into(),
        "Ash Walker".into(),
    );
    apply_selected_character_appearance(&lifecycle, &mut state);
    assert_eq!(state.character.appearance_parts, selected);
}

#[test]
fn lifecycle_complete_action_reflects_pending_save_confirmation() {
    assert_eq!(complete_action_visuals(true), ("explore", "Continue"));
    assert_eq!(complete_action_visuals(false), ("quit", "Quit"));
}

#[test]
fn appearance_buttons_change_the_part_for_the_slot_the_row_represents() {
    use crate::input::InputEvent;

    fn activate_button(
        lifecycle: &mut LifecycleState,
        navigation: &mut NavigationState,
        index: usize,
    ) {
        let mut queue = Vec::new();
        crate::bevy_presentation::queue_choice_activation(&mut queue, index);
        for event in queue {
            match event {
                InputEvent::SetSelection(selected) => {
                    super::set_selection(lifecycle, navigation, selected);
                }
                InputEvent::Confirm => {
                    let _ = super::activate_selection(lifecycle, navigation);
                }
                other => panic!("unexpected appearance button event: {other:?}"),
            }
        }
    }

    let mut lifecycle = LifecycleState::default();
    let mut navigation = NavigationState::default();
    lifecycle.start_new_game(&mut navigation);
    select_creation_tab(&mut lifecycle, &mut navigation, 2);

    assert!(lifecycle.appearance_slots.len() > 1);
    let part_slot = lifecycle
        .appearance_slots
        .iter()
        .position(|slot| slot.parts.len() > 1)
        .expect("the bundled appearance catalogue has at least one multi-part slot");
    lifecycle.selected_appearance_slot = part_slot;
    let part_count = lifecycle.appearance_slots[part_slot].parts.len();

    let other_slot = (part_slot + 1) % lifecycle.appearance_slots.len();
    let other_part_before = lifecycle.appearance_slots[other_slot].selected_part;

    activate_button(&mut lifecycle, &mut navigation, 3 + part_slot * 2);
    assert_eq!(lifecycle.selected_appearance_slot, part_slot);
    assert_eq!(
        lifecycle.appearance_slots[part_slot].selected_part,
        part_count - 1,
        "The left arrow must wrap to the previous part in its own slot"
    );
    assert_eq!(
        lifecycle.appearance_slots[other_slot].selected_part, other_part_before,
        "Changing one layer must not change another layer"
    );

    activate_button(&mut lifecycle, &mut navigation, 4 + part_slot * 2);
    assert_eq!(lifecycle.selected_appearance_slot, part_slot);
    assert_eq!(
        lifecycle.appearance_slots[part_slot].selected_part, 0,
        "The right arrow must advance within the same slot"
    );

    let other_count = lifecycle.appearance_slots[other_slot].parts.len();
    activate_button(&mut lifecycle, &mut navigation, 3 + other_slot * 2);
    assert_eq!(lifecycle.selected_appearance_slot, other_slot);
    if other_count > 1 {
        assert_eq!(
            lifecycle.appearance_slots[other_slot].selected_part,
            other_count - 1,
            "The other slot must respond to its own left arrow"
        );
    }
}
