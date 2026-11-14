//! Renderer-neutral assembly of character illustration parts.
//!
//! NPCs use deterministic weighted selection; the player resolves explicitly
//! selected stable part IDs. Image loading and compositing remain frontend work.

use std::collections::{BTreeMap, HashSet};

use crate::content::CharacterPartContent;
use crate::rng::DeterministicRng;

///
/// Slots that may deliberately contain no portrait layer. Keep this convention
/// renderer-neutral so NPC generation, character creation, and Character Lab
/// agree on which layers are optional.
pub(crate) fn appearance_slot_has_none_choice(slot: &str) -> bool {
    matches!(slot, "headgear" | "accessory")
}

/// Stable save marker for an explicit empty choice. Missing markers retain the
/// legacy behavior of resolving to the slot's first available part.
pub(crate) fn appearance_slot_none_id(slot: &str) -> String {
    format!("__teus_none__:{slot}")
}

pub(crate) fn appearance_slot_choice_count(slot: &str, part_count: usize) -> usize {
    if part_count == 0 {
        0
    } else {
        part_count
            + if appearance_slot_has_none_choice(slot) {
                1
            } else {
                0
            }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CharacterPortraitLayer {
    pub id: String,
    pub slot: String,
    pub asset: String,
    pub layer: i32,
}

/// Stable, non-random defaults used for new players and saves created before
/// player appearance selection existed. One valid part is chosen per slot.
pub(crate) fn default_character_appearance_ids(parts: &[CharacterPartContent]) -> Vec<String> {
    selectable_player_parts(parts)
        .values()
        .filter_map(|candidates| candidates.first().map(|part| part.id.clone()))
        .collect()
}

/// Resolve a player's explicitly selected part IDs without invoking NPC portrait
/// generation. Missing IDs fall back to the first valid part in their slot so
/// old saves and saves with unavailable mod content still render consistently.
pub(crate) fn resolve_character_portrait(
    parts: &[CharacterPartContent],
    selected_ids: &[String],
) -> Vec<CharacterPortraitLayer> {
    let selected_ids = selected_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut selected = Vec::new();
    for (slot, candidates) in selectable_player_parts(parts) {
        let none_id = appearance_slot_none_id(&slot);
        if selected_ids.contains(none_id.as_str()) {
            continue;
        }
        let chosen = candidates
            .iter()
            .find(|part| selected_ids.contains(part.id.as_str()))
            .copied()
            .or_else(|| candidates.first().copied());
        let Some(chosen) = chosen else {
            continue;
        };
        selected.push(CharacterPortraitLayer {
            id: chosen.id.clone(),
            slot,
            asset: chosen.asset.clone(),
            layer: chosen.layer,
        });
    }
    selected.sort_by(|left, right| {
        left.layer
            .cmp(&right.layer)
            .then_with(|| left.slot.cmp(&right.slot))
            .then_with(|| left.id.cmp(&right.id))
    });
    selected
}

fn selectable_player_parts(
    parts: &[CharacterPartContent],
) -> BTreeMap<String, Vec<&CharacterPartContent>> {
    let mut slots = BTreeMap::<String, Vec<&CharacterPartContent>>::new();
    for part in parts.iter().filter(|part| {
        !part.id.trim().is_empty() && !part.asset.trim().is_empty() && part.requires_tags.is_empty()
    }) {
        slots.entry(part.slot.clone()).or_default().push(part);
    }
    for candidates in slots.values_mut() {
        candidates.sort_by(|left, right| left.id.cmp(&right.id));
    }
    slots
}

pub(crate) fn actor_appearance_seed(world_seed: u64, actor_id: u64) -> u64 {
    let mixed = world_seed ^ actor_id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xA076_1D64_78BD_642F;
    DeterministicRng::new(mixed).next_u64()
}

/// Select zero or one compatible part per slot through weighted, deterministic selection.
pub(crate) fn generate_character_portrait(
    parts: &[CharacterPartContent],
    seed: u64,
    appearance_tags: &[String],
) -> Vec<CharacterPortraitLayer> {
    let tags = appearance_tags
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut slots: BTreeMap<String, Vec<&CharacterPartContent>> = BTreeMap::new();

    for part in parts {
        if part.weight == 0 || part.id.trim().is_empty() || part.asset.trim().is_empty() {
            continue;
        }
        slots.entry(part.slot.clone()).or_default().push(part);
    }

    let mut rng = DeterministicRng::new(seed);
    let mut selected = Vec::new();
    for (slot, mut candidates) in slots {
        candidates.retain(|part| {
            part.requires_tags
                .iter()
                .all(|tag| tags.contains(tag.as_str()))
                && part
                    .excludes_tags
                    .iter()
                    .all(|tag| !tags.contains(tag.as_str()))
        });
        candidates.sort_by(|left, right| left.id.cmp(&right.id));
        let total_weight = candidates
            .iter()
            .map(|part| u64::from(part.weight))
            .sum::<u64>();
        if total_weight == 0 {
            continue;
        }
        // One additional unit of weight represents choosing no layer. This is
        // deterministic per actor seed and only applies to the optional slots.
        let none_weight = if appearance_slot_has_none_choice(&slot) {
            1
        } else {
            0
        };
        let selection_weight = total_weight + none_weight;
        let mut draw = rng.next_u64() % selection_weight;
        if draw >= total_weight {
            continue;
        }
        let mut chosen = candidates[0];
        for candidate in candidates {
            let weight = u64::from(candidate.weight);
            if draw < weight {
                chosen = candidate;
                break;
            }
            draw -= weight;
        }
        selected.push(CharacterPortraitLayer {
            id: chosen.id.clone(),
            slot,
            asset: chosen.asset.clone(),
            layer: chosen.layer,
        });
    }

    selected.sort_by(|left, right| {
        left.layer
            .cmp(&right.layer)
            .then_with(|| left.slot.cmp(&right.slot))
            .then_with(|| left.id.cmp(&right.id))
    });
    selected
}

#[cfg(test)]
#[path = "../tests/unit/character_appearance.rs"]
mod tests;
