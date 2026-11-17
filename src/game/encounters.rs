//! Deterministic, data-driven materialization of persistent local encounters.
//!
//! Profiles sharing a spawn group are weighted alternatives. Different groups
//! are evaluated independently, allowing multiple persistent encounters on one
//! map without changing the existing default-group behavior.

use crate::content::definitions::{EncounterContent, EncounterSpawnConditions};
use crate::model::{GameState, Location};
use crate::rng::DeterministicRng;
use crate::spatial::{
    ensure_spatial_encounter_avoiding, GridPosition, LocalMap, SpatialEncounterDefinition,
};
use std::collections::BTreeMap;

const DEFAULT_ENCOUNTER_SPAWN_GROUP: &str = "default";

/// Materialize one eligible profile per unrepresented spawn group on entry.
///
/// Existing map records are authoritative: a record reserves its group even
/// after defeat, and later profile changes cannot replace or heal it. Groups
/// with no currently eligible profile remain unresolved and can spawn later
/// when their authored conditions become true.
pub(crate) fn ensure_spatial_encounter_on_entry(
    state: &GameState,
    location: &Location,
    map: &mut LocalMap,
    avoid_position: Option<GridPosition>,
) -> bool {
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
        let mut groups = BTreeMap::<String, Vec<&EncounterContent>>::new();
        for profile in profiles {
            let group = profile
                .spawn_group
                .as_deref()
                .unwrap_or(DEFAULT_ENCOUNTER_SPAWN_GROUP);
            if group.trim().is_empty() {
                continue;
            }
            groups.entry(group.to_string()).or_default().push(profile);
        }

        let mut materialized = false;
        for (group, candidates) in groups {
            if map.encounters.iter().any(|existing| {
                effective_spawn_group(existing.spawn_group.as_deref()) == group.as_str()
            }) {
                continue;
            }
            let Some(selected) = select_profile(state, location, map, &group, candidates) else {
                continue;
            };
            materialized |= materialize_profile(location, map, selected, &group, avoid_position);
        }
        return materialized;
    }

    // Older campaigns and mods may have supplied only the global threat state
    // and a location-name-based encounter definition. Keep that path as a
    // fallback, but do not let it bypass authored spawn conditions or create a
    // second record in the default group.
    if map.encounters.iter().any(|existing| {
        effective_spawn_group(existing.spawn_group.as_deref()) == DEFAULT_ENCOUNTER_SPAWN_GROUP
    }) || !state.threat.active
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
            id: format!(
                "location.{}.encounter.legacy.{profile_key:016x}",
                location.id
            ),
            quest_id: None,
            spawn_group: None,
            enemy_name,
            description,
            max_hp,
            power,
            trophy_item_name,
        },
        avoid_position,
    )
}

fn effective_spawn_group(group: Option<&str>) -> &str {
    group.unwrap_or(DEFAULT_ENCOUNTER_SPAWN_GROUP)
}

fn select_profile<'a>(
    state: &GameState,
    location: &Location,
    map: &LocalMap,
    spawn_group: &str,
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
    let selection_seed = if spawn_group == DEFAULT_ENCOUNTER_SPAWN_GROUP {
        spawn_seed(state, location, map)
    } else {
        spawn_seed(state, location, map) ^ stable_hash(spawn_group)
    };
    let mut rng = DeterministicRng::new(selection_seed);
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
        && profile.spawn_group.is_none()
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
        && conditions
            .required_completed_quest_id
            .as_deref()
            .is_none_or(|required| {
                state
                    .world
                    .completed_quest_ids
                    .iter()
                    .any(|completed| completed == required)
            })
        && conditions
            .required_condition_name
            .as_deref()
            .is_none_or(|required| {
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
    spawn_group: &str,
    avoid_position: Option<GridPosition>,
) -> bool {
    let identity = profile
        .id
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| {
            let profile_hash = stable_hash(profile_identity(profile));
            if spawn_group == DEFAULT_ENCOUNTER_SPAWN_GROUP {
                format!("legacy.{profile_hash:016x}")
            } else {
                format!(
                    "legacy.{profile_hash:016x}.group.{:016x}",
                    stable_hash(spawn_group)
                )
            }
        });
    ensure_spatial_encounter_avoiding(
        map,
        SpatialEncounterDefinition {
            id: format!("location.{}.encounter.{}", location.id, identity),
            quest_id: profile.quest_id.clone(),
            spawn_group: (spawn_group != DEFAULT_ENCOUNTER_SPAWN_GROUP)
                .then(|| spawn_group.to_string()),
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
