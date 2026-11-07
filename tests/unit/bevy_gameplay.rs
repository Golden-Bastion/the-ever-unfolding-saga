use super::*;

#[test]
fn celestial_art_regions_select_separate_sun_and_moon_panels() {
    let (sun, moon) = celestial_source_rects(UVec2::new(1_672, 941))
        .expect("landscape celestial atlas should be accepted");

    assert!(sun.min.x >= 0.0);
    assert!(sun.max.x < moon.min.x);
    assert!(moon.max.x <= 1_672.0);
    assert!(sun.min.y >= 0.0);
    assert_eq!(sun.min.y, moon.min.y);
    assert_eq!(sun.max.y, moon.max.y);
    assert!(moon.max.y <= 941.0);
}

#[test]
fn celestial_art_regions_reject_non_landscape_images() {
    assert!(celestial_source_rects(UVec2::new(512, 1024)).is_none());
    assert!(celestial_source_rects(UVec2::new(1, 512)).is_none());
}

#[test]
fn sky_visual_uses_dedicated_celestial_art_instead_of_icon_names() {
    assert_eq!(sky_visual(3).celestial, CelestialBody::Sun);
    assert_eq!(sky_visual(10).celestial, CelestialBody::Moon);
}

#[test]
fn gameplay_runtime_pause_entry_and_exit_update_runtime_state() {
    let mut gameplay = GameplayState::default();
    let mut runtime = GameplayRuntimeState::default();
    let mut navigation = NavigationState::default();

    open_pause(&mut gameplay, &mut runtime, &mut navigation);
    assert_eq!(gameplay.screen, GameplayScreen::Pause);
    assert!(runtime.is_paused());

    close_pause(&mut gameplay, &mut runtime, &mut navigation);
    assert_eq!(gameplay.screen, GameplayScreen::Dashboard);
    assert!(!runtime.is_paused());
}

#[test]
fn animate_world_clouds_does_not_move_clouds_while_paused() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(GameplayRuntimeState { paused: true });

    let entity = app
        .world_mut()
        .spawn((
            WorldCloud {
                base_x: 42.0,
                drift: 8.0,
                speed: 1.0,
            },
            Node {
                left: percent(17.0),
                ..default()
            },
        ))
        .id();

    app.add_systems(Update, animate_world_clouds);
    app.update();

    let node = app
        .world()
        .get::<Node>(entity)
        .expect("cloud node should remain");
    assert_eq!(node.left, percent(17.0));
}

#[test]
fn pause_action_count_matches_load_visibility() {
    assert_eq!(pause_actions_from(false).len(), 5);
    assert_eq!(pause_actions_from(true).len(), 6);
}

#[test]
fn pause_action_order_keeps_options_before_quit() {
    assert!(matches!(
        pause_actions_from(true).as_slice(),
        [
            PauseAction::Resume,
            PauseAction::Character,
            PauseAction::NewGame,
            PauseAction::LoadGame,
            PauseAction::Options,
            PauseAction::Quit
        ]
    ));
}

#[test]
fn gameplay_dashboard_uses_exploration_as_the_main_activity_entry() {
    let state = crate::model::create_new_state(
        "Test World",
        crate::model::WorldMode::New,
        "Ash".to_string(),
        "Wanderer".to_string(),
    );
    let actions = dashboard_actions(&state);

    assert_eq!(
        &actions[..PRIMARY_GAMEPLAY_ACTIONS.len()],
        &PRIMARY_GAMEPLAY_ACTIONS
    );
    assert_eq!(
        actions.len(),
        PRIMARY_GAMEPLAY_ACTIONS.len() + usize::from(state.threat.active)
    );
}
