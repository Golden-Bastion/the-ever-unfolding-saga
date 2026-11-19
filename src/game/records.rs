use crate::game::state_effects;
use crate::model::{GameState, Quest};
use crate::presentation::{
    CharacterView, InventoryDetailView, InventoryView, ItemView, QuestLogView, QuestObjectiveView,
    QuestView,
};

fn character_view(state: &GameState) -> CharacterView {
    CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    }
}

fn item_view(item: &crate::model::Item) -> ItemView {
    ItemView {
        id: item.id,
        name: item.name.clone(),
        description: item.description.clone(),
    }
}

pub(crate) fn build_inventory_view(state: &GameState) -> InventoryView {
    InventoryView {
        character: character_view(state),
        items: state.character.inventory.iter().map(item_view).collect(),
    }
}

pub(crate) fn build_inventory_detail_view(
    state: &GameState,
    selected: usize,
) -> Option<InventoryDetailView> {
    let item = state.character.inventory.get(selected)?;
    Some(InventoryDetailView {
        item: item_view(item),
        position: selected + 1,
        total: state.character.inventory.len(),
        art: state
            .campaign_content
            .as_ref()
            .and_then(|content| content.item_art_for(&item.name))
            .map(str::to_string),
    })
}

fn build_quest_view(quest: &Quest, tracked: bool) -> QuestView {
    let status = if quest.completed {
        "COMPLETED"
    } else if quest_is_ready(quest) {
        "READY"
    } else {
        "ACTIVE"
    };
    let objectives = quest
        .objectives
        .iter()
        .map(|objective| QuestObjectiveView {
            label: objective.display_label(),
            progress: objective.progress,
            required: objective.required,
            completed: objective.completed,
        })
        .collect();
    QuestView {
        id: quest.id,
        tracked: tracked && quest.offered && !quest.completed,
        title: quest.title.clone(),
        description: quest.description.clone(),
        objectives,
        status: status.to_string(),
        completed: quest.completed,
        reward_claimed: quest.reward_claimed,
        reward_item_name: (!quest.reward_item_name.trim().is_empty())
            .then(|| quest.reward_item_name.clone()),
    }
}

pub(crate) fn build_quest_log_view(state: &GameState) -> QuestLogView {
    QuestLogView {
        character: character_view(state),
        quests: state
            .quests
            .iter()
            .filter(|quest| quest.offered || quest.completed)
            .map(|quest| {
                build_quest_view(
                    quest,
                    state.character.tracked_quest_id == Some(quest.id),
                )
            })
            .collect(),
    }
}

pub(crate) fn build_quest_view_for(state: &GameState, quest_index: usize) -> Option<QuestView> {
    let quest = state
        .quests
        .iter()
        .filter(|quest| quest.offered || quest.completed)
        .nth(quest_index)?;
    Some(build_quest_view(
        quest,
        state.character.tracked_quest_id == Some(quest.id),
    ))
}

fn quest_is_ready(quest: &Quest) -> bool {
    !quest.completed
        && !quest.objectives.is_empty()
        && quest.objectives.iter().all(|objective| objective.completed)
}

pub(crate) fn record_journal_note(state: &mut GameState, note: &str) -> bool {
    if note.is_empty() {
        return false;
    }
    state.character.notes.push(note.to_string());
    state_effects::advance_time(state, 1);
    state.character.turn += 1;
    let character_name = state.character.display_name();
    state.world.record_history(
        state.character.turn,
        format!("{} noted: {}", character_name, note),
    );
    true
}
