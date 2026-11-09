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

    close_pause(
        &mut gameplay,
        &mut runtime,
        &mut navigation,
        ScreenId::Gameplay,
    );
    assert_eq!(gameplay.screen, GameplayScreen::Dashboard);
    assert!(!runtime.is_paused());
}

#[test]
fn spatial_pause_menu_keeps_spatial_exploration_as_its_parent() {
    let mut gameplay = GameplayState::default();
    let mut runtime = GameplayRuntimeState::default();
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::SpatialExploration),
        ..default()
    };
    open_pause(&mut gameplay, &mut runtime, &mut navigation);
    assert_eq!(gameplay.screen, GameplayScreen::Pause);
    assert!(runtime.is_paused());
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert!(accepts_gameplay_input(
        navigation.current_screen,
        gameplay.screen
    ));
    assert_eq!(
        pause_parent_screen(navigation.current_screen),
        ScreenId::SpatialExploration
    );
}

#[test]
fn closing_spatial_pause_menu_restores_the_spatial_screen_and_resumes_play() {
    let mut gameplay = GameplayState::default();
    let mut runtime = GameplayRuntimeState::default();
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::SpatialExploration),
        ..default()
    };
    open_pause(&mut gameplay, &mut runtime, &mut navigation);
    close_pause(
        &mut gameplay,
        &mut runtime,
        &mut navigation,
        ScreenId::SpatialExploration,
    );
    assert_eq!(gameplay.screen, GameplayScreen::Dashboard);
    assert!(!runtime.is_paused());
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
}

#[test]
fn gameplay_input_accepts_only_gameplay_and_an_active_spatial_pause_menu() {
    assert!(accepts_gameplay_input(
        Some(ScreenId::Gameplay),
        GameplayScreen::Dashboard
    ));
    assert!(accepts_gameplay_input(
        Some(ScreenId::SpatialExploration),
        GameplayScreen::Pause
    ));
    assert!(accepts_gameplay_input(
        Some(ScreenId::SpatialExploration),
        GameplayScreen::Navigation
    ));
    assert!(!accepts_gameplay_input(
        Some(ScreenId::SpatialExploration),
        GameplayScreen::Dashboard
    ));
    assert!(!accepts_gameplay_input(
        Some(ScreenId::Character),
        GameplayScreen::Pause
    ));
}

#[test]
fn world_map_layout_is_deterministic_and_places_known_routes_by_graph_distance() {
    let view = WorldMapView {
        current_location: Some(crate::presentation::LocationView {
            id: 10,
            name: "Crossroads".to_string(),
            region_name: "Ashen Vale".to_string(),
            ..Default::default()
        }),
        nodes: vec![
            crate::presentation::WorldMapNodeView {
                location: crate::presentation::LocationView {
                    id: 10,
                    name: "Crossroads".to_string(),
                    region_name: "Ashen Vale".to_string(),
                    ..Default::default()
                },
                region_id: 1,
                explored: true,
                current: true,
                reachable: false,
            },
            crate::presentation::WorldMapNodeView {
                location: crate::presentation::LocationView {
                    id: 11,
                    name: "Old Shrine".to_string(),
                    region_name: "Ashen Vale".to_string(),
                    ..Default::default()
                },
                region_id: 1,
                explored: true,
                current: false,
                reachable: true,
            },
            crate::presentation::WorldMapNodeView {
                location: crate::presentation::LocationView {
                    id: 12,
                    name: "Cinder Hamlet".to_string(),
                    region_name: "Ashen Vale".to_string(),
                    ..Default::default()
                },
                region_id: 1,
                explored: false,
                current: false,
                reachable: true,
            },
            crate::presentation::WorldMapNodeView {
                location: crate::presentation::LocationView {
                    id: 13,
                    name: "Sunken Watch".to_string(),
                    region_name: "Ashen Vale".to_string(),
                    ..Default::default()
                },
                region_id: 1,
                explored: false,
                current: false,
                reachable: false,
            },
        ],
        routes: vec![
            crate::presentation::WorldMapRouteView {
                from_id: 10,
                to_id: 11,
                bidirectional: true,
            },
            crate::presentation::WorldMapRouteView {
                from_id: 10,
                to_id: 12,
                bidirectional: true,
            },
            crate::presentation::WorldMapRouteView {
                from_id: 11,
                to_id: 13,
                bidirectional: false,
            },
        ],
        destinations: Vec::new(),
    };

    let first = build_world_map_layout(&view);
    let second = build_world_map_layout(&view);

    assert_eq!(first, second);
    assert_eq!(first.nodes.iter().find(|node| node.location_id == 10).unwrap().column, 0);
    assert_eq!(first.nodes.iter().find(|node| node.location_id == 11).unwrap().column, 1);
    assert_eq!(first.nodes.iter().find(|node| node.location_id == 12).unwrap().column, 1);
    assert_eq!(first.nodes.iter().find(|node| node.location_id == 13).unwrap().column, 2);
    assert_ne!(
        first.nodes.iter().find(|node| node.location_id == 11).unwrap().top,
        first.nodes.iter().find(|node| node.location_id == 12).unwrap().top
    );
}

#[test]
fn spatial_map_action_uses_map_label_and_existing_travel_icon() {
    assert_eq!(pause_action_visuals(PauseAction::Map), ("travel", "Map"));
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
fn pause_action_count_matches_load_visibility_spatial_context_and_safe_meditation() {
    assert_eq!(pause_actions_from(false, false, false).len(), 5);
    assert_eq!(pause_actions_from(true, false, false).len(), 6);
    assert_eq!(pause_actions_from(false, true, false).len(), 6);
    assert_eq!(pause_actions_from(true, true, false).len(), 7);
    assert_eq!(pause_actions_from(false, true, true).len(), 7);
    assert_eq!(pause_actions_from(true, true, true).len(), 8);
}

#[test]
fn pause_action_order_keeps_options_before_quit() {
    assert!(matches!(
        pause_actions_from(true, false, false).as_slice(),
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

#[test]
fn spatial_pause_menu_places_travel_and_safe_meditation_before_character_actions() {
    assert!(matches!(
        pause_actions_from(true, true, true).as_slice(),
        [
            PauseAction::Resume,
            PauseAction::Map,
            PauseAction::Meditate,
            PauseAction::Character,
            PauseAction::NewGame,
            PauseAction::LoadGame,
            PauseAction::Options,
            PauseAction::Quit
        ]
    ));
}

#[test]
fn meditation_is_hidden_when_unsafe_and_outside_spatial_exploration() {
    assert!(!pause_actions_from(false, true, false).contains(&PauseAction::Meditate));
    assert!(!pause_actions_from(false, false, true).contains(&PauseAction::Meditate));
}
