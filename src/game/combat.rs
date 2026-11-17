use super::{character, interactions, legacy, state_effects};
use crate::model::{EntityId, GameState, Item};
use crate::presentation::{CharacterView, CombatResultView, CombatView, CombatantView};
use crate::spatial::SpatialEncounterStatus;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombatEncounter {
    pub(crate) enemy_name: String,
    pub(crate) enemy_hp: i32,
    pub(crate) enemy_max_hp: i32,
    pub(crate) enemy_power: i32,
    pub(crate) enemy_id: EntityId,
    pub(crate) location_id: EntityId,
    pub(crate) location_name: String,
    pub(crate) spatial_encounter_id: Option<String>,
    pub(crate) spatial_quest_id: Option<String>,
    trophy_name: String,
    pub(crate) events: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CombatStartError {
    MissingLocation,
    MissingSpatialEncounter,
    SpatialEncounterOutOfReach,
}

impl fmt::Display for CombatStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLocation => formatter.write_str("The threat cannot be reached here."),
            Self::MissingSpatialEncounter => {
                formatter.write_str("That encounter is no longer available here.")
            }
            Self::SpatialEncounterOutOfReach => {
                formatter.write_str("Move closer before confronting that encounter.")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CombatResult {
    Victory,
    Fled,
    Defeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CombatStep {
    Continue,
    Result {
        view: Box<CombatResultView>,
        outcome: CombatResult,
    },
}

/// Start combat against the exact persistent encounter selected in spatial play.
pub(crate) fn start_spatial_encounter(
    state: &mut GameState,
    encounter_id: &str,
) -> Result<CombatEncounter, CombatStartError> {
    let location = state
        .world
        .location_by_id(state.character.location_id)
        .cloned()
        .ok_or(CombatStartError::MissingLocation)?;
    let map_id = format!("location.{}", location.id);
    let map = state
        .world
        .local_maps
        .iter()
        .find(|map| map.location_id == map_id)
        .ok_or(CombatStartError::MissingSpatialEncounter)?;
    let spatial_encounter = map
        .encounters
        .iter()
        .find(|encounter| encounter.id == encounter_id)
        .filter(|encounter| encounter.status == SpatialEncounterStatus::Active)
        .cloned()
        .ok_or(CombatStartError::MissingSpatialEncounter)?;

    let player_position = state
        .character
        .exploration_session
        .as_ref()
        .filter(|session| session.location_id == map_id)
        .and(state.character.exploration_position)
        .map(|position| (position.x, position.y))
        .unwrap_or((f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5));
    let can_confront =
        crate::interaction_actions::discover_available_interactions(map, state, player_position)
            .iter()
            .any(|interaction| {
                interaction.action
                    == crate::interaction_actions::InteractionActionKind::EngageEncounter
                    && interaction.target
                        == crate::interaction_actions::InteractionTarget::Encounter(
                            encounter_id.into(),
                        )
            });
    if !can_confront {
        return Err(CombatStartError::SpatialEncounterOutOfReach);
    }

    let character_name = state.character.display_name();
    let enemy_name = spatial_encounter.enemy_name.clone();
    Ok(CombatEncounter {
        enemy_name: enemy_name.clone(),
        enemy_hp: spatial_encounter.current_hp,
        enemy_max_hp: spatial_encounter.max_hp,
        enemy_power: spatial_encounter.power,
        enemy_id: state.world.allocate_id(),
        location_id: location.id,
        location_name: location.name,
        spatial_encounter_id: Some(spatial_encounter.id),
        spatial_quest_id: spatial_encounter.quest_id,
        trophy_name: spatial_encounter.trophy_item_name,
        events: vec![format!(
            "{} confronts {} at {}.",
            character_name,
            enemy_name,
            location_name_for_message(state)
        )],
    })
}

pub(crate) fn resolve_action(
    state: &mut GameState,
    encounter: &mut CombatEncounter,
    action: usize,
) -> CombatStep {
    match action {
        0 => {
            state_effects::advance_time(state, 1);
            state.character.turn += 1;
            let damage = (3 + state.character.effective_might()).max(1);
            encounter.enemy_hp = (encounter.enemy_hp - damage).max(0);
            encounter.events.push(format!(
                "You strike {} for {} damage.",
                encounter.enemy_name, damage
            ));
            let character_name = state.character.display_name();
            state.world.record_history(
                state.character.turn,
                format!(
                    "{} struck {} for {} damage.",
                    character_name, encounter.enemy_name, damage
                ),
            );
            if encounter.enemy_hp <= 0 {
                return finish_victory(state, encounter);
            }
            let retaliation = encounter.enemy_power;
            encounter.events.push(take_combat_damage(
                state,
                retaliation,
                &encounter.enemy_name,
                &encounter.location_name,
            ));
        }
        1 => {
            state_effects::advance_time(state, 1);
            state.character.turn += 1;
            let retaliation =
                (encounter.enemy_power - 1 - state.character.attributes.endurance / 2).max(0);
            let character_name = state.character.display_name();
            state.world.record_history(
                state.character.turn,
                format!(
                    "{} guarded against {}.",
                    character_name, encounter.enemy_name
                ),
            );
            encounter.events.push(format!(
                "You guard. Incoming damage is reduced to {}.",
                retaliation
            ));
            if retaliation > 0 {
                encounter.events.push(take_combat_damage(
                    state,
                    retaliation,
                    &encounter.enemy_name,
                    &encounter.location_name,
                ));
            } else {
                encounter
                    .events
                    .push("The blow glances off harmlessly.".to_string());
            }
        }
        2 => {
            state_effects::advance_time(state, 1);
            state.character.turn += 1;
            let character_name = state.character.display_name();
            state.world.record_history(
                state.character.turn,
                format!(
                    "{} fled from {} at {}.",
                    character_name, encounter.enemy_name, encounter.location_name
                ),
            );
            encounter.events.push(format!(
                "You flee. {} remains in {}.",
                encounter.enemy_name, encounter.location_name
            ));
            trim_combat_events(&mut encounter.events);
            return CombatStep::Result {
                view: Box::new(build_result_view(
                    state,
                    encounter,
                    &encounter.location_name,
                    "Fled",
                    "The threat remains.",
                )),
                outcome: CombatResult::Fled,
            };
        }
        _ => return CombatStep::Continue,
    }

    persist_spatial_encounter_health(state, encounter);
    if state.character.hp <= 0 {
        return finish_defeat(state, encounter);
    }

    trim_combat_events(&mut encounter.events);
    CombatStep::Continue
}

pub(crate) fn build_combat_view(state: &GameState, encounter: &CombatEncounter) -> CombatView {
    CombatView {
        character: character_view(state),
        player_condition: active_condition(state).map(str::to_string),
        enemy: CombatantView {
            name: encounter.enemy_name.clone(),
            current_hp: encounter.enemy_hp,
            max_hp: encounter.enemy_max_hp,
        },
        enemy_power: encounter.enemy_power,
        location_name: encounter.location_name.clone(),
        turn: state.character.turn,
        events: encounter.events.clone(),
        actions: vec![
            "Attack".to_string(),
            "Guard".to_string(),
            "Flee".to_string(),
        ],
    }
}

fn finish_victory(state: &mut GameState, encounter: &mut CombatEncounter) -> CombatStep {
    let enemy_name = encounter.enemy_name.clone();
    let character_name = state.character.display_name();
    if let Some(encounter_id) = encounter.spatial_encounter_id.as_deref() {
        set_spatial_encounter_defeated(state, encounter.location_id, encounter_id);
    }
    if state.threat.active
        && state
            .threat
            .source_location_id
            .is_none_or(|source| source == encounter.location_id)
    {
        state.threat.clear();
    }
    if let Some(location) = state.world.location_by_id_mut(encounter.location_id) {
        location.dangerous = false;
    }
    state.character.turn += 1;
    state.world.record_history(
        state.character.turn,
        format!(
            "{} defeated {} at {}.",
            character_name, enemy_name, encounter.location_name
        ),
    );
    let trophy = Item {
        id: encounter.enemy_id,
        name: encounter.trophy_name.clone(),
        description: format!(
            "A proof that the {} was confronted and survived.",
            encounter.location_name
        ),
    };
    state.character.inventory.push(trophy.clone());
    interactions::grant_reward_reputation(state, &trophy);
    interactions::update_faction_memory_for_location(
        state,
        encounter.location_id,
        format!("{} was cleared of danger.", encounter.location_name),
    );
    crate::game::quests::record_spatial_encounter_defeat(
        state,
        &enemy_name,
        encounter.location_id,
        encounter.spatial_quest_id.as_deref(),
    );
    character::gain_experience(state, 15);
    let result_note = format!(
        "Loot: {}\n{}\n\nThe threat is broken. The place is quieter now.",
        trophy.name, trophy.description
    );
    encounter.events.push("Victory".to_string());
    encounter.events.push(format!("Defeated: {}", enemy_name));
    encounter.events.push(format!("Loot: {}", trophy.name));
    encounter
        .events
        .push(format!("Description: {}", trophy.description));
    encounter
        .events
        .push("The threat is broken. The place is quieter now.".to_string());
    trim_combat_events(&mut encounter.events);
    CombatStep::Result {
        view: Box::new(build_result_view(
            state,
            encounter,
            &encounter.location_name,
            "Victory",
            &result_note,
        )),
        outcome: CombatResult::Victory,
    }
}

fn finish_defeat(state: &mut GameState, encounter: &mut CombatEncounter) -> CombatStep {
    let location_name = encounter.location_name.clone();
    legacy::mark_character_dead(
        state,
        format!("{} overcame them", encounter.enemy_name),
        &location_name,
    );
    encounter.events.push("Defeat".to_string());
    encounter.events.push("You were overwhelmed.".to_string());
    trim_combat_events(&mut encounter.events);
    CombatStep::Result {
        view: Box::new(build_result_view(
            state,
            encounter,
            &location_name,
            "Defeat",
            "You were overwhelmed.",
        )),
        outcome: CombatResult::Defeat,
    }
}

fn build_result_view(
    state: &GameState,
    encounter: &CombatEncounter,
    location_name: &str,
    result_title: &str,
    result_note: &str,
) -> CombatResultView {
    let _ = location_name;
    CombatResultView {
        combat: build_combat_view(state, encounter),
        result_title: result_title.to_string(),
        result_note: result_note.to_string(),
    }
}

fn character_view(state: &GameState) -> CharacterView {
    CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    }
}

fn active_condition(state: &GameState) -> Option<&str> {
    state
        .character
        .conditions
        .first()
        .map(|condition| condition.name.as_str())
}

fn trim_combat_events(events: &mut Vec<String>) {
    const MAX_EVENTS: usize = 12;
    if events.len() > MAX_EVENTS {
        let excess = events.len() - MAX_EVENTS;
        events.drain(0..excess);
    }
}

pub(crate) fn encounter_profile(
    state: &GameState,
    location_name: &str,
) -> (String, String, i32, i32, String) {
    if let Some(profile) = state
        .campaign_content
        .as_ref()
        .and_then(|content| content.encounter_for(location_name))
    {
        (
            profile.enemy_name.clone(),
            profile.description.clone(),
            profile.enemy_hp,
            profile.enemy_power,
            profile.trophy_item_name.clone(),
        )
    } else {
        (
            "Ash-Crazed Marauder".to_string(),
            "A hostile wanderer driven to violence by the ash.".to_string(),
            7,
            2,
            "Marauder's Token".to_string(),
        )
    }
}

fn persist_spatial_encounter_health(state: &mut GameState, encounter: &CombatEncounter) {
    let Some(encounter_id) = encounter.spatial_encounter_id.as_deref() else {
        return;
    };
    let map_id = format!("location.{}", encounter.location_id);
    if let Some(spatial_encounter) = state
        .world
        .local_maps
        .iter_mut()
        .find(|map| map.location_id == map_id)
        .and_then(|map| {
            map.encounters
                .iter_mut()
                .find(|entry| entry.id == encounter_id)
        })
    {
        spatial_encounter.current_hp = encounter.enemy_hp.clamp(0, spatial_encounter.max_hp);
        if spatial_encounter.current_hp == 0 {
            spatial_encounter.status = SpatialEncounterStatus::Defeated;
        }
    }
}

fn set_spatial_encounter_defeated(
    state: &mut GameState,
    location_id: EntityId,
    encounter_id: &str,
) {
    let map_id = format!("location.{location_id}");
    if let Some(spatial_encounter) = state
        .world
        .local_maps
        .iter_mut()
        .find(|map| map.location_id == map_id)
        .and_then(|map| {
            map.encounters
                .iter_mut()
                .find(|entry| entry.id == encounter_id)
        })
    {
        spatial_encounter.current_hp = 0;
        spatial_encounter.status = SpatialEncounterStatus::Defeated;
    }
}

fn take_combat_damage(
    state: &mut GameState,
    damage: i32,
    enemy_name: &str,
    location_name: &str,
) -> String {
    if damage <= 0 {
        return "The blow glances off harmlessly.".to_string();
    }
    state.character.hp -= damage;
    let character_name = state.character.display_name();
    state.world.record_history(
        state.character.turn,
        format!(
            "{} took {} damage from {} at {}.",
            character_name, damage, enemy_name, location_name
        ),
    );
    format!("You take {} damage from {}.", damage, enemy_name)
}

fn location_name_for_message(state: &GameState) -> String {
    state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| location.name.clone())
        .unwrap_or_else(|| "unknown location".to_string())
}
