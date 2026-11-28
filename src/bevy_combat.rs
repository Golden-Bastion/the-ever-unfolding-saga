//! Bevy presentation and interaction for combat.
//!
//! Combat resolution remains in `game::combat`; this module only owns the
//! frontend state needed to present a turn and dispatch semantic input.

use bevy::prelude::*;

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::game::combat::{self, CombatEncounter, CombatResult, CombatStep};
use verdandi::prelude::{ImpactMaterial, WeaponHandlingKind, WeaponHandlingMaterial, WeaponImpactKind};
use crate::input::InputEvent;
use crate::presentation::CombatResultView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatScreenPhase {
    Active,
    Result,
}

#[derive(Component)]
struct CombatScreenRoot;

#[derive(Component)]
struct CombatEventLines;

#[derive(Component)]
struct CombatEventsPanel;

#[derive(Component)]
struct CombatPlayerGauge;

#[derive(Component)]
struct CombatEnemyGauge;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum CombatTextKind {
    Summary,
    Condition,
    EnemyName,
    EnemyPower,
    ResultTitle,
    ResultNote,
    Instruction,
}

#[derive(Component)]
struct CombatText(CombatTextKind);

#[derive(Resource, Default)]
pub(crate) struct CombatState {
    encounter: Option<CombatEncounter>,
    result: Option<CombatResultView>,
    outcome: Option<CombatResult>,
    phase: Option<CombatScreenPhase>,
    selected: usize,
    dirty: bool,
}

impl CombatState {
    fn reset(&mut self, encounter: CombatEncounter) {
        self.encounter = Some(encounter);
        self.result = None;
        self.outcome = None;
        self.phase = Some(CombatScreenPhase::Active);
        self.selected = 0;
        self.dirty = true;
    }

    pub(crate) fn is_presenting(&self) -> bool {
        self.encounter.is_some() && self.phase.is_some()
    }

    pub(crate) fn result_outcome(&self) -> Option<CombatResult> {
        if self.phase == Some(CombatScreenPhase::Result) {
            self.outcome
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.encounter = None;
        self.result = None;
        self.outcome = None;
        self.phase = None;
        self.selected = 0;
        self.dirty = true;
    }

    fn action_count(&self) -> usize {
        self.encounter.as_ref().map(|_| 3).unwrap_or(0)
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<CombatState>()
        .add_systems(Update, (combat_input, render_if_active).chain());
}

pub(crate) fn begin_spatial(
    state: &mut crate::model::GameState,
    combat_state: &mut CombatState,
    encounter_id: &str,
) -> Result<(), String> {
    let encounter =
        combat::start_spatial_encounter(state, encounter_id).map_err(|error| error.to_string())?;
    combat_state.reset(encounter);
    Ok(())
}

fn combat_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut combat_state: ResMut<CombatState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
    mut audio_events: ResMut<crate::bevy_audio_events::AudioEventQueue>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || lifecycle.session.is_none()
        || navigation.current_screen != Some(ScreenId::Combat)
    {
        return;
    }
    if input_queue.0.is_empty() {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up | InputEvent::Character('k') => {
                move_selection(&mut combat_state, &mut navigation, -1)
            }
            InputEvent::Down | InputEvent::Character('j') => {
                move_selection(&mut combat_state, &mut navigation, 1)
            }
            InputEvent::Home => set_selection(&mut combat_state, &mut navigation, 0),
            InputEvent::SetSelection(index) => {
                set_selection(&mut combat_state, &mut navigation, index);
            }
            InputEvent::End => {
                let last = combat_state.action_count().saturating_sub(1);
                set_selection(&mut combat_state, &mut navigation, last);
            }
            InputEvent::Character('1') => {
                set_selection(&mut combat_state, &mut navigation, 0);
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation, &mut audio_events);
            }
            InputEvent::Character('2') => {
                set_selection(&mut combat_state, &mut navigation, 1);
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation, &mut audio_events);
            }
            InputEvent::Character('3') => {
                set_selection(&mut combat_state, &mut navigation, 2);
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation, &mut audio_events);
            }
            InputEvent::Confirm => {
                if combat_state.phase == Some(CombatScreenPhase::Active) {
                    let last = combat_state.action_count().saturating_sub(1);
                    combat_state.selected = navigation.selected.min(last);
                }
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation, &mut audio_events);
            }
            InputEvent::Cancel => {
                combat_state.clear();
                return_from_combat(&mut navigation);
            }
            _ => {}
        }
    }
}

fn return_from_combat(navigation: &mut NavigationState) {
    navigation.return_to_previous_screen(ScreenId::SpatialExploration);
}

fn move_selection(
    combat_state: &mut CombatState,
    navigation: &mut NavigationState,
    direction: isize,
) {
    if combat_state.phase != Some(CombatScreenPhase::Active) {
        return;
    }
    let count = combat_state.action_count();
    if count == 0 {
        return;
    }
    combat_state.selected =
        (combat_state.selected as isize + direction).rem_euclid(count as isize) as usize;
    navigation.selected = combat_state.selected;
}

fn set_selection(
    combat_state: &mut CombatState,
    navigation: &mut NavigationState,
    selected: usize,
) {
    if combat_state.phase != Some(CombatScreenPhase::Active) || combat_state.action_count() == 0 {
        return;
    }
    combat_state.selected = selected.min(combat_state.action_count() - 1);
    navigation.selected = combat_state.selected;
}

fn activate_current_selection(
    lifecycle: &mut LifecycleState,
    combat_state: &mut CombatState,
    navigation: &mut NavigationState,
    audio_events: &mut crate::bevy_audio_events::AudioEventQueue,
) {
    match combat_state.phase {
        Some(CombatScreenPhase::Active) => {
            let action = combat_state.selected;
            let (step, player_hp_before, player_hp_after, seed) = {
                let Some(session) = lifecycle.session.as_mut() else {
                    return;
                };
                let Some(encounter) = combat_state.encounter.as_mut() else {
                    return;
                };
                let player_hp_before = session.state.character.hp;
                let seed = session
                    .state
                    .world
                    .generation
                    .map(|generation| generation.seed)
                    .or(session.state.world.geography_seed)
                    .unwrap_or(session.state.rng_state)
                    ^ u64::from(session.state.character.turn);
                let step = combat::resolve_action(&mut session.state, encounter, action);
                (
                    step,
                    player_hp_before,
                    session.state.character.hp,
                    seed,
                )
            };

            match action {
                0 => audio_events.weapon_impact(
                    WeaponImpactKind::Strike,
                    ImpactMaterial::Flesh,
                    0.72,
                    seed ^ 0x4154_5441_434B,
                ),
                1 => audio_events.weapon_impact(
                    WeaponImpactKind::Block,
                    ImpactMaterial::Shield,
                    0.52,
                    seed ^ 0x4755_4152_44,
                ),
                2 => {},
                _ => {}
            }
            if player_hp_after < player_hp_before {
                audio_events.weapon_impact(
                    WeaponImpactKind::Strike,
                    ImpactMaterial::Flesh,
                    0.38,
                    seed ^ 0x4849_5450_4C41_59,
                );
            }
            if matches!(&step, CombatStep::Result { outcome: CombatResult::Victory | CombatResult::Fled, .. }) {
                audio_events.weapon_handling(
                    WeaponHandlingKind::Sheathe,
                    WeaponHandlingMaterial::Metal,
                    seed ^ 0x5348_4541_5448_45,
                );
            }
            apply_step(combat_state, navigation, step);
        }
        Some(CombatScreenPhase::Result) => {
            let defeated = combat_state
                .result
                .as_ref()
                .map(|result| result.result_title == "Defeat")
                .unwrap_or(false);
            combat_state.clear();
            if defeated {
                navigation.return_screen = None;
                lifecycle.phase = LifecyclePhase::Death;
                lifecycle.selected = 0;
                lifecycle.mark_dirty();
            } else {
                return_from_combat(navigation);
            }
        }
        None => {}
    }
}

fn apply_step(combat_state: &mut CombatState, navigation: &mut NavigationState, step: CombatStep) {
    match step {
        CombatStep::Continue => combat_state.dirty = true,
        CombatStep::Result { view, outcome } => {
            combat_state.result = Some(*view);
            combat_state.outcome = Some(outcome);
            combat_state.phase = Some(CombatScreenPhase::Result);
            combat_state.selected = 0;
            navigation.selected = 0;
            combat_state.dirty = true;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    mut combat_state: ResMut<CombatState>,
    navigation: Res<NavigationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
    combat_roots: Query<Entity, With<CombatScreenRoot>>,
    event_hosts: Query<Entity, With<CombatEventLines>>,
    player_gauges: Query<(Entity, &bevy_presentation::UiHealthGauge), With<CombatPlayerGauge>>,
    enemy_gauges: Query<(Entity, &bevy_presentation::UiHealthGauge), With<CombatEnemyGauge>>,
    mut labels: Query<(&CombatText, &mut Text)>,
    children: Query<&Children>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || lifecycle.session.is_none()
        || navigation.current_screen != Some(ScreenId::Combat)
    {
        for root in &combat_roots {
            commands.entity(root).despawn();
        }
        return;
    }
    if !combat_state.dirty {
        return;
    }

    let Some(encounter) = combat_state.encounter.as_ref() else {
        return;
    };
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let view = combat::build_combat_view(&session.state, encounter);

    if combat_roots.iter().next().is_none() {
        for root in &roots {
            commands.entity(root).despawn();
        }
        spawn_combat_screen(&mut commands, &view, combat_state.selected);
    } else {
        update_combat_labels(&mut labels, &view, &combat_state);
        update_combat_gauge(
            &mut commands,
            &player_gauges,
            view.character.hp,
            view.character.max_hp,
        );
        update_combat_gauge(
            &mut commands,
            &enemy_gauges,
            view.enemy.current_hp,
            view.enemy.max_hp,
        );

        if let Some(host) = event_hosts.iter().next() {
            bevy_presentation::clear_content_host(&mut commands, host, &children);
            render_combat_events(&mut commands, host, &view.events);
        }
    }

    combat_state.dirty = false;
}

fn spawn_combat_screen(
    commands: &mut Commands,
    view: &crate::presentation::CombatView,
    selected: usize,
) {
    let root = bevy_presentation::spawn_screen(commands, "COMBAT");
    commands.entity(root).insert(CombatScreenRoot);

    let actors = bevy_presentation::spawn_panel(commands, root);
    spawn_combat_text(
        commands,
        actors,
        CombatTextKind::Summary,
        format!(
            "{} | Turn {} | {}",
            view.character.display_name(),
            view.turn,
            view.location_name
        ),
        false,
    );
    spawn_combat_text(
        commands,
        actors,
        CombatTextKind::Condition,
        condition_text(view),
        true,
    );
    bevy_presentation::spawn_label(commands, actors, "Enemy");
    spawn_combat_text(
        commands,
        actors,
        CombatTextKind::EnemyName,
        view.enemy.name.clone(),
        false,
    );
    let enemy_gauge = bevy_presentation::spawn_health_gauge(
        commands,
        actors,
        view.enemy.current_hp,
        view.enemy.max_hp,
    );
    commands.entity(enemy_gauge).insert(CombatEnemyGauge);
    spawn_combat_text(
        commands,
        actors,
        CombatTextKind::EnemyPower,
        format!("Power: {}", view.enemy_power),
        true,
    );
    bevy_presentation::spawn_label(commands, actors, "You");
    let player_gauge = bevy_presentation::spawn_health_gauge(
        commands,
        actors,
        view.character.hp,
        view.character.max_hp,
    );
    commands.entity(player_gauge).insert(CombatPlayerGauge);

    let events = bevy_presentation::spawn_panel(commands, root);
    commands.entity(events).insert(CombatEventsPanel);
    bevy_presentation::spawn_label(commands, events, "Combat log");
    let event_lines = commands
        .spawn((
            CombatEventLines,
            Node {
                width: percent(100),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .id();
    commands.entity(events).add_child(event_lines);
    render_combat_events(commands, event_lines, &view.events);

    let actions = bevy_presentation::spawn_panel(commands, root);
    for (index, action) in view.actions.iter().enumerate() {
        let icon = match index {
            0 => "attack",
            1 => "insight",
            _ => "evade",
        };
        let button = bevy_presentation::spawn_action_button(
            commands,
            actions,
            index,
            icon,
            format!("{}: {}", index + 1, action),
        );
        if index == selected {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }
    spawn_combat_text(
        commands,
        actions,
        CombatTextKind::ResultTitle,
        String::new(),
        false,
    );
    spawn_combat_text(
        commands,
        actions,
        CombatTextKind::ResultNote,
        String::new(),
        true,
    );
    spawn_combat_text(
        commands,
        actions,
        CombatTextKind::Instruction,
        instruction_text(false),
        true,
    );
}

fn spawn_combat_text(
    commands: &mut Commands,
    parent: Entity,
    kind: CombatTextKind,
    text: impl Into<String>,
    muted: bool,
) -> Entity {
    let entity = if muted {
        bevy_presentation::spawn_muted_label(commands, parent, text)
    } else {
        bevy_presentation::spawn_label(commands, parent, text)
    };
    commands.entity(entity).insert(CombatText(kind));
    entity
}

fn condition_text(view: &crate::presentation::CombatView) -> String {
    view.player_condition
        .as_deref()
        .map(|condition| format!("Condition: {condition}"))
        .unwrap_or_else(|| "Condition: none".to_string())
}

fn result_title_text(state: &CombatState, result: bool) -> String {
    if result {
        state
            .result
            .as_ref()
            .map(|result| result.result_title.clone())
            .unwrap_or_default()
    } else {
        String::new()
    }
}

fn result_note_text(state: &CombatState, result: bool) -> String {
    if result {
        state
            .result
            .as_ref()
            .map(|result| result.result_note.clone())
            .unwrap_or_default()
    } else {
        String::new()
    }
}

fn instruction_text(result: bool) -> String {
    if result {
        "Press Enter to continue.".into()
    } else {
        "Select an action with arrows or 1-3, then press Enter.".into()
    }
}

fn update_combat_labels(
    labels: &mut Query<(&CombatText, &mut Text)>,
    view: &crate::presentation::CombatView,
    state: &CombatState,
) {
    let result = state.phase == Some(CombatScreenPhase::Result);
    for (label, mut text) in labels.iter_mut() {
        let next = match label.0 {
            CombatTextKind::Summary => format!(
                "{} | Turn {} | {}",
                view.character.display_name(),
                view.turn,
                view.location_name
            ),
            CombatTextKind::Condition => condition_text(view),
            CombatTextKind::EnemyName => view.enemy.name.clone(),
            CombatTextKind::EnemyPower => format!("Power: {}", view.enemy_power),
            CombatTextKind::ResultTitle => result_title_text(state, result),
            CombatTextKind::ResultNote => result_note_text(state, result),
            CombatTextKind::Instruction => instruction_text(result),
        };
        if text.as_str() != next {
            *text = Text::new(next);
        }
    }
}

fn update_combat_gauge<M: Component>(
    commands: &mut Commands,
    gauges: &Query<(Entity, &bevy_presentation::UiHealthGauge), With<M>>,
    current: i32,
    maximum: i32,
) {
    if let Some((entity, gauge)) = gauges.iter().next() {
        if gauge.current != current || gauge.maximum != maximum {
            commands
                .entity(entity)
                .insert(bevy_presentation::UiHealthGauge { current, maximum });
        }
    }
}

fn render_combat_events(commands: &mut Commands, host: Entity, events: &[String]) {
    if events.is_empty() {
        bevy_presentation::spawn_muted_label(commands, host, "The encounter begins.");
    } else {
        for event in events.iter().rev().take(12).rev() {
            bevy_presentation::spawn_muted_label(commands, host, event.clone());
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/bevy_combat.rs"]
mod tests;
