use crate::game::character;
use crate::model::{EntityId, GameState, Item, QuestObjective, QuestObjectiveKind};

fn quest_key(state: &GameState, quest_index: usize) -> Option<String> {
    state.quests.get(quest_index).map(|quest| {
        if quest.content_id.is_empty() {
            format!("legacy.quest.{}", quest.id)
        } else {
            quest.content_id.clone()
        }
    })
}

pub(crate) fn normalize_quest(state: &mut GameState, quest_index: usize) -> bool {
    let Some(quest_snapshot) = state.quests.get(quest_index).cloned() else {
        return false;
    };
    if !quest_snapshot.objectives.is_empty() {
        return false;
    }

    let target_location = state
        .world
        .location_by_id(quest_snapshot.target_location_id)
        .map(|location| location.name.clone())
        .unwrap_or_else(|| quest_snapshot.target_location_id.to_string());
    let mut objectives = vec![QuestObjective::new(
        QuestObjectiveKind::VisitLocation,
        target_location,
        1,
    )];
    if !quest_snapshot.required_item_name.trim().is_empty() {
        objectives.push(QuestObjective::new(
            QuestObjectiveKind::AcquireItem,
            quest_snapshot.required_item_name,
            1,
        ));
    }
    if let Some(enemy_name) = state
        .campaign_content
        .as_ref()
        .and_then(|content| {
            state
                .world
                .location_by_id(quest_snapshot.target_location_id)
                .and_then(|location| content.encounter_for(&location.name))
        })
        .map(|encounter| encounter.enemy_name.clone())
    {
        objectives.push(QuestObjective::new(
            QuestObjectiveKind::DefeatEnemy,
            enemy_name,
            1,
        ));
    }
    if let Some(quest) = state.quests.get_mut(quest_index) {
        quest.objectives = objectives;
    }
    true
}

pub(crate) fn normalize_all(state: &mut GameState) -> usize {
    let mut normalized = 0;
    for index in 0..state.quests.len() {
        if normalize_quest(state, index) {
            normalized += 1;
        }
    }
    normalized
}

pub(crate) fn sync_active_quests(state: &mut GameState) {
    normalize_all(state);
    for index in 0..state.quests.len() {
        if state.quests[index].offered && !state.quests[index].completed {
            sync_objectives(state, index);
        }
    }
}

pub(crate) fn sync_objectives(state: &mut GameState, quest_index: usize) {
    let (offered, completed, objectives) = match state.quests.get(quest_index) {
        Some(quest) => (quest.offered, quest.completed, quest.objectives.clone()),
        None => return,
    };
    if !offered || completed {
        return;
    }

    let current_location_name = state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| location.name.clone());
    let mut updates = Vec::with_capacity(objectives.len());
    for objective in objectives {
        let progress = match objective.kind {
            QuestObjectiveKind::AcquireItem => state
                .character
                .inventory
                .iter()
                .filter(|item| item.name == objective.target)
                .count() as u32,
            QuestObjectiveKind::VisitLocation => current_location_name
                .as_deref()
                .filter(|location| *location == objective.target)
                .map(|_| objective.required)
                .unwrap_or(objective.progress),
            QuestObjectiveKind::DefeatEnemy => objective.progress,
            QuestObjectiveKind::DiscoverEvidence => {
                if state.character.knows_evidence(&objective.target) {
                    objective.required
                } else {
                    objective.progress
                }
            }
        };
        updates.push(progress.max(objective.progress).min(objective.required));
    }

    if let Some(quest) = state.quests.get_mut(quest_index) {
        for (objective, progress) in quest.objectives.iter_mut().zip(updates) {
            objective.progress = progress;
            objective.completed = progress >= objective.required;
        }
    }
}

pub(crate) fn record_enemy_defeat(state: &mut GameState, enemy_name: &str, location_id: EntityId) {
    normalize_all(state);
    for index in 0..state.quests.len() {
        let relevant = {
            let quest = &state.quests[index];
            quest.offered
                && !quest.completed
                && quest.target_location_id == location_id
                && quest.objectives.iter().any(|objective| {
                    objective.kind == QuestObjectiveKind::DefeatEnemy
                        && objective.target == enemy_name
                        && !objective.completed
                })
        };
        if !relevant {
            continue;
        }
        if let Some(quest) = state.quests.get_mut(index) {
            for objective in &mut quest.objectives {
                if objective.kind == QuestObjectiveKind::DefeatEnemy
                    && objective.target == enemy_name
                    && !objective.completed
                {
                    objective.progress =
                        objective.progress.saturating_add(1).min(objective.required);
                    objective.completed = objective.progress >= objective.required;
                }
            }
        }
        sync_objectives(state, index);
    }
}

pub(crate) fn objective_summary(state: &GameState, quest_index: usize) -> Vec<String> {
    let Some(quest) = state.quests.get(quest_index) else {
        return Vec::new();
    };
    quest
        .objectives
        .iter()
        .map(|objective| {
            let marker = if objective.completed { "x" } else { " " };
            format!(
                "[{}] {} ({}/{})",
                marker,
                objective.display_label(),
                objective.progress,
                objective.required
            )
        })
        .collect()
}

pub(crate) fn try_complete(state: &mut GameState, quest_index: usize) -> bool {
    normalize_quest(state, quest_index);
    sync_objectives(state, quest_index);
    let Some(quest_snapshot) = state.quests.get(quest_index).cloned() else {
        return false;
    };
    if !quest_snapshot.offered
        || quest_snapshot.completed
        || quest_snapshot.objectives.is_empty()
        || !quest_snapshot
            .objectives
            .iter()
            .all(|objective| objective.completed)
    {
        return false;
    }

    let Some(key) = quest_key(state, quest_index) else {
        return false;
    };
    if state
        .world
        .completed_quest_ids
        .iter()
        .any(|known| known == &key)
    {
        return false;
    }

    let mut consumed = Vec::new();
    for objective in &quest_snapshot.objectives {
        if objective.kind != QuestObjectiveKind::AcquireItem {
            continue;
        }
        for _ in 0..objective.required {
            let Some(item_index) = state
                .character
                .inventory
                .iter()
                .position(|item| item.name == objective.target)
            else {
                return false;
            };
            consumed.push(state.character.inventory.remove(item_index));
        }
    }

    let character_name = state.character.display_name();
    if let Some(quest) = state.quests.get_mut(quest_index) {
        quest.completed = true;
        quest.reward_claimed = true;
        quest.completed_by = Some(character_name.clone());
    }
    state.world.completed_quest_ids.push(key.clone());

    let title = quest_snapshot.title.clone();
    let faction_id = quest_snapshot.faction_id;
    let giver_npc_id = quest_snapshot.giver_npc_id;
    let location_name = state
        .world
        .location_by_id(quest_snapshot.target_location_id)
        .map(|location| location.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let memory = format!("{} completed the quest {}.", character_name, title);

    adjust_faction_reputation(state, faction_id, 5, memory.clone());
    remember_npc(state, giver_npc_id, memory.clone());
    remember_faction(state, faction_id, memory.clone());
    state.world.record_event_history(
        state.character.turn,
        format!("quest.{}.completed", key),
        location_name,
        memory,
    );

    let reward_name = quest_snapshot.reward_item_name.trim().to_string();
    let reward_name = if reward_name.is_empty() {
        "Unnamed Reward".to_string()
    } else {
        reward_name
    };
    let reward = Item {
        id: state.world.allocate_id(),
        name: reward_name,
        description: format!("A token earned by completing {}.", title),
    };
    state.character.inventory.push(reward.clone());
    character::gain_experience(state, 25);
    true
}

fn adjust_faction_reputation(
    state: &mut GameState,
    faction_id: EntityId,
    delta: i32,
    memory: String,
) {
    if let Some(faction) = state
        .factions
        .iter_mut()
        .find(|faction| faction.id == faction_id)
    {
        faction.reputation += delta;
        push_memory(&mut faction.memory, memory);
    }
}

fn remember_faction(state: &mut GameState, faction_id: EntityId, memory: String) {
    if let Some(faction) = state
        .factions
        .iter_mut()
        .find(|faction| faction.id == faction_id)
    {
        push_memory(&mut faction.memory, memory);
    }
}

fn remember_npc(state: &mut GameState, npc_id: EntityId, memory: String) {
    if let Some(npc) = state.npcs.iter_mut().find(|npc| npc.id == npc_id) {
        push_memory(&mut npc.memory, memory);
    }
}

fn push_memory(memory: &mut Vec<String>, value: String) {
    memory.push(value);
    if memory.len() > 5 {
        let remove_count = memory.len() - 5;
        memory.drain(0..remove_count);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/game/quests.rs"]
mod tests;
