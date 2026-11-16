use crate::content::load_campaign_content;
use crate::model::{Faction, GameState, Npc, Quest};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CampaignSeedReport {
    pub locations_added: usize,
    pub factions_added: usize,
    pub npcs_added: usize,
    pub quests_added: usize,
}

fn faction_id_by_name(state: &GameState, faction_name: &str) -> Option<crate::model::EntityId> {
    state
        .factions
        .iter()
        .find(|faction| faction.name == faction_name)
        .map(|faction| faction.id)
}

pub(crate) fn bootstrap_campaign_content(state: &mut GameState) -> CampaignSeedReport {
    let content = state
        .campaign_content
        .clone()
        .unwrap_or_else(load_campaign_content);
    let mut report = CampaignSeedReport {
        locations_added: if state.world.generation.is_some() {
            0
        } else {
            content.seed_world(&mut state.world)
        },
        ..Default::default()
    };
    state.campaign_content = Some(content.clone());

    for faction_content in &content.factions {
        if state
            .factions
            .iter()
            .any(|faction| faction.name == faction_content.name)
        {
            continue;
        }
        let id = state.world.allocate_id();
        state
            .factions
            .push(Faction::new(id, faction_content.name.clone()));
        report.factions_added += 1;
    }

    for npc_content in &content.npcs {
        if state.npcs.iter().any(|npc| npc.name == npc_content.name) {
            continue;
        }
        let Some(location_id) = state
            .world
            .location_by_name(&npc_content.location_name)
            .map(|location| location.id)
        else {
            continue;
        };
        let faction_id = npc_content
            .faction_name
            .as_deref()
            .and_then(|name| faction_id_by_name(state, name));
        let id = state.world.allocate_id();
        let mut npc = Npc::new(
            id,
            npc_content.name.clone(),
            npc_content.title.clone(),
            location_id,
            faction_id,
        );
        npc.memory = npc_content.memory.clone();
        state.npcs.push(npc);
        report.npcs_added += 1;
    }

    for quest_content in &content.quests {
        if state
            .quests
            .iter()
            .any(|quest| quest.content_id == quest_content.id)
        {
            continue;
        }
        let Some(target_location_id) = state
            .world
            .location_by_name(&quest_content.location_name)
            .map(|location| location.id)
        else {
            continue;
        };
        let Some(faction_id) = faction_id_by_name(state, &quest_content.faction_name) else {
            continue;
        };
        let Some(giver_npc_id) = state
            .npcs
            .iter()
            .find(|npc| npc.name == quest_content.giver_npc_name)
            .map(|npc| npc.id)
        else {
            continue;
        };
        let id = state.world.allocate_id();
        let quest_index = state.quests.len();
        state.quests.push(Quest::new(
            id,
            quest_content.id.clone(),
            quest_content.title.clone(),
            quest_content.description.clone(),
            target_location_id,
            faction_id,
            giver_npc_id,
            quest_content.required_item_name.clone(),
            quest_content.reward_item_name.clone(),
        ));

        if !quest_content.required_evidence_ids.is_empty() {
            // Build the standard visit/item/encounter objectives first, then
            // append authored evidence goals so this quest keeps every path.
            crate::game::quests::normalize_quest(state, quest_index);
            let evidence_objectives = quest_content
                .required_evidence_ids
                .iter()
                .map(|evidence_id| {
                    let label = content
                        .world
                        .locations
                        .iter()
                        .flat_map(|location| location.spatial_discoveries.iter())
                        .find(|discovery| discovery.id == *evidence_id)
                        .map(|discovery| discovery.title.clone())
                        .unwrap_or_else(|| evidence_id.clone());
                    crate::model::QuestObjective::discover_evidence(evidence_id.clone(), label)
                })
                .collect::<Vec<_>>();
            state.quests[quest_index]
                .objectives
                .extend(evidence_objectives);
        }
        report.quests_added += 1;
    }

    if state.world.generation.is_some() {
        let (generated_factions, generated_npcs) =
            crate::procedural_entities::populate_generated_entities(state, &content);
        report.factions_added += generated_factions;
        report.npcs_added += generated_npcs;
        crate::procedural_authored::integrate_authored_content(state, &content);
        crate::procedural_relationships::populate_generated_relationships(state);
        crate::procedural_opportunities::populate_generated_opportunities(state);
    }

    crate::game::quests::normalize_all(state);
    report
}

pub(crate) fn validate_loaded_state(state: &GameState) -> Vec<String> {
    let mut warnings = Vec::new();
    if !state.character.alive && state.character.hp > 0 {
        warnings.push("character is marked dead while still having HP".to_string());
    }
    if state
        .world
        .location_by_id(state.character.location_id)
        .is_none()
    {
        warnings.push(format!(
            "character references unknown location id {}",
            state.character.location_id
        ));
    }
    for npc in &state.npcs {
        if state.world.location_by_id(npc.location_id).is_none() {
            warnings.push(format!(
                "npc {} references unknown location id {}",
                npc.name, npc.location_id
            ));
        }
        if let Some(faction_id) = npc.faction_id {
            if !state
                .factions
                .iter()
                .any(|faction| faction.id == faction_id)
            {
                warnings.push(format!(
                    "npc {} references unknown faction id {}",
                    npc.name, faction_id
                ));
            }
        }
    }
    for quest in &state.quests {
        if state
            .world
            .location_by_id(quest.target_location_id)
            .is_none()
        {
            warnings.push(format!(
                "quest {} references unknown target location id {}",
                quest.title, quest.target_location_id
            ));
        }
        if !state
            .factions
            .iter()
            .any(|faction| faction.id == quest.faction_id)
        {
            warnings.push(format!(
                "quest {} references unknown faction id {}",
                quest.title, quest.faction_id
            ));
        }
        if !state.npcs.iter().any(|npc| npc.id == quest.giver_npc_id) {
            warnings.push(format!(
                "quest {} references unknown giver npc id {}",
                quest.title, quest.giver_npc_id
            ));
        }
        for objective in &quest.objectives {
            if objective.target.trim().is_empty() {
                warnings.push(format!(
                    "quest {} contains an objective with an empty target",
                    quest.title
                ));
            }
            if objective.required == 0 {
                warnings.push(format!(
                    "quest {} contains an objective with zero required progress",
                    quest.title
                ));
            }
            if objective.progress > objective.required {
                warnings.push(format!(
                    "quest {} objective {} exceeds required progress",
                    quest.title, objective.target
                ));
            }
        }
    }
    warnings
}

#[cfg(test)]
#[path = "../../tests/unit/game/world.rs"]
mod tests;
