//! Bevy presentation for transient player feedback.
//!
//! Gameplay code remains frontend-neutral. This layer observes authoritative
//! state changes and turns player-facing consequences into temporary Bevy
//! screens, including the interactive level-up choice that used to live in the
//! terminal presentation layer.

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::input::InputEvent;
use crate::model::GameState;

#[derive(Debug, Clone, PartialEq, Eq)]
enum FeedbackEntry {
    LevelUp {
        level: u32,
    },
    QuestReward {
        added_items: Vec<(String, String)>,
        consumed_items: Vec<String>,
        reputation_changes: Vec<(String, i32)>,
    },
}

#[derive(Resource, Default)]
pub(crate) struct FeedbackState {
    initialized: bool,
    last_level: u32,
    last_inventory: HashMap<u64, (String, String)>,
    last_factions: HashMap<String, i32>,
    entries: VecDeque<FeedbackEntry>,
    return_screen: Option<ScreenId>,
    selected: usize,
    active: bool,
    dirty: bool,
}

#[derive(Component)]
struct FeedbackOverlay;

pub(crate) fn install(app: &mut App) {
    app.init_resource::<FeedbackState>().add_systems(
        Update,
        (observe_state_changes, feedback_input, render_if_active).chain(),
    );
}

fn observe_state_changes(
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut feedback: ResMut<FeedbackState>,
) {
    let Some(session) = lifecycle.session.as_ref() else {
        feedback.initialized = false;
        feedback.last_level = 0;
        feedback.last_inventory.clear();
        feedback.last_factions.clear();
        feedback.entries.clear();
        feedback.return_screen = None;
        feedback.selected = 0;
        feedback.active = false;
        feedback.dirty = true;
        return;
    };

    if lifecycle.phase != LifecyclePhase::Complete {
        return;
    }

    let state = &session.state;
    if !feedback.initialized {
        snapshot_state(&mut feedback, state);
        feedback.initialized = true;
        return;
    }

    if feedback.active || navigation.current_screen == Some(ScreenId::Feedback) {
        snapshot_state(&mut feedback, state);
        return;
    }

    let added_items = state
        .character
        .inventory
        .iter()
        .filter(|item| !feedback.last_inventory.contains_key(&item.id))
        .map(|item| (item.name.clone(), item.description.clone()))
        .collect::<Vec<_>>();
    let consumed_items = feedback
        .last_inventory
        .iter()
        .filter(|(id, _)| !state.character.inventory.iter().any(|item| item.id == **id))
        .map(|(_, (name, _))| name.clone())
        .collect::<Vec<_>>();
    let reputation_changes = state
        .factions
        .iter()
        .filter_map(|faction| {
            let previous = feedback
                .last_factions
                .get(&faction.name)
                .copied()
                .unwrap_or(0);
            let delta = faction.reputation - previous;
            (delta != 0).then(|| (faction.name.clone(), delta))
        })
        .collect::<Vec<_>>();

    if state.character.level > feedback.last_level {
        for level in (feedback.last_level + 1)..=state.character.level {
            feedback.entries.push_back(FeedbackEntry::LevelUp { level });
        }
    }

    if is_quest_reward_context(navigation.current_screen)
        && (!added_items.is_empty() || !consumed_items.is_empty() || !reputation_changes.is_empty())
    {
        feedback.entries.push_back(FeedbackEntry::QuestReward {
            added_items,
            consumed_items,
            reputation_changes,
        });
    }

    snapshot_state(&mut feedback, state);
    if !feedback.entries.is_empty() {
        feedback.return_screen = navigation.current_screen;
        feedback.selected = 0;
        feedback.active = true;
        navigation_is_feedback(&navigation, &mut feedback);
    }
}

fn navigation_is_feedback(_navigation: &NavigationState, feedback: &mut FeedbackState) {
    feedback.dirty = true;
}

fn is_quest_reward_context(screen: Option<ScreenId>) -> bool {
    matches!(screen, Some(ScreenId::Conversation))
}

fn snapshot_state(feedback: &mut FeedbackState, state: &GameState) {
    feedback.last_level = state.character.level;
    feedback.last_inventory = state
        .character
        .inventory
        .iter()
        .map(|item| (item.id, (item.name.clone(), item.description.clone())))
        .collect();
    feedback.last_factions = state
        .factions
        .iter()
        .map(|faction| (faction.name.clone(), faction.reputation))
        .collect();
}

fn feedback_input(
    mut navigation: ResMut<NavigationState>,
    mut feedback: ResMut<FeedbackState>,
    mut input_queue: ResMut<GameplayInputQueue>,
    mut lifecycle: ResMut<LifecycleState>,
) {
    if !feedback.active
        || navigation.current_screen != Some(ScreenId::Feedback)
        || input_queue.0.is_empty()
    {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => move_level_selection(&mut feedback, -1),
            InputEvent::Down => move_level_selection(&mut feedback, 1),
            InputEvent::Home => set_level_selection(&mut feedback, 0),
            InputEvent::End => set_level_selection(&mut feedback, 2),
            InputEvent::Character('1') => {
                choose_attribute(&mut lifecycle, &mut feedback, 0, &mut navigation)
            }
            InputEvent::Character('2') => {
                choose_attribute(&mut lifecycle, &mut feedback, 1, &mut navigation)
            }
            InputEvent::Character('3') => {
                choose_attribute(&mut lifecycle, &mut feedback, 2, &mut navigation)
            }
            InputEvent::Confirm => {
                if matches!(
                    feedback.entries.front(),
                    Some(FeedbackEntry::LevelUp { .. })
                ) {
                    let selected = feedback.selected;
                    choose_attribute(&mut lifecycle, &mut feedback, selected, &mut navigation);
                } else {
                    dismiss_current(&mut feedback, &mut navigation);
                }
            }
            InputEvent::Cancel
                if !matches!(
                    feedback.entries.front(),
                    Some(FeedbackEntry::LevelUp { .. })
                ) =>
            {
                dismiss_current(&mut feedback, &mut navigation);
            }
            _ => {}
        }
    }
}

fn move_level_selection(feedback: &mut FeedbackState, direction: isize) {
    if !matches!(
        feedback.entries.front(),
        Some(FeedbackEntry::LevelUp { .. })
    ) {
        return;
    }
    feedback.selected = (feedback.selected as isize + direction).rem_euclid(3) as usize;
    feedback.dirty = true;
}

fn set_level_selection(feedback: &mut FeedbackState, selected: usize) {
    if !matches!(
        feedback.entries.front(),
        Some(FeedbackEntry::LevelUp { .. })
    ) {
        return;
    }
    feedback.selected = selected.min(2);
    feedback.dirty = true;
}

fn choose_attribute(
    lifecycle: &mut LifecycleState,
    feedback: &mut FeedbackState,
    choice: usize,
    navigation: &mut NavigationState,
) {
    if !matches!(
        feedback.entries.front(),
        Some(FeedbackEntry::LevelUp { .. })
    ) {
        return;
    }
    let Some(session) = lifecycle.session.as_mut() else {
        return;
    };
    match choice.min(2) {
        0 => session.state.character.attributes.might += 1,
        1 => session.state.character.attributes.insight += 1,
        _ => session.state.character.attributes.endurance += 1,
    }
    feedback.entries.pop_front();
    feedback.selected = 0;
    finish_or_advance(feedback, navigation);
}

fn dismiss_current(feedback: &mut FeedbackState, navigation: &mut NavigationState) {
    feedback.entries.pop_front();
    feedback.selected = 0;
    finish_or_advance(feedback, navigation);
}

fn finish_or_advance(feedback: &mut FeedbackState, navigation: &mut NavigationState) {
    if feedback.entries.is_empty() {
        feedback.active = false;
        navigation.current_screen = feedback.return_screen.take();
        navigation.selected = 0;
    }
    feedback.dirty = true;
}

fn render_if_active(
    mut commands: Commands,
    mut navigation: ResMut<NavigationState>,
    mut feedback: ResMut<FeedbackState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
    overlays: Query<Entity, With<FeedbackOverlay>>,
) {
    if !feedback.active {
        for overlay in &overlays {
            commands.entity(overlay).despawn();
        }
        return;
    }

    if navigation.current_screen != Some(ScreenId::Feedback) {
        navigation.current_screen = Some(ScreenId::Feedback);
        navigation.selected = feedback.selected;
    }

    if !feedback.dirty {
        return;
    }

    for overlay in &overlays {
        commands.entity(overlay).despawn();
    }

    let Some(parent) = roots.iter().next() else {
        return;
    };
    let overlay = commands
        .spawn((
            FeedbackOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: bevy_presentation::responsive_vmin(3.333),
                right: bevy_presentation::responsive_vmin(3.333),
                top: bevy_presentation::responsive_vmin(3.333),
                bottom: bevy_presentation::responsive_vmin(3.333),
                padding: UiRect::all(bevy_presentation::responsive_vmin(2.778)),
                min_width: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: bevy_presentation::responsive_vmin(1.944),
                overflow: Overflow::scroll(),
                ..default()
            },
            BackgroundColor(Color::srgba(7.0 / 255.0, 7.0 / 255.0, 7.0 / 255.0, 0.97)),
        ))
        .id();
    commands.entity(parent).add_child(overlay);

    match feedback.entries.front() {
        Some(FeedbackEntry::LevelUp { level }) => {
            render_level_up(&mut commands, overlay, *level, feedback.selected)
        }
        Some(FeedbackEntry::QuestReward {
            added_items,
            consumed_items,
            reputation_changes,
        }) => render_quest_reward(
            &mut commands,
            overlay,
            added_items,
            consumed_items,
            reputation_changes,
        ),
        None => {}
    }
    feedback.dirty = false;
}

fn render_level_up(commands: &mut Commands, parent: Entity, level: u32, selected: usize) {
    bevy_presentation::spawn_label(commands, parent, "LEVEL UP");
    bevy_presentation::spawn_muted_label(
        commands,
        parent,
        format!("You have grown stronger. You reached level {level}."),
    );
    bevy_presentation::spawn_muted_label(commands, parent, "Choose a new strength:");
    for (index, (icon, label)) in [
        ("attack", "Might (+1 attack)"),
        ("insight", "Insight (+1 search/recovery)"),
        ("endurance", "Endurance (+1 meditation healing)"),
    ]
    .into_iter()
    .enumerate()
    {
        let button = bevy_presentation::spawn_action_button(commands, parent, index, icon, label);
        if index == selected {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }
    bevy_presentation::spawn_muted_label(
        commands,
        parent,
        "Choose with arrows and Enter, or press 1-3.",
    );
}

fn render_quest_reward(
    commands: &mut Commands,
    parent: Entity,
    added_items: &[(String, String)],
    consumed_items: &[String],
    reputation_changes: &[(String, i32)],
) {
    bevy_presentation::spawn_label(commands, parent, "QUEST REWARDS");
    if !added_items.is_empty() {
        bevy_presentation::spawn_muted_label(commands, parent, "You gain:");
        for (name, description) in added_items {
            bevy_presentation::spawn_label(commands, parent, name.clone());
            if !description.trim().is_empty() {
                bevy_presentation::spawn_muted_label(commands, parent, description.clone());
            }
        }
    }
    if !consumed_items.is_empty() {
        bevy_presentation::spawn_muted_label(commands, parent, "Quest items consumed:");
        for name in consumed_items {
            bevy_presentation::spawn_muted_label(commands, parent, format!("  {name}"));
        }
    }
    if !reputation_changes.is_empty() {
        bevy_presentation::spawn_muted_label(commands, parent, "Reputation:");
        for (faction, delta) in reputation_changes {
            bevy_presentation::spawn_muted_label(
                commands,
                parent,
                format!("  {faction} {delta:+}"),
            );
        }
    }
    bevy_presentation::spawn_muted_label(commands, parent, "Press Enter to continue.");
}

#[cfg(test)]
#[path = "../tests/unit/bevy_feedback.rs"]
mod tests;
