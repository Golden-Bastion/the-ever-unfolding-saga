use crate::game::state_effects::{self, add_or_refresh_condition};
use crate::model::{Condition, EntityId, GameState, HistoryEntryType};
use crate::persistence::save_game;
use crate::presentation::{
    CharacterView, MeditationResultView, MeditationTargetView, MeditationView,
};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
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

/// Return whether the active location has a route connection to the destination,
/// regardless of whether its crossing infrastructure currently permits travel.
/// This lets the interaction layer explain unavailable passages without making
/// them traversable. When a physical passage supplies a route ID, validate that
/// exact route rather than accepting another parallel connection.
pub(crate) fn has_location_route_connection(
    state: &GameState,
    target_id: EntityId,
    route_id: Option<&str>,
) -> bool {
    let source_id = state.character.location_id;
    if state.world.location_by_id(source_id).is_none()
        || state.world.location_by_id(target_id).is_none()
    {
        return false;
    }
    match route_id {
        Some(id) => state
            .world
            .route_by_id(id)
            .is_some_and(|route| route.allows_travel(source_id, target_id)),
        None => state
            .world
            .routes_from(source_id)
            .any(|(route, destination)| {
                destination == target_id && route.allows_travel(source_id, destination)
            }),
    }
}

/// Return whether the active location has a route that permits travel to
/// the destination. When a physical passage supplies a route ID, validate
/// that exact route rather than accepting another parallel connection.
pub(crate) fn has_location_travel_route(
    state: &GameState,
    target_id: EntityId,
    route_id: Option<&str>,
) -> bool {
    let route_is_traversable = |route: &crate::model::WorldRoute| {
        route
            .geography
            .as_ref()
            .is_none_or(|geography| geography.travel_block_reason().is_none())
    };
    let source_id = state.character.location_id;
    if state.world.location_by_id(source_id).is_none()
        || state.world.location_by_id(target_id).is_none()
    {
        return false;
    }
    match route_id {
        Some(id) => state.world.route_by_id(id).is_some_and(|route| {
            route.allows_travel(source_id, target_id) && route_is_traversable(route)
        }),
        None => state
            .world
            .routes_from(source_id)
            .any(|(route, destination)| destination == target_id && route_is_traversable(route)),
    }
}

/// Whether the campaign has established this location as visited.
///
/// The current location and persistent local maps are authoritative. For saves made before local maps
/// were persisted in the world, recover visited destinations from the retained
/// travel history of the current character and its recorded predecessors.
pub(crate) fn has_visited_location(state: &GameState, location_id: EntityId) -> bool {
    if state.character.location_id == location_id {
        return true;
    }
    let stable_id = format!("location.{location_id}");
    if state
        .world
        .local_maps
        .iter()
        .any(|map| map.location_id == stable_id)
    {
        return true;
    }

    let Some(location) = state.world.location_by_id(location_id) else {
        return false;
    };
    let mut traveler_names = HashSet::from([state.character.display_name()]);
    for entry in &state.world.history {
        if entry.entry_type != HistoryEntryType::Narrative {
            continue;
        }
        if let Some(name) = entry
            .text
            .strip_suffix(" entered the world.")
            .or_else(|| entry.text.strip_suffix(" inherited the world."))
        {
            if !name.is_empty() {
                traveler_names.insert(name.to_string());
            }
        }
    }

    let travel_suffix = format!(" traveled to {}.", location.name);
    state.world.history.iter().any(|entry| {
        entry.entry_type == HistoryEntryType::Narrative
            && entry
                .text
                .strip_suffix(&travel_suffix)
                .is_some_and(|traveler| traveler_names.contains(traveler))
    })
}

/// One directed hop in the fastest known route, retaining the exact route identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KnownTravelHop {
    pub location_id: EntityId,
    pub route_id: String,
    pub time_points: u32,
}

/// Estimate one route hop from its persisted route plan. Legacy and non-geographic
/// routes retain the original four-hour minimum until they gain route geometry.
fn route_travel_time_points(state: &GameState, route: &crate::model::WorldRoute) -> u32 {
    let Some(geography) = route.geography.as_ref() else {
        return 2;
    };
    let world_seed = state
        .world
        .geography_seed
        .or_else(|| state.world.generation.map(|generation| generation.seed));
    let ruleset_version = if state.world.geography_ruleset_version == 0 {
        state
            .world
            .generation
            .map_or(0, |generation| generation.geography_ruleset_version)
    } else {
        state.world.geography_ruleset_version
    };
    geography
        .travel_estimate(world_seed, ruleset_version)
        .time_points
}

/// Find the least-time directed path to a previously visited location using only
/// discovered, permitted connections between visited locations. Ties are resolved
/// by stable location and route IDs.
pub(crate) fn find_known_travel_route_path(
    state: &GameState,
    target_id: EntityId,
) -> Option<Vec<KnownTravelHop>> {
    let start_id = state.character.location_id;
    if target_id == start_id || !has_visited_location(state, target_id) {
        return None;
    }

    let mut queue = BinaryHeap::from([Reverse((0_u64, start_id))]);
    let mut distances = HashMap::from([(start_id, 0_u64)]);
    let mut previous = HashMap::<EntityId, (EntityId, String, u32)>::new();

    while let Some(Reverse((known_cost, source_id))) = queue.pop() {
        if distances.get(&source_id).copied() != Some(known_cost) {
            continue;
        }
        if source_id == target_id {
            break;
        }
        state.world.location_by_id(source_id)?;

        let mut exits = state
            .world
            .routes_from(source_id)
            .filter(|(route, destination)| {
                state.world.is_route_discovered(&route.id)
                    && route
                        .geography
                        .as_ref()
                        .is_none_or(|geography| geography.travel_block_reason().is_none())
                    && has_visited_location(state, *destination)
            })
            .collect::<Vec<_>>();
        exits.sort_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| left.0.id.cmp(&right.0.id))
        });

        for (route, next_id) in exits {
            let hop_time = route_travel_time_points(state, route);
            let next_cost = known_cost.saturating_add(u64::from(hop_time));
            let candidate_predecessor = (source_id, route.id.clone());
            let should_update = match distances.get(&next_id).copied() {
                None => true,
                Some(existing) if next_cost < existing => true,
                Some(existing) if next_cost == existing => {
                    previous
                        .get(&next_id)
                        .is_none_or(|(previous_id, previous_route, _)| {
                            candidate_predecessor < (*previous_id, previous_route.clone())
                        })
                }
                _ => false,
            };
            if !should_update {
                continue;
            }

            distances.insert(next_id, next_cost);
            previous.insert(next_id, (source_id, route.id.clone(), hop_time));
            queue.push(Reverse((next_cost, next_id)));
        }
    }

    if !distances.contains_key(&target_id) {
        return None;
    }

    let mut path = Vec::new();
    let mut cursor = target_id;
    while cursor != start_id {
        let (previous_id, route_id, time_points) = previous.get(&cursor)?.clone();
        path.push(KnownTravelHop {
            location_id: cursor,
            route_id,
            time_points,
        });
        cursor = previous_id;
    }
    path.reverse();
    Some(path)
}

/// Return the location IDs in the least-time known route, excluding the current location.
#[cfg(test)]
pub(crate) fn find_known_travel_path(
    state: &GameState,
    target_id: EntityId,
) -> Option<Vec<EntityId>> {
    find_known_travel_route_path(state, target_id)
        .map(|path| path.into_iter().map(|hop| hop.location_id).collect())
}

/// Fast travel skips local movement but follows the least-time known directed route
/// and applies the normal travel action for every hop. A death or active threat at
/// an intermediate arrival interrupts the journey at that location.
pub(crate) fn fast_travel_to(
    state: &mut GameState,
    target_id: EntityId,
) -> std::io::Result<Vec<EntityId>> {
    let path = find_known_travel_route_path(state, target_id).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Fast travel is only available to known locations connected by explored routes.",
        )
    })?;

    let mut traveled = Vec::with_capacity(path.len());
    for hop in path {
        let previous_turn = state.character.turn;
        travel_via_route(state, hop.location_id, &hop.route_id)?;
        traveled.push(hop.location_id);

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

pub(crate) fn travel_via_route(
    state: &mut GameState,
    target_id: EntityId,
    route_id: &str,
) -> std::io::Result<()> {
    let duration = {
        let Some(route) = state.world.route_by_id(route_id) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Travel requires the selected route to connect the current location and destination.",
            ));
        };
        if !has_location_travel_route(state, target_id, Some(route_id)) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Travel requires the selected route to connect the current location and destination.",
            ));
        }
        route_travel_time_points(state, route)
    };
    apply_route_travel(state, target_id, duration)?;
    state.world.mark_route_discovered(route_id.to_string());
    Ok(())
}

/// Select the least-time permitted route when a caller does not supply a passage identity.
#[cfg(test)]
pub(crate) fn travel_to(state: &mut GameState, target_id: EntityId) -> std::io::Result<()> {
    let source_id = state.character.location_id;
    let selected_route = state
        .world
        .routes_from(source_id)
        .filter(|(route, destination)| {
            *destination == target_id
                && has_location_travel_route(state, target_id, Some(&route.id))
        })
        .map(|(route, _)| (route.id.clone(), route_travel_time_points(state, route)))
        .min_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)))
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Travel requires a world route that permits travel to the destination.",
            )
        })?;
    apply_route_travel(state, target_id, selected_route.1)
}

fn apply_route_travel(
    state: &mut GameState,
    target_id: EntityId,
    duration_time_points: u32,
) -> std::io::Result<()> {
    state_effects::advance_time(state, duration_time_points);
    if state_effects::is_night(state.world.time_points) {
        add_or_refresh_condition(
            &mut state.character.conditions,
            Condition::new("Exhausted", 2, -1),
        );
    }
    state.character.turn += 1;
    state.character.location_id = target_id;
    if let Some(location) = state.world.location_by_id(target_id) {
        state.world.mark_region_surveyed(location.region_id);
    }
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
