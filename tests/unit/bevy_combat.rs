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

fn spatial_encounter_game(enemy_hp: i32) -> crate::model::GameState {
    let mut game = crate::model::create_new_state(
        "Spatial Combat Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_id = game.character.location_id;
    let mut map = crate::spatial::generate_local_map(
        19,
        format!("location.{location_id}"),
        crate::spatial::LocalMapConfig::default(),
    );
    assert!(crate::spatial::ensure_spatial_encounter(
        &mut map,
        "encounter.combat_test.1".into(),
        "Ash Marauder".into(),
        "A hostile figure blocks the way.".into(),
        enemy_hp,
        0,
        "Marauder Token".into(),
    ));
    let encounter_position = map.encounters[0].position;
    game.character.exploration_session = Some(crate::spatial::ExplorationSession {
        location_id: map.location_id.clone(),
        position: encounter_position,
    });
    game.character.exploration_position = Some(crate::model::ExplorationPosition {
        x: f32::from(encounter_position.x) + 0.5,
        y: f32::from(encounter_position.y) + 0.5,
    });
    game.world.local_maps.push(map);
    game
}

#[test]
fn combat_targets_the_selected_encounter_when_multiple_records_coexist() {
    let mut game = spatial_encounter_game(30);
    let map = &mut game.world.local_maps[0];
    let primary = map.encounters[0].clone();
    assert!(crate::spatial::ensure_spatial_encounter_avoiding(
        map,
        crate::spatial::SpatialEncounterDefinition {
            id: "encounter.combat_test.support".into(),
            quest_id: None,
            spawn_group: Some("support".into()),
            enemy_name: "Cinder Scout".into(),
            description: "A second hostile figure circles nearby.".into(),
            max_hp: 11,
            power: 2,
            trophy_item_name: "Scout Token".into(),
        },
        Some(primary.position),
    ));
    let secondary = map
        .encounters
        .iter()
        .find(|entry| entry.id == "encounter.combat_test.support")
        .unwrap()
        .clone();

    let primary_combat = crate::game::combat::start_spatial_encounter(&mut game, &primary.id)
        .expect("the primary encounter should remain confrontable");
    assert_eq!(primary_combat.enemy_name, primary.enemy_name);
    assert_eq!(primary_combat.enemy_hp, primary.current_hp);

    game.character.exploration_position = Some(crate::model::ExplorationPosition {
        x: f32::from(secondary.position.x) + 0.5,
        y: f32::from(secondary.position.y) + 0.5,
    });
    let secondary_combat = crate::game::combat::start_spatial_encounter(&mut game, &secondary.id)
        .expect("the second encounter should be independently confrontable");
    assert_eq!(secondary_combat.enemy_name, "Cinder Scout");
    assert_eq!(secondary_combat.enemy_hp, 11);
}

#[test]
fn spatial_encounter_cannot_be_started_from_outside_interaction_range() {
    let mut game = spatial_encounter_game(30);
    game.character.exploration_position = Some(crate::model::ExplorationPosition {
        x: f32::from(game.world.local_maps[0].width) - 1.5,
        y: f32::from(game.world.local_maps[0].height) - 1.5,
    });
    assert!(matches!(
        crate::game::combat::start_spatial_encounter(&mut game, "encounter.combat_test.1"),
        Err(crate::game::combat::CombatStartError::SpatialEncounterOutOfReach)
    ));
}

#[test]
fn spatial_encounter_health_survives_combat_retreat_and_reentry() {
    let mut game = spatial_encounter_game(30);
    let mut encounter =
        crate::game::combat::start_spatial_encounter(&mut game, "encounter.combat_test.1")
            .expect("active spatial encounter should start");
    assert_eq!(encounter.enemy_hp, 30);

    let step = crate::game::combat::resolve_action(&mut game, &mut encounter, 0);
    assert!(matches!(step, CombatStep::Continue));
    let saved_hp = game.world.local_maps[0].encounters[0].current_hp;
    assert!(saved_hp < 30);
    assert_eq!(saved_hp, encounter.enemy_hp);

    let mut resumed =
        crate::game::combat::start_spatial_encounter(&mut game, "encounter.combat_test.1")
            .expect("the same spatial encounter should resume");
    assert_eq!(resumed.enemy_hp, saved_hp);
    let fled = crate::game::combat::resolve_action(&mut game, &mut resumed, 2);
    assert!(matches!(
        fled,
        CombatStep::Result {
            outcome: crate::game::combat::CombatResult::Fled,
            ..
        }
    ));

    let after_flee =
        crate::game::combat::start_spatial_encounter(&mut game, "encounter.combat_test.1")
            .expect("fleeing should leave the encounter active");
    assert_eq!(after_flee.enemy_hp, saved_hp);
}

#[test]
fn spatial_encounter_victory_persists_the_defeated_state() {
    let mut game = spatial_encounter_game(1);
    game.threat.activate(
        game.character.location_id,
        "Local threat",
        "The threat is at this location.",
    );
    let mut encounter =
        crate::game::combat::start_spatial_encounter(&mut game, "encounter.combat_test.1")
            .expect("active spatial encounter should start");
    let step = crate::game::combat::resolve_action(&mut game, &mut encounter, 0);
    assert!(matches!(
        step,
        CombatStep::Result {
            outcome: crate::game::combat::CombatResult::Victory,
            ..
        }
    ));

    let record = &game.world.local_maps[0].encounters[0];
    assert_eq!(record.current_hp, 0);
    assert_eq!(
        record.status,
        crate::spatial::SpatialEncounterStatus::Defeated
    );
    assert!(!game.threat.active);
}
#[test]
fn spatial_encounter_victory_does_not_clear_a_threat_at_another_location() {
    let mut game = spatial_encounter_game(1);
    let other_location = game
        .world
        .locations
        .iter()
        .find(|location| location.id != game.character.location_id)
        .expect("world should contain another location")
        .id;
    game.threat.activate(
        other_location,
        "Distant threat",
        "Something remains elsewhere.",
    );
    let mut encounter =
        crate::game::combat::start_spatial_encounter(&mut game, "encounter.combat_test.1")
            .expect("local encounter should not depend on the global threat");
    let step = crate::game::combat::resolve_action(&mut game, &mut encounter, 0);
    assert!(matches!(
        step,
        CombatStep::Result {
            outcome: crate::game::combat::CombatResult::Victory,
            ..
        }
    ));
    assert!(game.threat.active);
    assert_eq!(game.threat.source_location_id, Some(other_location));
}
