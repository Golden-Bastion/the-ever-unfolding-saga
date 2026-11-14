//! Bevy presentation and interaction for combat.
//!
//! Combat resolution remains in `game::combat`; this module only owns the
//! frontend state needed to present a turn and dispatch semantic input.

use bevy::prelude::*;

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::game::combat::{self, CombatEncounter, CombatStep};
use crate::input::InputEvent;
use crate::presentation::CombatResultView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatScreenPhase {
    Active,
    Result,
}

#[derive(Resource, Default)]
pub(crate) struct CombatState {
    encounter: Option<CombatEncounter>,
    result: Option<CombatResultView>,
    phase: Option<CombatScreenPhase>,
    selected: usize,
    dirty: bool,
}

impl CombatState {
    fn reset(&mut self, encounter: CombatEncounter) {
        self.encounter = Some(encounter);
        self.result = None;
        self.phase = Some(CombatScreenPhase::Active);
        self.selected = 0;
        self.dirty = true;
    }

    fn clear(&mut self) {
        self.encounter = None;
        self.result = None;
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

pub(crate) fn begin(
    state: &mut crate::model::GameState,
    combat_state: &mut CombatState,
) -> Result<(), String> {
    let encounter = combat::start_encounter(state).map_err(|error| error.to_string())?;
    combat_state.reset(encounter);
    Ok(())
}

fn combat_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut combat_state: ResMut<CombatState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
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
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation);
            }
            InputEvent::Character('2') => {
                set_selection(&mut combat_state, &mut navigation, 1);
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation);
            }
            InputEvent::Character('3') => {
                set_selection(&mut combat_state, &mut navigation, 2);
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation);
            }
            InputEvent::Confirm => {
                if combat_state.phase == Some(CombatScreenPhase::Active) {
                    let last = combat_state.action_count().saturating_sub(1);
                    combat_state.selected = navigation.selected.min(last);
                }
                activate_current_selection(&mut lifecycle, &mut combat_state, &mut navigation);
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
) {
    match combat_state.phase {
        Some(CombatScreenPhase::Active) => {
            let step = {
                let Some(session) = lifecycle.session.as_mut() else {
                    return;
                };
                let Some(encounter) = combat_state.encounter.as_mut() else {
                    return;
                };
                combat::resolve_action(&mut session.state, encounter, combat_state.selected)
            };
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
        CombatStep::Result { view, .. } => {
            combat_state.result = Some(*view);
            combat_state.phase = Some(CombatScreenPhase::Result);
            combat_state.selected = 0;
            navigation.selected = 0;
            combat_state.dirty = true;
        }
    }
}

fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    mut combat_state: ResMut<CombatState>,
    navigation: Res<NavigationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || lifecycle.session.is_none()
        || navigation.current_screen != Some(ScreenId::Combat)
        || !combat_state.dirty
    {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }

    let Some(encounter) = combat_state.encounter.as_ref() else {
        return;
    };
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let view = combat::build_combat_view(&session.state, encounter);
    let root = bevy_presentation::spawn_screen(&mut commands, "COMBAT");

    let actors = bevy_presentation::spawn_panel(&mut commands, root);
    bevy_presentation::spawn_label(
        &mut commands,
        actors,
        format!(
            "{} | Turn {} | {}",
            view.character.display_name(),
            view.turn,
            view.location_name
        ),
    );
    bevy_presentation::spawn_muted_label(
        &mut commands,
        actors,
        view.player_condition
            .as_deref()
            .map(|condition| format!("Condition: {condition}"))
            .unwrap_or_else(|| "Condition: none".to_string()),
    );
    bevy_presentation::spawn_label(&mut commands, actors, "Enemy");
    bevy_presentation::spawn_label(&mut commands, actors, view.enemy.name.clone());
    bevy_presentation::spawn_gauge(
        &mut commands,
        actors,
        view.enemy.current_hp,
        view.enemy.max_hp,
    );
    bevy_presentation::spawn_muted_label(
        &mut commands,
        actors,
        format!("Power: {}", view.enemy_power),
    );
    bevy_presentation::spawn_label(&mut commands, actors, "You");
    bevy_presentation::spawn_gauge(
        &mut commands,
        actors,
        view.character.hp,
        view.character.max_hp,
    );

    let events = bevy_presentation::spawn_panel(&mut commands, root);
    bevy_presentation::spawn_label(&mut commands, events, "Combat log");
    if view.events.is_empty() {
        bevy_presentation::spawn_muted_label(&mut commands, events, "The encounter begins.");
    } else {
        for event in view.events.iter().rev().take(12).rev() {
            bevy_presentation::spawn_muted_label(&mut commands, events, event.clone());
        }
    }

    let actions = bevy_presentation::spawn_panel(&mut commands, root);
    for (index, action) in view.actions.iter().enumerate() {
        let icon = match index {
            0 => "attack",
            1 => "insight",
            _ => "evade",
        };
        let button = bevy_presentation::spawn_action_button(
            &mut commands,
            actions,
            index,
            icon,
            format!("{}: {}", index + 1, action),
        );
        if combat_state.phase == Some(CombatScreenPhase::Active) && index == combat_state.selected {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }
    if combat_state.phase == Some(CombatScreenPhase::Result) {
        if let Some(result) = combat_state.result.as_ref() {
            bevy_presentation::spawn_label(&mut commands, actions, result.result_title.clone());
            bevy_presentation::spawn_muted_label(
                &mut commands,
                actions,
                result.result_note.clone(),
            );
            bevy_presentation::spawn_muted_label(
                &mut commands,
                actions,
                "Press Enter to continue.",
            );
        }
    } else {
        bevy_presentation::spawn_muted_label(
            &mut commands,
            actions,
            "Select an action with arrows or 1-3, then press Enter.",
        );
    }

    combat_state.dirty = false;
}

#[cfg(test)]
#[path = "../tests/unit/bevy_combat.rs"]
mod tests;
