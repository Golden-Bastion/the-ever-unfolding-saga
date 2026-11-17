//! Deterministic, data-driven materialization of persistent local encounters.
//!
//! This module selects authored encounter profiles from the current world
//! context, then snapshots the selected profile into the renderer-neutral local
//! map. Existing map records always win: content or condition changes never
//! silently replace, heal, or respawn an established encounter.

use crate::content::definitions::{EncounterContent, EncounterSpawnConditions};
use crate::model::{GameState, Location};
use crate::rng::DeterministicRng;
use crate::spatial::{
    ensure_spatial_encounter_avoiding, GridPosition, LocalMap, SpatialEncounterDefinition,
};

/// Materialize the first eligible encounter for a location when its local map
/// has no encounter history. A defeated record is history too and prevents a
/// later re-entry from silently respawning the threat.
pub(crate) fn ensure_spatial_encounter_on_entry(
    state: &GameState,
    location: &Location,
    map: &mut LocalMap,
    avoid_position: Option<GridPosition>,
) -> bool {
    if !map.encounters.is_empty() {
        return false;
    }

    let content = state.campaign_content.as_ref();
    let content_location_id = content.and_then(|content| {
        content
            .world
            .locations
            .iter()
            .find(|definition| definition.name == location.name)
            .map(|definition| definition.id.as_str())
    });
    let profiles = content
        .map(|content| {
            content.encounter_profiles_for_location(
                content_location_id.unwrap_or_default(),
                &location.name,
            )
        })
        .unwrap_or_default();

    if !profiles.is_empty() {
        let Some(selected) = select_profile(state, location, map, profiles) else {
            return false;
        };
        return materialize_profile(location, map, selected, avoid_position);
    }

    // Older campaigns and mods may have supplied only the global threat state
    // and a location-name-based encounter definition. Keep that path as a
    // fallback, but do not let it bypass authored spawn conditions.
    if !state.threat.active
        || state
            .threat
            .source_location_id
            .is_some_and(|source| source != location.id)
    {
        return false;
    }
    let (enemy_name, description, max_hp, power, trophy_item_name) =
        crate::game::combat::encounter_profile(state, &location.name);
    let profile_key = stable_hash(&format!(
        "{}|{}|{}",
        location.name, enemy_name, trophy_item_name
    ));
    ensure_spatial_encounter_avoiding(
        map,
        SpatialEncounterDefinition {
            id: format!("location.{}.encounter.legacy.{profile_key:016x}", location.id),
            quest_id: None,
            enemy_name,
            description,
            max_hp,
            power,
            trophy_item_name,
        },
        avoid_position,
    )
}

fn select_profile<'a>(
    state: &GameState,
    location: &Location,
    map: &LocalMap,
    profiles: Vec<&'a EncounterContent>,
) -> Option<&'a EncounterContent> {
    let mut candidates = profiles
        .into_iter()
        .filter(|profile| profile.weight > 0)
        .filter(|profile| {
            !is_legacy_profile(profile) || legacy_threat_belongs_here(state, location)
        })
        .filter(|profile| {
            profile.enemy_hp > 0
                && profile.enemy_power >= 0
                && !profile.enemy_name.trim().is_empty()
                && !profile.description.trim().is_empty()
                && !profile.trophy_item_name.trim().is_empty()
        })
        .filter(|profile| {
            profile
                .conditions
                .as_ref()
                .is_none_or(|conditions| conditions_match(state, location, conditions))
        })
        .filter(|profile| chance_succeeds(state, location, map, profile))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| profile_identity(left).cmp(profile_identity(right)));
    if candidates.is_empty() {
        return None;
    }

    let total_weight = candidates.iter().fold(0_u64, |total, profile| {
        total.saturating_add(u64::from(profile.weight))
    });
    if total_weight == 0 {
        return None;
    }
    let mut rng = DeterministicRng::new(spawn_seed(state, location, map));
    let mut selection = rng.next_u64() % total_weight;
    for profile in &candidates {
        let weight = u64::from(profile.weight);
        if selection < weight {
            return Some(*profile);
        }
        selection -= weight;
    }
    candidates.last().copied()
}

fn is_legacy_profile(profile: &EncounterContent) -> bool {
    profile.id.is_none()
        && profile.location_id.is_none()
        && profile.quest_id.is_none()
        && profile.conditions.is_none()
}

fn legacy_threat_belongs_here(state: &GameState, location: &Location) -> bool {
    state.threat.active
        && state
            .threat
            .source_location_id
            .is_none_or(|source| source == location.id)
}

fn conditions_match(
    state: &GameState,
    location: &Location,
    conditions: &EncounterSpawnConditions,
) -> bool {
    let night = crate::game::state_effects::is_night(state.world.time_points);
    conditions.night.is_none_or(|required| required == night)
        && conditions
            .dangerous
            .is_none_or(|required| required == location.dangerous)
        && conditions
            .min_day
            .is_none_or(|minimum| state.world.day >= minimum)
        && conditions
            .max_day
            .is_none_or(|maximum| state.world.day <= maximum)
        && conditions.required_completed_quest_id.as_deref().is_none_or(|required| {
            state
                .world
                .completed_quest_ids
                .iter()
                .any(|completed| completed == required)
        })
        && conditions.required_condition_name.as_deref().is_none_or(|required| {
            state
                .character
                .conditions
                .iter()
                .any(|condition| condition.name == required)
        })
}

fn chance_succeeds(
    state: &GameState,
    location: &Location,
    map: &LocalMap,
    profile: &EncounterContent,
) -> bool {
    let chance = profile.chance_percent.unwrap_or(100).min(100);
    if chance == 100 {
        return true;
    }
    if chance == 0 {
        return false;
    }
    let seed = spawn_seed(state, location, map) ^ stable_hash(profile_identity(profile));
    let mut rng = DeterministicRng::new(seed);
    rng.next_u64() % 100 < u64::from(chance)
}

fn materialize_profile(
    location: &Location,
    map: &mut LocalMap,
    profile: &EncounterContent,
    avoid_position: Option<GridPosition>,
) -> bool {
    let identity = profile
        .id
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| format!("legacy.{:016x}", stable_hash(profile_identity(profile))));
    ensure_spatial_encounter_avoiding(
        map,
        SpatialEncounterDefinition {
            id: format!("location.{}.encounter.{}", location.id, identity),
            quest_id: profile.quest_id.clone(),
            enemy_name: profile.enemy_name.clone(),
            description: profile.description.clone(),
            max_hp: profile.enemy_hp,
            power: profile.enemy_power,
            trophy_item_name: profile.trophy_item_name.clone(),
        },
        avoid_position,
    )
}

fn profile_identity(profile: &EncounterContent) -> &str {
    profile.id.as_deref().unwrap_or(&profile.enemy_name)
}

fn spawn_seed(state: &GameState, location: &Location, map: &LocalMap) -> u64 {
    map.seed
        ^ location.id.rotate_left(17)
        ^ u64::from(state.world.day).rotate_left(31)
        ^ u64::from(state.world.time_points).rotate_left(47)
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
#[path = "../../tests/unit/game/encounters.rs"]
mod tests;
