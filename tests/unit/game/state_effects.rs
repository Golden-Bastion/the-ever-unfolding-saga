use super::*;
use crate::content::load_campaign_content;
use crate::model::{GameState, WorldGenerationMetadata, WorldMode};
use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};
use crate::procedural_entities::populate_generated_entities;
use crate::procedural_opportunities::populate_generated_opportunities;
use crate::procedural_relationships::populate_generated_relationships;

#[test]
fn advancing_time_applies_generated_world_evolution() {
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
    populate_generated_opportunities(&mut state);

    let quest_index = state
        .quests
        .iter()
        .position(|quest| quest.content_id.starts_with("generated.quest."))
        .expect("generated quest should exist");
    let target_id = state.quests[quest_index].target_location_id;
    state.quests[quest_index].completed = true;
    state.quests[quest_index].reward_claimed = true;
    advance_time(&mut state, 1);
    assert!(!state.world.location_by_id(target_id).unwrap().dangerous);
}
