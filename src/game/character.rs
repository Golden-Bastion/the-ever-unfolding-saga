use crate::model::GameState;
use crate::presentation::{
    AttributesView, CharacterSheetView, CharacterView, ConditionView, EvidenceView, FactionView,
};

pub(crate) fn gain_experience(state: &mut GameState, amount: u32) {
    state.character.experience += amount;
    loop {
        let threshold = state.character.level * 50;
        if state.character.experience < threshold {
            break;
        }
        state.character.experience -= threshold;
        state.character.level += 1;
    }
}

pub(crate) fn build_character_sheet_view(state: &GameState) -> CharacterSheetView {
    let character = &state.character;
    CharacterSheetView {
        character: CharacterView {
            name: character.name.clone(),
            title: character.title.clone(),
            hp: character.hp,
            max_hp: character.max_hp,
        },
        level: character.level,
        experience: character.experience,
        next_level_experience: character.level * 50,
        attributes: AttributesView {
            might: character.attributes.might,
            insight: character.attributes.insight,
            endurance: character.attributes.endurance,
            effective_might: character.effective_might(),
            effective_insight: character.effective_insight(),
            effective_endurance: character.effective_endurance(),
        },
        conditions: character
            .conditions
            .iter()
            .map(|condition| ConditionView {
                name: condition.name.clone(),
                remaining: condition.remaining,
                penalty: condition.penalty,
                bonus: condition.bonus,
            })
            .collect(),
        factions: state
            .factions
            .iter()
            .map(|faction| FactionView {
                name: faction.name.clone(),
                reputation: faction.reputation,
                memories: faction
                    .memory
                    .iter()
                    .filter(|entry| !crate::model::is_internal_memory_entry(entry))
                    .cloned()
                    .collect(),
            })
            .collect(),
        evidence: character
            .known_evidence_ids
            .iter()
            .map(|evidence_id| {
                let definition = state.campaign_content.as_ref().and_then(|content| {
                    content
                        .world
                        .locations
                        .iter()
                        .flat_map(|location| location.spatial_discoveries.iter())
                        .find(|discovery| discovery.id == *evidence_id)
                });
                EvidenceView {
                    id: evidence_id.clone(),
                    title: definition
                        .map(|discovery| discovery.title.clone())
                        .unwrap_or_else(|| evidence_id.clone()),
                    description: definition
                        .map(|discovery| discovery.description.clone())
                        .unwrap_or_default(),
                }
            })
            .collect(),
        notes: character.notes.clone(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/game/character.rs"]
mod tests;
