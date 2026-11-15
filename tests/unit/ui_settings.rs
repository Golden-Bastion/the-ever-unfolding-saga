use super::*;

#[test]
fn interface_preference_adjustments_respect_limits() {
    let mut settings = UiSettings::default();
    for _ in 0..20 {
        settings.adjust_ui_scale(SETTING_STEP);
        settings.adjust_font_size(-SETTING_STEP);
    }
    assert_eq!(settings.ui_scale_percent(), MAX_UI_SCALE_PERCENT);
    assert_eq!(settings.font_size_percent(), MIN_FONT_SIZE_PERCENT);
    assert!(!settings.adjust_ui_scale(SETTING_STEP));
    assert!(!settings.adjust_font_size(-SETTING_STEP));
}

#[test]
fn extreme_interface_adjustments_saturate_without_overflow() {
    let mut settings = UiSettings::default();

    assert!(settings.adjust_ui_scale(i16::MAX));
    assert_eq!(settings.ui_scale_percent(), MAX_UI_SCALE_PERCENT);
    assert!(settings.adjust_ui_scale(i16::MIN));
    assert_eq!(settings.ui_scale_percent(), MIN_UI_SCALE_PERCENT);

    assert!(settings.adjust_font_size(i16::MAX));
    assert_eq!(settings.font_size_percent(), MAX_FONT_SIZE_PERCENT);
    assert!(settings.adjust_font_size(i16::MIN));
    assert_eq!(settings.font_size_percent(), MIN_FONT_SIZE_PERCENT);
}

#[test]
fn reset_restores_both_interface_preferences() {
    let mut settings = UiSettings {
        ui_scale_percent: 120,
        font_size_percent: 145,
    };
    settings.reset();
    assert_eq!(settings, UiSettings::default());
}

#[test]
fn loaded_preferences_are_clamped_to_supported_ranges() {
    let mut settings = UiSettings {
        ui_scale_percent: 255,
        font_size_percent: 0,
    };
    settings.normalize();
    assert_eq!(settings.ui_scale_percent(), MAX_UI_SCALE_PERCENT);
    assert_eq!(settings.font_size_percent(), MIN_FONT_SIZE_PERCENT);
}

#[test]
fn applying_settings_updates_layout_typography_and_touch_targets() {
    let mut world = World::new();
    let settings = UiSettings {
        ui_scale_percent: 75,
        font_size_percent: 150,
    };
    world.insert_resource(settings);
    world.insert_resource(UiScale(1.0));
    world.insert_resource(RemSize(20.0));
    let window = world
        .spawn((
            Window {
                resolution: (1280, 720).into(),
                ..default()
            },
            bevy::window::PrimaryWindow,
        ))
        .id();

    let layout_node = world.spawn((Node::default(), EmSize(20.0))).id();
    let text_node = world
        .spawn((Node::default(), EmSize(20.0), TextFont::default()))
        .id();
    let height_target = world
        .spawn((
            Node {
                min_height: Val::Px(crate::bevy_presentation::UI_TOUCH_TARGET_PX),
                ..default()
            },
            crate::bevy_presentation::UiTouchTargetShape::MinimumHeight,
        ))
        .id();
    let minimum_bounds_target = world
        .spawn((
            Node {
                min_width: Val::Px(52.0),
                min_height: Val::Px(crate::bevy_presentation::UI_TOUCH_TARGET_PX),
                ..default()
            },
            crate::bevy_presentation::UiTouchTargetShape::MinimumBounds {
                width: 52,
                height: 48,
            },
        ))
        .id();
    let square_target = world
        .spawn((
            Node {
                width: Val::Px(crate::bevy_presentation::UI_TOUCH_TARGET_PX),
                min_width: Val::Px(crate::bevy_presentation::UI_TOUCH_TARGET_PX),
                height: Val::Px(crate::bevy_presentation::UI_TOUCH_TARGET_PX),
                min_height: Val::Px(crate::bevy_presentation::UI_TOUCH_TARGET_PX),
                ..default()
            },
            crate::bevy_presentation::UiTouchTargetShape::Square,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(apply_ui_settings);
    schedule.run(&mut world);

    let base_viewport_em = 720.0 * BASE_REM_VMIN_PERCENT;
    assert!((world.resource::<UiScale>().0 - 0.75).abs() < f32::EPSILON);
    assert!((world.resource::<RemSize>().0 - base_viewport_em * 1.5).abs() < 0.01);
    assert!((world.get::<EmSize>(layout_node).unwrap().0 - base_viewport_em * 0.75).abs() < 0.01);
    assert!((world.get::<EmSize>(text_node).unwrap().0 - 20.0).abs() < 0.01);

    let touch_scale = if cfg!(target_os = "android") {
        0.75
    } else {
        1.0
    };
    let expected_touch_size =
        touch_target_base_size(crate::bevy_presentation::UI_TOUCH_TARGET_PX, touch_scale);
    assert_eq!(
        world.get::<Node>(height_target).unwrap().min_height,
        Val::Px(expected_touch_size)
    );

    let bounds = world.get::<Node>(minimum_bounds_target).unwrap();
    assert_eq!(
        bounds.min_width,
        Val::Px(touch_target_base_size(52.0, touch_scale))
    );
    assert_eq!(bounds.min_height, Val::Px(expected_touch_size));

    let square = world.get::<Node>(square_target).unwrap();
    assert_eq!(square.width, Val::Px(expected_touch_size));
    assert_eq!(square.min_width, Val::Px(expected_touch_size));
    assert_eq!(square.height, Val::Px(expected_touch_size));
    assert_eq!(square.min_height, Val::Px(expected_touch_size));

    for _ in 0..10 {
        assert!(world
            .resource_mut::<UiSettings>()
            .adjust_ui_scale(SETTING_STEP));
    }
    assert!(world
        .resource_mut::<UiSettings>()
        .adjust_font_size(-SETTING_STEP));
    let late_layout_node = world.spawn((Node::default(), EmSize(1.0))).id();
    world.get_mut::<Window>(window).unwrap().resolution = (1920, 1080).into();
    schedule.run(&mut world);

    let resized_viewport_em = 1080.0 * BASE_REM_VMIN_PERCENT;
    assert!((world.resource::<UiScale>().0 - 1.25).abs() < f32::EPSILON);
    assert!((world.resource::<RemSize>().0 - resized_viewport_em * 1.45).abs() < 0.01);
    assert!(
        (world.get::<EmSize>(layout_node).unwrap().0 - resized_viewport_em * 1.25).abs() < 0.01
    );
    assert!(
        (world.get::<EmSize>(late_layout_node).unwrap().0 - resized_viewport_em * 1.25).abs()
            < 0.01
    );
    assert!((world.get::<EmSize>(text_node).unwrap().0 - 20.0).abs() < 0.01);

    let resized_touch_scale = if cfg!(target_os = "android") {
        1.25
    } else {
        1.0
    };
    let resized_touch_size = touch_target_base_size(
        crate::bevy_presentation::UI_TOUCH_TARGET_PX,
        resized_touch_scale,
    );
    assert_eq!(
        world.get::<Node>(height_target).unwrap().min_height,
        Val::Px(resized_touch_size)
    );

    let bounds = world.get::<Node>(minimum_bounds_target).unwrap();
    assert_eq!(
        bounds.min_width,
        Val::Px(touch_target_base_size(52.0, resized_touch_scale))
    );
    assert_eq!(bounds.min_height, Val::Px(resized_touch_size));

    let square = world.get::<Node>(square_target).unwrap();
    assert_eq!(square.width, Val::Px(resized_touch_size));
    assert_eq!(square.min_width, Val::Px(resized_touch_size));
    assert_eq!(square.height, Val::Px(resized_touch_size));
    assert_eq!(square.min_height, Val::Px(resized_touch_size));

    // New UI nodes are initialized even when neither preferences nor the
    // viewport have changed since the previous reconciliation.
    let newly_added_layout = world.spawn((Node::default(), EmSize(1.0))).id();
    let newly_added_touch_target = world
        .spawn((
            Node::default(),
            crate::bevy_presentation::UiTouchTargetShape::MinimumHeight,
        ))
        .id();
    schedule.run(&mut world);

    assert!(
        (world.get::<EmSize>(newly_added_layout).unwrap().0 - resized_viewport_em * 1.25).abs()
            < 0.01
    );
    assert_eq!(
        world
            .get::<Node>(newly_added_touch_target)
            .unwrap()
            .min_height,
        Val::Px(resized_touch_size)
    );
}
