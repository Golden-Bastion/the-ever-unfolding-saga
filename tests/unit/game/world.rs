use super::*;
use crate::model::{WorldGenerationMetadata, WorldMode};
use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};

#[test]
fn generated_world_bootstrap_populates_generated_entities() {
    let content = load_campaign_content();
    let seed = 9090;
    let config = WorldGenerationConfig::default();
    let mut world = generate_world("Test", seed, config);
    world.mode = WorldMode::New;
    world.generation = Some(WorldGenerationMetadata {
        seed,
        region_count: config.region_count,
        location_count: config.location_count,
        extra_edges: config.extra_edges,
        geography_ruleset_version: crate::geography::GEOGRAPHY_RULESET_VERSION,
    });
    place_authored_content(&mut world, &content);
    let character = world.spawn_character("Tester".into(), "Warden".into());
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
        campaign_content: Some(content),
    };

    let report = bootstrap_campaign_content(&mut state);
    assert!(report.factions_added > 0);
    assert!(report.npcs_added > 0);
    assert!(state
        .npcs
        .iter()
        .all(|npc| state.world.location_by_id(npc.location_id).is_some()));
}

#[test]
fn generated_bootstrap_is_idempotent() {
    let content = load_campaign_content();
    let seed = 9091;
    let config = WorldGenerationConfig::default();
    let mut world = generate_world("Test", seed, config);
    world.mode = WorldMode::New;
    world.generation = Some(WorldGenerationMetadata {
        seed,
        region_count: config.region_count,
        location_count: config.location_count,
        extra_edges: config.extra_edges,
        geography_ruleset_version: crate::geography::GEOGRAPHY_RULESET_VERSION,
    });
    place_authored_content(&mut world, &content);
    let character = world.spawn_character("Tester".into(), "Warden".into());
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
        campaign_content: Some(content),
    };

    let first = bootstrap_campaign_content(&mut state);
    let counts = (state.factions.len(), state.npcs.len());
    let second = bootstrap_campaign_content(&mut state);
    assert!(first.factions_added > 0);
    assert!(first.npcs_added > 0);
    assert_eq!((second.factions_added, second.npcs_added), (0, 0));
    assert_eq!(counts, (state.factions.len(), state.npcs.len()));
}
