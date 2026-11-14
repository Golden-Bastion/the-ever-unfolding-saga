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

fn spawn_combat_shell_fixture(mut commands: Commands) {
    let view = crate::presentation::CombatView::default();
    spawn_combat_screen(&mut commands, &view, 0);
}

fn clear_combat_event_lines(
    mut commands: Commands,
    hosts: Query<Entity, With<CombatEventLines>>,
    children: Query<&Children>,
) {
    for host in &hosts {
        bevy_presentation::clear_content_host(&mut commands, host, &children);
    }
}

#[test]
fn combat_updates_can_replace_log_content_without_rebuilding_the_screen_or_gauges() {
    let mut world = World::new();
    let mut setup = Schedule::default();
    setup.add_systems(spawn_combat_shell_fixture);
    setup.run(&mut world);
    world.flush();

    let root = world
        .query_filtered::<Entity, With<CombatScreenRoot>>()
        .iter(&world)
        .next()
        .expect("combat root should be mounted");
    let event_lines = world
        .query_filtered::<Entity, With<CombatEventLines>>()
        .iter(&world)
        .next()
        .expect("combat log content host should be mounted");
    let event_panel = world
        .query_filtered::<Entity, With<CombatEventsPanel>>()
        .iter(&world)
        .next()
        .expect("combat log scroll panel should be mounted");
    let player_gauge = world
        .query_filtered::<Entity, With<CombatPlayerGauge>>()
        .iter(&world)
        .next()
        .expect("player gauge should be mounted");
    let enemy_gauge = world
        .query_filtered::<Entity, With<CombatEnemyGauge>>()
        .iter(&world)
        .next()
        .expect("enemy gauge should be mounted");

    world
        .entity_mut(event_panel)
        .insert(bevy::ui::ScrollPosition(Vec2::new(0.0, 84.0)));

    let mut clear = Schedule::default();
    clear.add_systems(clear_combat_event_lines);
    clear.run(&mut world);
    world.flush();

    assert!(world.get::<CombatScreenRoot>(root).is_some());
    assert!(world.get::<CombatEventLines>(event_lines).is_some());
    assert!(world.get::<CombatEventsPanel>(event_panel).is_some());
    assert!(world.get::<CombatPlayerGauge>(player_gauge).is_some());
    assert!(world.get::<CombatEnemyGauge>(enemy_gauge).is_some());
    assert_eq!(
        world
            .get::<bevy::ui::ScrollPosition>(event_panel)
            .unwrap()
            .0,
        Vec2::new(0.0, 84.0)
    );
    assert!(world
        .get::<Children>(event_lines)
        .is_none_or(|children| children.is_empty()));
}
