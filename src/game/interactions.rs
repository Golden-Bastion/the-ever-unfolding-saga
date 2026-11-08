use crate::character_appearance::{
    actor_appearance_seed, generate_character_portrait, resolve_character_portrait,
};
use crate::game::{quests, state_effects};
use crate::model::{EntityId, Faction, GameState, Quest};
use crate::presentation::{
    CharacterPortraitLayerView, CharacterPortraitView, CharacterView, ConversationView, NpcView,
    TalkView,
};

pub(crate) fn npc_ids_at_location(state: &GameState, location_id: EntityId) -> Vec<EntityId> {
    state
        .npcs
        .iter()
        .filter(|npc| npc.location_id == location_id)
        .map(|npc| npc.id)
        .collect()
}

pub(crate) fn npc_index_by_id(state: &GameState, npc_id: EntityId) -> Option<usize> {
    state.npcs.iter().position(|npc| npc.id == npc_id)
}

fn quest_key(quest: &Quest) -> String {
    if quest.content_id.is_empty() {
        format!("legacy.quest.{}", quest.id)
    } else {
        quest.content_id.clone()
    }
}

pub(crate) fn faction_id_by_name(state: &GameState, faction_name: &str) -> Option<EntityId> {
    state
        .factions
        .iter()
        .find(|faction| faction.name == faction_name)
        .map(|faction| faction.id)
}

fn faction_by_id_mut(state: &mut GameState, faction_id: EntityId) -> Option<&mut Faction> {
    state
        .factions
        .iter_mut()
        .find(|faction| faction.id == faction_id)
}

fn build_talk_view(state: &GameState) -> TalkView {
    let location_id = state.character.location_id;
    TalkView {
        npcs: npc_ids_at_location(state, location_id)
            .into_iter()
            .filter_map(|npc_id| {
                let npc = state.npcs.get(npc_index_by_id(state, npc_id)?)?;
                let faction_name = npc.faction_id.and_then(|faction_id| {
                    state
                        .factions
                        .iter()
                        .find(|faction| faction.id == faction_id)
                        .map(|faction| faction.name.clone())
                });
                Some(NpcView {
                    name: npc.name.clone(),
                    title: npc.title.clone(),
                    faction_name,
                })
            })
            .collect(),
    }
}

fn portrait_view(
    layers: Vec<crate::character_appearance::CharacterPortraitLayer>,
) -> CharacterPortraitView {
    CharacterPortraitView {
        layers: layers
            .into_iter()
            .map(|layer| CharacterPortraitLayerView {
                id: layer.id,
                slot: layer.slot,
                asset: layer.asset,
                layer: layer.layer,
            })
            .collect(),
    }
}

fn build_conversation_view(state: &GameState, npc_index: usize) -> ConversationView {
    let npc = &state.npcs[npc_index];
    let npc_view = NpcView {
        name: npc.name.clone(),
        title: npc.title.clone(),
        faction_name: npc.faction_id.and_then(|faction_id| {
            state
                .factions
                .iter()
                .find(|faction| faction.id == faction_id)
                .map(|faction| faction.name.clone())
        }),
    };
    let npc_name = npc_view.display_name();
    let content = state.campaign_content.as_ref();
    let portrait = content
        .and_then(|content| content.portrait_for(&npc.name))
        .map(str::to_string);
    let parts = content
        .map(|content| content.character_parts.as_slice())
        .unwrap_or(&[]);
    let world_seed = state
        .world
        .generation
        .map(|generation| generation.seed)
        .unwrap_or(state.world.id);
    let npc_tags = content
        .map(|content| content.appearance_tags_for(&npc.name))
        .unwrap_or(&[]);
    let player_layers = resolve_character_portrait(parts, &state.character.appearance_parts);
    let npc_layers =
        generate_character_portrait(parts, actor_appearance_seed(world_seed, npc.id), npc_tags);
    let player = CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    };
    let memory = npc
        .memory
        .iter()
        .rev()
        .find(|entry| !crate::model::is_internal_memory_entry(entry))
        .cloned();
    let available = npc_is_available_now(state.world.time_points);
    let unavailable_message =
        (!available).then(|| npc_unavailable_message(&npc_name, state.world.time_points));
    let mut options = vec![
        "Ask if they need help".to_string(),
        "Tell them it's done".to_string(),
    ];
    if state.character.effective_insight() >= 2 && memory.is_some() {
        options.push("Ask what they remember".to_string());
    }

    ConversationView {
        player,
        player_portrait: portrait_view(player_layers),
        npc: npc_view,
        npc_portrait: portrait_view(npc_layers),
        portrait,
        memory,
        options,
        available,
        unavailable_message,
    }
}

fn remember_npc(state: &mut GameState, npc_id: EntityId, memory: String) {
    if let Some(index) = npc_index_by_id(state, npc_id) {
        let npc = &mut state.npcs[index];
        npc.memory.push(memory);
        if npc.memory.len() > 5 {
            let remove_count = npc.memory.len() - 5;
            npc.memory.drain(0..remove_count);
        }
    }
}

fn remember_faction(state: &mut GameState, faction_id: EntityId, memory: String) {
    if let Some(faction) = faction_by_id_mut(state, faction_id) {
        faction.memory.push(memory);
        if faction.memory.len() > 5 {
            let remove_count = faction.memory.len() - 5;
            faction.memory.drain(0..remove_count);
        }
    }
}

fn npc_unavailable_message(npc_name: &str, points: u32) -> String {
    let slot = points % 12;
    let (reason, hint) = match slot {
        0 | 1 => ("It is too late in the night.", "Try again after dawn."),
        10 | 11 => ("It is too late tonight.", "Try again in the morning."),
        2..=5 => (
            "It is still too early in the day.",
            "Try again later today.",
        ),
        6..=9 => ("It is too late in the day.", "Try again tomorrow morning."),
        _ => ("They are unavailable right now.", "Try again later."),
    };
    format!(
        "{} is not available right now. {} {}",
        npc_name, reason, hint
    )
}

fn npc_is_available_now(points: u32) -> bool {
    matches!(points % 12, 2..=9)
}

pub(crate) fn update_faction_memory_for_location(
    state: &mut GameState,
    location_id: EntityId,
    memory: String,
) {
    let npc_ids = npc_ids_at_location(state, location_id);
    let mut faction_ids = Vec::new();
    for npc_id in npc_ids {
        if let Some(index) = npc_index_by_id(state, npc_id) {
            let npc = &state.npcs[index];
            if let Some(faction_id) = npc.faction_id {
                faction_ids.push(faction_id);
                remember_npc(state, npc_id, memory.clone());
            }
        }
    }
    faction_ids.sort_unstable();
    faction_ids.dedup();
    for faction_id in faction_ids {
        remember_faction(state, faction_id, memory.clone());
    }
}

pub(crate) fn build_talk_view_for_bevy(state: &GameState) -> TalkView {
    build_talk_view(state)
}

pub(crate) fn build_conversation_view_for_bevy(
    state: &GameState,
    npc_index: usize,
) -> Option<ConversationView> {
    if npc_index >= state.npcs.len() {
        None
    } else {
        Some(build_conversation_view(state, npc_index))
    }
}

pub(crate) fn perform_conversation_choice(
    state: &mut GameState,
    npc_id: EntityId,
    choice: usize,
) -> Vec<String> {
    let Some(npc_index) = npc_index_by_id(state, npc_id) else {
        return vec!["That person can no longer be found.".to_string()];
    };
    let view = build_conversation_view(state, npc_index);
    let npc_name = view.npc.display_name();
    if !view.available {
        return view.unavailable_message.into_iter().collect::<Vec<_>>();
    }

    let quest_indices: Vec<usize> = state
        .quests
        .iter()
        .enumerate()
        .filter(|(_, quest)| quest.giver_npc_id == npc_id)
        .map(|(index, _)| index)
        .collect();
    if quest_indices.is_empty() && view.options.len() <= 2 {
        return vec![format!("{} has little to say.", npc_name)];
    }

    let mut messages = Vec::new();
    match choice {
        0 => {
            let mut found_offer = false;
            for quest_index in quest_indices {
                let (key, title, description, faction_id, offered, completed) = {
                    let quest = &state.quests[quest_index];
                    (
                        quest_key(quest),
                        quest.title.clone(),
                        quest.description.clone(),
                        quest.faction_id,
                        quest.offered,
                        quest.completed,
                    )
                };
                if state
                    .world
                    .completed_quest_ids
                    .iter()
                    .any(|known| known == &key)
                    || completed
                {
                    continue;
                }
                found_offer = true;
                if offered {
                    messages.push(format!(
                        "{} says: 'You already agreed to help with {}.'",
                        npc_name, title
                    ));
                } else {
                    if let Some(quest) = state.quests.get_mut(quest_index) {
                        quest.offered = true;
                    }
                    quests::sync_objectives(state, quest_index);
                    messages.push(format!("{} says: '{}'", npc_name, description));
                    messages.extend(quests::objective_summary(state, quest_index));
                    remember_npc(state, npc_id, format!("offered the quest {}", title));
                    remember_faction(
                        state,
                        faction_id,
                        format!("{} offered the quest {}.", npc_name, title),
                    );
                }
            }
            if !found_offer {
                messages.push(format!(
                    "{} has no work for you. Whatever was asked here has already been done.",
                    npc_name
                ));
            }
        }
        1 => {
            let mut handled = false;
            for quest_index in quest_indices {
                let (key, title, offered, completed) = {
                    let quest = &state.quests[quest_index];
                    (
                        quest_key(quest),
                        quest.title.clone(),
                        quest.offered,
                        quest.completed,
                    )
                };
                if state
                    .world
                    .completed_quest_ids
                    .iter()
                    .any(|known| known == &key)
                    || completed
                {
                    continue;
                }
                if !offered {
                    messages.push(format!(
                        "{} does not know what you are talking about. You have not accepted any work from them.",
                        npc_name
                    ));
                    handled = true;
                    continue;
                }
                handled = true;
                if quests::try_complete(state, quest_index) {
                    messages.push(format!("Quest complete: {}", title));
                } else {
                    messages.push(format!(
                        "{} looks at you expectantly. The work is not finished yet:",
                        npc_name
                    ));
                    messages.extend(quests::objective_summary(state, quest_index));
                    messages.push(format!("Quest: {}", title));
                }
            }
            if !handled {
                messages.push(format!(
                    "{} has no unfinished deed to hear about.",
                    npc_name
                ));
            }
        }
        2 if view.options.len() > 2 => {
            if let Some(memory) = view.memory {
                messages.push(format!(
                    "{} searches your face, then recalls: {}",
                    npc_name, memory
                ));
            }
        }
        _ => return messages,
    }
    state_effects::advance_time(state, 1);
    state.character.turn += 1;
    messages.push("Time passes.".to_string());
    messages
}

#[cfg(test)]
#[path = "../../tests/unit/game/interactions.rs"]
mod tests;
