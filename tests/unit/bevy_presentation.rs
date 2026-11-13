use super::*;

#[test]
fn touch_scroll_requires_a_deliberate_drag() {
    assert!(!touch_scroll_dragged(Vec2::ZERO, Vec2::new(12.0, 0.0)));
    assert!(!touch_scroll_dragged(Vec2::ZERO, Vec2::new(8.0, 8.0)));
    assert!(touch_scroll_dragged(Vec2::ZERO, Vec2::new(12.1, 0.0)));
}

#[test]
fn touch_coordinates_convert_from_logical_to_physical_space() {
    let position = physical_touch_position(Vec2::new(100.0, 50.0), 2.5);
    assert_eq!(position, Vec2::new(250.0, 125.0));
}

fn spawn_console_input_regression_fixture(mut commands: Commands) {
    let panel = commands.spawn_empty().id();
    spawn_label(&mut commands, panel, "> help");
    spawn_console_input_row(&mut commands, panel, "> ");
}

#[test]
fn console_history_prefix_does_not_create_an_extra_text_input() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(spawn_console_input_regression_fixture);
    schedule.run(&mut world);

    let input_count = {
        let mut inputs = world.query_filtered::<Entity, With<ContextualConsoleInput>>();
        inputs.iter(&world).count()
    };
    let tab_count = {
        let mut tabs = world.query_filtered::<Entity, With<ContextualConsoleTab>>();
        tabs.iter(&world).count()
    };
    let command_echo_is_plain_text = {
        let mut labels = world.query::<(&Text, Option<&ContextualConsoleInput>)>();
        labels
            .iter(&world)
            .any(|(text, input)| text.as_str() == "> help" && input.is_none())
    };

    assert_eq!(input_count, 1);
    assert_eq!(tab_count, 1);
    assert!(command_echo_is_plain_text);
}

#[test]
fn screen_header_row_keeps_the_detail_right_aligned_without_wrapping() {
    let row = screen_header_row_node();
    assert_eq!(row.flex_direction, FlexDirection::Row);
    assert_eq!(row.flex_wrap, FlexWrap::NoWrap);
    assert_eq!(row.justify_content, JustifyContent::SpaceBetween);
    assert_eq!(row.align_items, AlignItems::Center);
}

#[test]
fn standalone_labels_retain_intrinsic_height() {
    let node = standalone_label_node();

    assert_eq!(node.min_height, Val::Auto);
    assert_eq!(node.flex_shrink, 0.0);
}

#[derive(Resource)]
struct ButtonGroupingFixture {
    canvas: Entity,
    positioned: Entity,
    absolute_stacked: Entity,
    ordinary: Entity,
}

fn spawn_button_grouping_fixture(mut commands: Commands) {
    let canvas = commands.spawn_empty().id();

    let positioned = spawn_positioned_action_button_with_detail(
        &mut commands,
        canvas,
        0,
        "travel",
        "Old Shrine",
        "FAST TRAVEL | 1 route step",
    );
    commands.entity(positioned).insert((
        UiManualActivation,
        UiWarmButton,
        Node {
            position_type: PositionType::Absolute,
            left: px(120.0),
            top: px(84.0),
            width: px(204.0),
            height: px(84.0),
            ..default()
        },
    ));

    // The layout organizer must also protect absolute buttons created through
    // the generic helper, in case a caller assigns absolute positioning later.
    let absolute_stacked = spawn_action_button_with_detail(
        &mut commands,
        canvas,
        1,
        "travel",
        "Cinder Hamlet",
        "FAST TRAVEL | 1 route step",
    );
    commands.entity(absolute_stacked).insert((
        UiManualActivation,
        Node {
            position_type: PositionType::Absolute,
            left: px(428.0),
            top: px(84.0),
            width: px(204.0),
            height: px(84.0),
            ..default()
        },
    ));

    let ordinary = spawn_action_button_with_detail(
        &mut commands,
        canvas,
        2,
        "character",
        "Ordinary menu action",
        "This belongs in the vertical action group",
    );

    commands.insert_resource(ButtonGroupingFixture {
        canvas,
        positioned,
        absolute_stacked,
        ordinary,
    });
}

#[test]
fn action_grouping_preserves_canvas_positioned_map_destinations() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems((spawn_button_grouping_fixture, organize_action_button_groups).chain());
    schedule.run(&mut world);
    world.flush();

    let fixture = world.resource::<ButtonGroupingFixture>();
    let canvas_children = world
        .get::<Children>(fixture.canvas)
        .expect("the canvas should retain its children")
        .iter()
        .collect::<Vec<_>>();

    assert!(canvas_children.contains(&fixture.positioned));
    assert!(canvas_children.contains(&fixture.absolute_stacked));
    assert!(!canvas_children.contains(&fixture.ordinary));

    assert!(world
        .get::<UiManualActivation>(fixture.positioned)
        .is_some());
    assert!(world.get::<UiWarmButton>(fixture.positioned).is_some());
    assert!(world
        .get::<UiStackedActionButton>(fixture.positioned)
        .is_none());
    assert_eq!(
        world.get::<Node>(fixture.positioned).unwrap().left,
        px(120.0)
    );

    // An absolute node is never pulled into the menu group, even if it carries
    // the legacy stacked-button marker.
    assert!(world
        .get::<UiStackedActionButton>(fixture.absolute_stacked)
        .is_some());
    assert_eq!(
        world.get::<Node>(fixture.absolute_stacked).unwrap().left,
        px(428.0)
    );

    let groups = canvas_children
        .iter()
        .filter(|entity| world.get::<UiActionButtonGroup>(**entity).is_some())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(groups.len(), 1);
    assert!(world
        .get::<Children>(groups[0])
        .unwrap()
        .iter()
        .any(|entity| entity == fixture.ordinary));
}

#[test]
fn warm_map_buttons_have_a_visible_resting_state_and_distinct_interaction_feedback() {
    assert_eq!(
        button_visual_colors(false, false, false, true),
        (THEME_ACCENT, THEME_SELECTED)
    );
    assert_eq!(
        button_visual_colors(false, true, false, true),
        (THEME_ACCENT, THEME_HOVER)
    );
    assert_eq!(
        button_visual_colors(true, true, false, true),
        (THEME_ACCENT, THEME_PRESSED)
    );
    assert_eq!(
        button_visual_colors(false, false, true, true),
        (THEME_ACCENT, THEME_HOVER)
    );
    assert_eq!(
        button_visual_colors(false, false, false, false),
        (THEME_BORDER, THEME_SURFACE)
    );
}

#[test]
fn action_groups_are_bottom_left_and_width_capped() {
    let node = action_button_group_node();

    assert_eq!(node.width, percent(100));
    assert_eq!(node.max_width, px(UI_ACTION_MAX_WIDTH_PX));
    assert_eq!(node.align_self, AlignSelf::FlexStart);
    assert_eq!(node.margin.top, Val::Auto);
}

#[test]
fn stacked_action_buttons_do_not_expand_to_fill_parent() {
    let node = action_button_node();

    assert_eq!(node.flex_grow, 0.0);
    assert_eq!(node.flex_shrink, 0.0);
    assert_eq!(node.min_height, touch_target_size());
}

#[test]
fn option_rows_keep_labels_left_and_controls_right() {
    let label = option_row_label_node();
    let controls = option_row_controls_node();
    let row = option_row_node();
    assert_eq!(label.flex_grow, 1.0);
    assert_eq!(label.align_items, AlignItems::Start);
    assert_eq!(controls.justify_content, JustifyContent::End);
    assert_eq!(row.flex_direction, FlexDirection::Row);
    assert_eq!(row.flex_wrap, FlexWrap::NoWrap);
}

#[test]
fn options_scroll_region_is_separate_from_fixed_panel_and_footer() {
    let panel = fixed_panel_node();
    let list = scrollable_list_node();
    let footer = option_footer_node();
    assert_eq!(panel.flex_grow, 1.0);
    assert_eq!(panel.min_height, px(0));
    assert_eq!(panel.overflow, Overflow::clip());
    assert_eq!(list.flex_grow, 1.0);
    assert_eq!(list.min_height, px(0));
    assert_eq!(list.overflow, Overflow::scroll());
    assert_eq!(footer.flex_shrink, 0.0);
    assert_eq!(footer.flex_direction, FlexDirection::Row);
    assert_eq!(footer.justify_content, JustifyContent::End);
}

#[test]
fn option_rows_create_label_then_right_aligned_controls() {
    #[derive(Resource)]
    struct Fixture(OptionRowSlots);

    fn setup(mut commands: Commands) {
        let parent = commands.spawn_empty().id();
        let row = spawn_option_row(&mut commands, parent);
        commands.insert_resource(Fixture(row));
    }

    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(setup);
    schedule.run(&mut world);

    let slots = world.resource::<Fixture>().0;
    let children = world
        .get::<Children>(slots.row)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(children, vec![slots.label, slots.controls]);
    assert!(world.get::<OptionRowLabelSlot>(slots.label).is_some());
    assert!(world.get::<OptionRowControlsSlot>(slots.controls).is_some());
}

#[test]
fn compact_action_buttons_keep_touch_sized_bounds_without_filling_the_row() {
    #[derive(Resource)]
    struct Fixture(Entity);

    fn setup(mut commands: Commands) {
        let parent = commands.spawn_empty().id();
        let button = spawn_compact_action_button(&mut commands, parent, 7, "−5%");
        commands.insert_resource(Fixture(button));
    }

    let node = compact_action_button_node();
    assert_eq!(node.width, Val::Auto);
    assert_eq!(node.min_width, px(76));
    assert_eq!(node.max_width, px(180));
    assert_eq!(node.min_height, touch_target_size());
    assert_eq!(node.flex_grow, 0.0);
    assert_eq!(node.flex_shrink, 0.0);

    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(setup);
    schedule.run(&mut world);
    let button = world.resource::<Fixture>().0;
    assert_eq!(
        world.get::<UiTouchTargetShape>(button),
        Some(&UiTouchTargetShape::MinimumBounds {
            width: UI_TOUCH_TARGET_PX as u16,
            height: UI_TOUCH_TARGET_PX as u16,
        })
    );
    assert_eq!(world.get::<ChoiceButton>(button).unwrap().index, 7);
}

#[test]
fn icon_action_buttons_use_the_shared_touch_and_semantic_activation_path() {
    #[derive(Resource)]
    struct Fixture(Entity);

    fn setup(mut commands: Commands) {
        let parent = commands.spawn_empty().id();
        let button = spawn_icon_action_button(&mut commands, parent, 13, "back");
        commands.insert_resource(Fixture(button));
    }

    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(setup);
    schedule.run(&mut world);

    let button = world.resource::<Fixture>().0;
    assert!(world.get::<Button>(button).is_some());
    assert!(world.get::<UiActionButton>(button).is_some());
    assert!(world.get::<UiStyledButton>(button).is_some());
    assert!(world.get::<UiStackedActionButton>(button).is_none());
    assert_eq!(world.get::<ChoiceButton>(button).unwrap().index, 13);
    assert_eq!(
        world.get::<UiTouchTargetShape>(button),
        Some(&UiTouchTargetShape::MinimumHeight)
    );

    let mut queue = Vec::new();
    queue_choice_activation(&mut queue, world.get::<ChoiceButton>(button).unwrap().index);
    assert_eq!(
        queue,
        vec![InputEvent::SetSelection(13), InputEvent::Confirm]
    );
}

#[test]
fn choice_activation_selects_the_exact_index_before_confirming() {
    let mut queue = Vec::new();

    queue_choice_activation(&mut queue, 5);

    assert_eq!(
        queue,
        vec![InputEvent::SetSelection(5), InputEvent::Confirm]
    );
}

#[test]
fn gauge_ratio_handles_empty_and_clamps_values() {
    assert_eq!(gauge_ratio(0, 0), 0.0);
    assert_eq!(gauge_ratio(-5, 10), 0.0);
    assert_eq!(gauge_ratio(15, 10), 1.0);
    assert_eq!(gauge_ratio(5, 10), 0.5);
}

#[test]
fn shared_tab_strip_marks_only_the_selected_tab() {
    #[derive(Resource)]
    struct Fixture(Entity);

    fn setup(mut commands: Commands) {
        let parent = commands.spawn_empty().id();
        let strip = spawn_tab_strip(&mut commands, parent, &["World", "Character", "Start"], 1);
        commands.insert_resource(Fixture(strip));
    }

    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(setup);
    schedule.run(&mut world);

    let strip = world.resource::<Fixture>().0;
    let tabs = world
        .get::<Children>(strip)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(tabs.len(), 3);

    for (index, entity) in tabs.into_iter().enumerate() {
        assert_eq!(world.get::<ChoiceButton>(entity).unwrap().index, index);
        assert_eq!(world.get::<UiTabButton>(entity).unwrap().index, index);
        assert_eq!(world.get::<UiSelected>(entity).is_some(), index == 1);
    }
}

#[test]
fn tab_content_host_keeps_a_stable_flexible_layout() {
    let node = tab_content_node();

    assert_eq!(node.width, percent(100));
    assert_eq!(node.min_width, px(0));
    assert_eq!(node.min_height, px(0));
    assert_eq!(node.flex_grow, 1.0);
    assert_eq!(node.flex_shrink, 1.0);
    assert_eq!(node.flex_direction, FlexDirection::Column);
    assert_eq!(node.overflow, Overflow::clip());
}

#[test]
fn lifecycle_text_fields_do_not_request_automatic_input_focus() {
    #[derive(Resource)]
    struct Fixture(Entity);

    fn setup(mut commands: Commands) {
        let parent = commands.spawn_empty().id();
        let field = spawn_text_input_field(&mut commands, parent, 0, "World name", "A World", true);
        commands.insert_resource(Fixture(field));
    }

    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(setup);
    schedule.run(&mut world);

    let field = world.resource::<Fixture>().0;
    assert!(world.get::<EditableText>(field).is_some());
    assert!(world.get::<LifecycleTextField>(field).is_some());
    assert!(world.get::<bevy::input_focus::AutoFocus>(field).is_none());
}

#[test]
fn presentation_transition_scale_is_smooth_and_bounded() {
    assert!((presentation_transition_scale(0.96, 1.0, 0.0) - 0.96).abs() < 0.001);
    assert!((presentation_transition_scale(0.96, 1.0, 0.5) - 0.98).abs() < 0.001);
    assert!((presentation_transition_scale(0.96, 1.0, 1.0) - 1.0).abs() < 0.001);
    assert_eq!(presentation_transition_scale(0.96, 1.0, -1.0), 0.96);
    assert_eq!(presentation_transition_scale(0.96, 1.0, 2.0), 1.0);
}

#[test]
fn dedicated_screen_navigation_restores_the_screen_that_opened_it() {
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::SpatialExploration),
        return_screen: Some(ScreenId::SpatialExploration),
        selected: 4,
    };

    navigation.open_screen(ScreenId::Character);

    assert_eq!(navigation.current_screen, Some(ScreenId::Character));
    assert_eq!(navigation.return_screen, Some(ScreenId::SpatialExploration));
    assert_eq!(navigation.selected, 0);

    assert_eq!(
        navigation.return_to_previous_screen(ScreenId::SpatialExploration),
        ScreenId::SpatialExploration
    );
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.return_screen, None);
}

#[test]
fn dedicated_screen_navigation_uses_its_fallback_without_a_previous_screen() {
    let mut navigation = NavigationState::default();

    navigation.open_screen(ScreenId::Options);
    assert_eq!(navigation.return_screen, None);

    assert_eq!(
        navigation.return_to_previous_screen(ScreenId::SpatialExploration),
        ScreenId::SpatialExploration
    );
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.return_screen, None);
}
