//! Deterministic, renderer-neutral assembly of character illustration parts.
//!
//! The catalogue is campaign content; actors resolve their appearance from a
//! stable seed. Image loading and compositing remain frontend responsibilities.

use std::collections::{BTreeMap, HashSet};

use crate::content::CharacterPartContent;
use crate::rng::DeterministicRng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CharacterPortraitLayer {
    pub id: String,
    pub slot: String,
    pub asset: String,
    pub layer: i32,
}

pub(crate) fn actor_appearance_seed(world_seed: u64, actor_id: u64) -> u64 {
    let mixed = world_seed ^ actor_id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xA076_1D64_78BD_642F;
    DeterministicRng::new(mixed).next_u64()
}

/// Select one compatible part per slot through weighted, deterministic selection.
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

        let mut draw = rng.next_u64() % total_weight;
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
