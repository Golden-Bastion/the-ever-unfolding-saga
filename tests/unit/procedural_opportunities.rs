use super::*;
use crate::content::load_campaign_content;
use crate::model::{WorldGenerationMetadata, WorldMode};
use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};
use crate::procedural_entities::populate_generated_entities;
use crate::procedural_relationships::populate_generated_relationships;

fn prepared_state() -> GameState {
    let content = load_campaign_content();
    let seed = 9090;
    let config = WorldGenerationConfig::default();
    let mut world = generate_world("Ashen", seed, config);
    world.mode = WorldMode::New;
    world.generation = Some(WorldGenerationMetadata {
        seed,
        region_count: config.region_count,
        location_count: config.location_count,
        extra_edges: config.extra_edges,
        geography_ruleset_version: crate::geography::GEOGRAPHY_RULESET_VERSION,
    });
    place_authored_content(&mut world, &content);
    let character = world.spawn_character("Test".into(), "Warden".into());
    let mut state = GameState {
        world,
        character,
        threat: Default::default(),
        corpses: Vec::new(),
        factions: Vec::new(),
        npcs: Vec::new(),
        quests: Vec::new(),
        last_announced_location_id: None,
        rng_state: 0,
        campaign_content: Some(content.clone()),
    };
    populate_generated_entities(&mut state, &content);
    crate::procedural_authored::integrate_authored_content(&mut state, &content);
    populate_generated_relationships(&mut state);
    state
}

#[test]
fn generated_quest_references_valid_entities() {
    let mut state = prepared_state();
    populate_generated_opportunities(&mut state);
    assert!(state
        .campaign_content
        .as_ref()
        .unwrap()
        .events
        .iter()
        .any(|event| event.id.starts_with(EVENT_PREFIX)));
    let quest = state
        .quests
        .iter()
        .find(|q| q.content_id.starts_with(QUEST_PREFIX))
        .expect("generated quest");
    assert!(state
        .world
        .location_by_id(quest.target_location_id)
        .is_some());
    assert!(state.factions.iter().any(|f| f.id == quest.faction_id));
    let giver = state
        .npcs
        .iter()
        .find(|npc| npc.id == quest.giver_npc_id)
        .expect("generated quest giver");
    assert_eq!(giver.faction_id, Some(quest.faction_id));
    assert!(!quest.objectives.is_empty());
}

#[test]
fn completed_generated_quest_evolves_world() {
    let mut state = prepared_state();
    populate_generated_opportunities(&mut state);
    let index = state
        .quests
        .iter()
        .position(|q| q.content_id.starts_with(QUEST_PREFIX))
        .expect("generated quest");
    let location_id = state.quests[index].target_location_id;
    state
        .world
        .location_by_id_mut(location_id)
        .unwrap()
        .dangerous = true;
    state.quests[index].completed = true;
    assert_eq!(evolve_generated_world(&mut state), 1);
    assert!(!state.world.location_by_id(location_id).unwrap().dangerous);
    assert!(state
        .quests
        .iter()
        .any(|q| q.content_id.starts_with(EVOLUTION_QUEST_PREFIX)));
    assert_eq!(evolve_generated_world(&mut state), 0);
    assert!(evolution_already_processed(
        &state,
        &state.quests[index].content_id
    ));
}

#[test]
fn resolved_evolution_quest_is_not_reprocessed_even_if_location_becomes_dangerous_again() {
    let mut state = prepared_state();
    populate_generated_opportunities(&mut state);
    let index = state
        .quests
        .iter()
        .position(|q| q.content_id.starts_with(QUEST_PREFIX))
        .expect("generated quest");
    let location_id = state.quests[index].target_location_id;
    state
        .world
        .location_by_id_mut(location_id)
        .unwrap()
        .dangerous = true;
    state.quests[index].completed = true;
    assert_eq!(evolve_generated_world(&mut state), 1);
    state
        .world
        .location_by_id_mut(location_id)
        .unwrap()
        .dangerous = true;
    assert_eq!(evolve_generated_world(&mut state), 0);
    assert!(state.world.location_by_id(location_id).unwrap().dangerous);
}

#[test]
fn generated_state_survives_serialization() {
    let mut state = prepared_state();
    populate_generated_opportunities(&mut state);
    let index = state
        .quests
        .iter()
        .position(|q| q.content_id.starts_with(QUEST_PREFIX))
        .expect("generated quest");
    state.quests[index].completed = true;
    state
        .world
        .location_by_id_mut(state.quests[index].target_location_id)
        .unwrap()
        .dangerous = true;
    evolve_generated_world(&mut state);
    let restored: GameState = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert!(restored
        .quests
        .iter()
        .any(|q| q.content_id.starts_with(EVOLUTION_QUEST_PREFIX)));
    assert!(evolution_already_processed(
        &restored,
        &state.quests[index].content_id
    ));
}
