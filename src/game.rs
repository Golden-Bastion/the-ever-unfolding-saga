pub(crate) mod actions;
pub(crate) mod character;
pub(crate) mod combat;
pub(crate) mod console;
pub(crate) mod history_screen;
#[path = "game/interactions.rs"]
pub(crate) mod interactions_core;
pub(crate) mod legacy;
pub(crate) mod menu;
pub(crate) mod navigation;
pub(crate) mod world_map;
pub(crate) mod quests;
pub(crate) mod records;
pub(crate) mod state_effects;
pub(crate) mod time;
pub mod world;
pub(crate) mod world_view;

pub(crate) mod interactions {
    pub(crate) use super::interactions_core::*;

    use crate::model::{GameState, Item};

    pub(crate) fn grant_reward_reputation(state: &mut GameState, item: &Item) {
        let Some(faction_name) = (match item.name.as_str() {
            "Wardens' Seal" => Some("Cinder Wardens"),
            "Rootworker's Token" => Some("Hollow Market Kin"),
            "Bell Covenant Charm" => Some("Drowned Bell Covenant"),
            _ => None,
        }) else {
            return;
        };
        let Some(faction_id) = super::interactions_core::faction_id_by_name(state, faction_name)
        else {
            return;
        };
        if let Some(faction) = state
            .factions
            .iter_mut()
            .find(|faction| faction.id == faction_id)
        {
            faction.reputation += 5;
            faction.memory.push(format!(
                "Carrying {} marks affiliation with the faction.",
                item.name
            ));
            if faction.memory.len() > 5 {
                let remove_count = faction.memory.len() - 5;
                faction.memory.drain(0..remove_count);
            }
        }
    }
}

pub(crate) use world::validate_loaded_state;
