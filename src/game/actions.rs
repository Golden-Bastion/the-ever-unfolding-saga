use crate::game::state_effects::{self, add_or_refresh_condition};
use crate::model::{Condition, EntityId, GameState, HistoryEntryType};
use crate::persistence::save_game;
use crate::presentation::{
    CharacterView, MeditationResultView, MeditationTargetView, MeditationView,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

const MEDITATION_TARGETS: [(u32, &str); 8] = [
    (2, "Dawn"),
    (3, "Morning"),
    (5, "High Sun"),
    (6, "Afternoon"),
    (8, "Dusk"),
    (9, "Evening"),
    (11, "Midnight"),
    (0, "Deep Night"),
];

/// Return whether the active location and destination are joined by a
/// reciprocal world-graph connection. All travel presentations use this
/// shared rule; local-map exits are not an independent source of topology.
pub(crate) fn has_reciprocal_location_connection(state: &GameState, target_id: EntityId) -> bool {
    let Some(current_location) = state.world.location_by_id(state.character.location_id) else {
        return false;
    };
    let Some(target_location) = state.world.location_by_id(target_id) else {
        return false;
    };

    current_location.exits.contains(&target_id)
        && target_location.exits.contains(&current_location.id)
}

fn is_known_location(state: &GameState, location_id: EntityId) -> bool {
    let stable_id = format!("location.{location_id}");
    state
        .world
        .local_maps
        .iter()
        .any(|map| map.location_id == stable_id)
}

/// Find a shortest route to a previously visited location using only reciprocal
/// connections between locations whose local maps have already been established.
/// The returned IDs are ordered hops and exclude the current location.
pub(crate) fn find_known_travel_path(
    state: &GameState,
    target_id: EntityId,
) -> Option<Vec<EntityId>> {
    let start_id = state.character.location_id;
    if target_id == start_id || !is_known_location(state, target_id) {
        return None;
    }

    let mut queue = VecDeque::from([start_id]);
    let mut visited = HashSet::from([start_id]);
    let mut previous = HashMap::<EntityId, EntityId>::new();

    while let Some(source_id) = queue.pop_front() {
        let source = state.world.location_by_id(source_id)?;
        let mut exits = source.exits.clone();
        exits.sort_unstable();
        exits.dedup();

        for next_id in exits {
            if visited.contains(&next_id)
                || !is_known_location(state, next_id)
                || !state
                    .world
                    .location_by_id(next_id)
                    .is_some_and(|next| next.exits.contains(&source_id))
            {
                continue;
            }

            visited.insert(next_id);
            previous.insert(next_id, source_id);
            if next_id == target_id {
                let mut path = Vec::new();
                let mut cursor = target_id;
                while cursor != start_id {
                    path.push(cursor);
                    cursor = *previous.get(&cursor)?;
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(next_id);
        }
    }

    None
}

/// Fast travel skips local movement but follows the known reciprocal route
/// network and applies the normal travel action for every hop. A death or active
/// threat at an intermediate arrival interrupts the journey at that location.
pub(crate) fn fast_travel_to(
    state: &mut GameState,
    target_id: EntityId,
) -> std::io::Result<Vec<EntityId>> {
    let path = find_known_travel_path(state, target_id).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Fast travel is only available to known locations connected by explored routes.",
        )
    })?;

    let mut traveled = Vec::with_capacity(path.len());
    for location_id in path {
        let previous_turn = state.character.turn;
        travel_to(state, location_id)?;
        traveled.push(location_id);

        let arrival_event =
            state.world.history.iter().any(|entry| {
                entry.turn > previous_turn && entry.entry_type == HistoryEntryType::Event
            });
        if !state.character.alive || state.threat.active || arrival_event {
            break;
        }
    }
    Ok(traveled)
}

pub(crate) fn travel_to(
    state: &mut GameState,
    target_id: crate::model::EntityId,
) -> std::io::Result<()> {
    if !has_reciprocal_location_connection(state, target_id) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Travel requires a reciprocal world-graph connection.",
        ));
    }

    state_effects::advance_time(state, 2);
    if state_effects::is_night(state.world.time_points) {
        add_or_refresh_condition(
            &mut state.character.conditions,
            Condition::new("Exhausted", 2, -1),
        );
    }
    state.character.turn += 1;
    state.character.location_id = target_id;
    state.threat.clear();
    state.last_announced_location_id = None;
    let location = state.world.location_by_id(target_id).cloned();
    let location_name = location
        .as_ref()
        .map(|loc| loc.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let character_name = state.character.display_name();
    state.world.record_history(
        state.character.turn,
        format!("{} traveled to {}.", character_name, location_name),
    );
    crate::game::quests::sync_active_quests(state);
    let dangerous = location.as_ref().map(|loc| loc.dangerous).unwrap_or(false);
    let context = crate::events::EventContext::for_travel_arrival(
        &location_name,
        dangerous,
        state_effects::is_night(state.world.time_points),
    );
    crate::events::trigger_event(state, &context);
    if let Some(location) = location {
        if location.dangerous {
            state.threat.activate(
                location.id,
                format!("{} stirs", location.name),
                "The air is tense. Something here is still awake.".to_string(),
            );
        }
    }
    Ok(())
}

fn character_view(state: &GameState) -> CharacterView {
    CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    }
}

pub(crate) fn build_meditation_view(state: &GameState) -> MeditationView {
    let location_id = state.character.location_id;
    let active_threat_here = state.threat.active
        && state
            .threat
            .source_location_id
            .is_none_or(|source| source == location_id);
    let safe_to_meditate = !active_threat_here && !state.world.location_is_dangerous(location_id);
    MeditationView {
        character: character_view(state),
        current_time: crate::game::time::time_display(state.world.time_points, state.world.day),
        safe_to_meditate,
        unavailable_message: (!safe_to_meditate)
            .then(|| "The place is not safe enough to meditate here.".to_string()),
        targets: MEDITATION_TARGETS
            .iter()
            .map(|(_, label)| MeditationTargetView {
                label: (*label).to_string(),
            })
            .collect(),
    }
}

pub(crate) fn meditate_to_target(
    state: &mut GameState,
    save_path: &Path,
    selection: usize,
) -> std::io::Result<MeditationResultView> {
    let view = build_meditation_view(state);
    if !view.safe_to_meditate || selection >= MEDITATION_TARGETS.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            view.unavailable_message
                .unwrap_or_else(|| "That meditation target is unavailable.".to_string()),
        ));
    }

    let target_slot = MEDITATION_TARGETS[selection].0;
    let current_slot = state.world.time_points % 12;
    let portions = ((target_slot + 12 - current_slot) % 12).max(1);

    let healing = portions as i32 + state.character.effective_endurance();
    state_effects::advance_time(state, portions);
    state.character.turn += 1;
    state.character.heal(healing);
    state_effects::remove_condition(&mut state.character.conditions, "Exhausted");
    let mut rested = Condition::new("Well-rested", 3, 0);
    rested.bonus = 1;
    add_or_refresh_condition(&mut state.character.conditions, rested);
    let character_name = state.character.display_name();
    let target_label = view.targets[selection].label.clone();
    state.world.record_history(
        state.character.turn,
        format!(
            "{} meditated until {} and recovered.",
            character_name, target_label
        ),
    );
    save_game(save_path, state)?;

    Ok(MeditationResultView {
        ending_time: crate::game::time::time_display(state.world.time_points, state.world.day),
        portions,
        hp_recovered: healing,
        exhausted_removed: true,
        well_rested_applied: true,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/game/actions.rs"]
mod tests;
