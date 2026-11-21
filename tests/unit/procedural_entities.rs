use super::*;
use crate::content::load_campaign_content;
use crate::model::{WorldGenerationMetadata, WorldMode};
use crate::procedural::{generate_world, WorldGenerationConfig};

fn generated_state() -> GameState {
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
    crate::procedural::place_authored_content(&mut world, &content);
    let character = world.spawn_character("Test".into(), "Warden".into());
    GameState {
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
    }
}

#[test]
fn generated_entities_are_deterministic() {
    let content = load_campaign_content();
    let mut a = generated_state();
    let mut b = generated_state();
    let first = populate_generated_entities(&mut a, &content);
    let second = populate_generated_entities(&mut b, &content);
    assert_eq!(first, second);
    assert_eq!(
        a.world
            .locations
            .iter()
            .map(|l| &l.name)
            .collect::<Vec<_>>(),
        b.world
            .locations
            .iter()
            .map(|l| &l.name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        a.factions.iter().map(|f| &f.name).collect::<Vec<_>>(),
        b.factions.iter().map(|f| &f.name).collect::<Vec<_>>()
    );
    assert_eq!(
        a.npcs
            .iter()
            .map(|npc| (&npc.name, npc.location_id, npc.faction_id))
            .collect::<Vec<_>>(),
        b.npcs
            .iter()
            .map(|npc| (&npc.name, npc.location_id, npc.faction_id))
            .collect::<Vec<_>>()
    );
}

#[test]
fn generated_entities_are_idempotent() {
    let content = load_campaign_content();
    let mut state = generated_state();
    let first = populate_generated_entities(&mut state, &content);
    let counts = (state.factions.len(), state.npcs.len());
    let second = populate_generated_entities(&mut state, &content);
    assert_ne!((first.0, first.1), (0, 0));
    assert_eq!(second, (0, 0));
    assert_eq!(state.factions.len(), counts.0);
    assert_eq!(state.npcs.len(), counts.1);
}

#[test]
fn generated_npcs_reference_generated_locations_and_factions() {
    let content = load_campaign_content();
    let mut state = generated_state();
    populate_generated_entities(&mut state, &content);
    for npc in &state.npcs {
        assert!(state.world.location_by_id(npc.location_id).is_some());
        if let Some(faction_id) = npc.faction_id {
            assert!(state
                .factions
                .iter()
                .any(|faction| faction.id == faction_id));
        }
    }
}
