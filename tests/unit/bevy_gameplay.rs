use super::*;

#[test]
fn gameplay_runtime_pause_entry_and_exit_update_runtime_state() {
    let mut gameplay = GameplayMenuState::default();
    let mut runtime = GameplayRuntimeState::default();
    let mut navigation = NavigationState::default();

    open_pause(&mut gameplay, &mut runtime, &mut navigation);
    assert_eq!(gameplay.screen, GameplayMenuScreen::Pause);
    assert!(runtime.is_paused());

    close_pause(
        &mut gameplay,
        &mut runtime,
        &mut navigation,
    );
    assert_eq!(gameplay.screen, GameplayMenuScreen::Closed);
    assert!(!runtime.is_paused());
}

#[test]
fn spatial_pause_menu_keeps_spatial_exploration_as_its_parent() {
    let mut gameplay = GameplayMenuState::default();
    let mut runtime = GameplayRuntimeState::default();
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::SpatialExploration),
        ..default()
    };
    open_pause(&mut gameplay, &mut runtime, &mut navigation);
    assert_eq!(gameplay.screen, GameplayMenuScreen::Pause);
    assert!(runtime.is_paused());
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert!(accepts_gameplay_input(
        navigation.current_screen,
        gameplay.screen
    ));
}

#[test]
fn closing_spatial_pause_menu_restores_the_spatial_screen_and_resumes_play() {
    let mut gameplay = GameplayMenuState::default();
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
    );
    assert_eq!(gameplay.screen, GameplayMenuScreen::Closed);
    assert!(!runtime.is_paused());
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
}

#[test]
fn gameplay_input_accepts_only_spatial_navigation_overlays() {
    assert!(accepts_gameplay_input(
        Some(ScreenId::SpatialExploration),
        GameplayMenuScreen::Pause
    ));
    assert!(accepts_gameplay_input(
        Some(ScreenId::SpatialExploration),
        GameplayMenuScreen::WorldMap
    ));
    assert!(!accepts_gameplay_input(
        Some(ScreenId::SpatialExploration),
        GameplayMenuScreen::Closed
    ));
    assert!(!accepts_gameplay_input(
        Some(ScreenId::Character),
        GameplayMenuScreen::Pause
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
                fast_travel_index: None,
                fast_travel_steps: None,
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
                fast_travel_index: Some(0),
                fast_travel_steps: Some(1),
            },
            crate::presentation::WorldMapNodeView {
                location: crate::presentation::LocationView {
                    id: 12,
                    name: "Cinder Hamlet".to_string(),
                    region_name: "Ashen Vale".to_string(),
                    ..Default::default()
                },
                region_id: 1,
                explored: true,
                current: false,
                reachable: true,
                fast_travel_index: Some(1),
                fast_travel_steps: Some(1),
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
                fast_travel_index: None,
                fast_travel_steps: None,
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
        fast_travel_destinations: vec![
            crate::presentation::WorldMapDestinationView {
                location: crate::presentation::LocationView {
                    id: 11,
                    name: "Old Shrine".to_string(),
                    region_name: "Ashen Vale".to_string(),
                    ..Default::default()
                },
                route_steps: 1,
            },
            crate::presentation::WorldMapDestinationView {
                location: crate::presentation::LocationView {
                    id: 12,
                    name: "Cinder Hamlet".to_string(),
                    region_name: "Ashen Vale".to_string(),
                    ..Default::default()
                },
                route_steps: 1,
            },
        ],
    };

    let first = build_world_map_layout(&view);
    let second = build_world_map_layout(&view);

    assert_eq!(first, second);
    assert_eq!(
        first
            .nodes
            .iter()
            .find(|node| node.location_id == 10)
            .unwrap()
            .column,
        0
    );
    assert_eq!(
        first
            .nodes
            .iter()
            .find(|node| node.location_id == 11)
            .unwrap()
            .column,
        1
    );
    assert_eq!(
        first
            .nodes
            .iter()
            .find(|node| node.location_id == 12)
            .unwrap()
            .column,
        1
    );
    assert_eq!(
        first
            .nodes
            .iter()
            .find(|node| node.location_id == 13)
            .unwrap()
            .column,
        2
    );
    assert_ne!(
        first
            .nodes
            .iter()
            .find(|node| node.location_id == 11)
            .unwrap()
            .top,
        first
            .nodes
            .iter()
            .find(|node| node.location_id == 12)
            .unwrap()
            .top
    );
}

#[test]
fn spatial_pause_menu_exposes_the_world_map_action() {
    assert_eq!(pause_action_visuals(PauseAction::Map), ("travel", "Map"));
}

#[test]
fn pause_action_count_matches_load_visibility_and_safe_meditation() {
    assert_eq!(pause_actions_from(false, false).len(), 6);
    assert_eq!(pause_actions_from(true, false).len(), 7);
    assert_eq!(pause_actions_from(true, true).len(), 8);
}

#[test]
fn pause_action_order_keeps_options_before_quit() {
    assert!(matches!(
        pause_actions_from(false, false).as_slice(),
        [
            PauseAction::Resume,
            PauseAction::Map,
            PauseAction::Character,
            PauseAction::NewGame,
            PauseAction::Options,
            PauseAction::Quit
        ]
    ));
}

#[test]
fn spatial_pause_menu_places_travel_and_safe_meditation_before_character_actions() {
    assert!(matches!(
        pause_actions_from(true, true).as_slice(),
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
fn meditation_is_hidden_when_unsafe() {
    assert!(!pause_actions_from(true, false).contains(&PauseAction::Meditate));
}
