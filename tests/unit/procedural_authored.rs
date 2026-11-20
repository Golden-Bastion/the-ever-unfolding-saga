use super::*;
use crate::content::load_campaign_content;
use crate::model::{WorldGenerationMetadata, WorldMode};
use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};

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
    place_authored_content(&mut world, &content);
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
fn authored_content_is_anchored_deterministically() {
    let mut first = generated_state();
    let mut second = generated_state();
    let first_content = first.campaign_content.clone().expect("content is loaded");
    let second_content = second.campaign_content.clone().expect("content is loaded");

    crate::game::world::bootstrap_campaign_content(&mut first);
    crate::game::world::bootstrap_campaign_content(&mut second);

    let first_markers = first
        .npcs
        .iter()
        .filter_map(|npc| {
            npc.memory
                .iter()
                .find(|entry| entry.starts_with(AUTHORED_ANCHOR_MARKER))
        })
        .cloned()
        .collect::<Vec<_>>();
    let second_markers = second
        .npcs
        .iter()
        .filter_map(|npc| {
            npc.memory
                .iter()
                .find(|entry| entry.starts_with(AUTHORED_ANCHOR_MARKER))
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(first_markers, second_markers);

    for faction in &first.factions {
        if first_content
            .factions
            .iter()
            .any(|authored| authored.name == faction.name)
        {
            assert!(faction
                .memory
                .iter()
                .any(|entry| entry.starts_with(AUTHORED_ANCHOR_MARKER)));
        }
    }
    assert_eq!(first_content.factions.len(), second_content.factions.len());
}

#[test]
fn authored_quest_references_remain_valid_after_generation() {
    let mut state = generated_state();
    let content = state.campaign_content.clone().expect("content is loaded");
    crate::game::world::bootstrap_campaign_content(&mut state);

    for quest in &content.quests {
        let runtime_quest = state
            .quests
            .iter()
            .find(|candidate| candidate.content_id == quest.id)
            .expect("authored quest should be present");
        assert_eq!(
            state
                .world
                .location_by_id(runtime_quest.target_location_id)
                .map(|location| location.name.as_str()),
            Some(quest.location_name.as_str())
        );
        assert!(state.factions.iter().any(|faction| {
            faction.id == runtime_quest.faction_id && faction.name == quest.faction_name
        }));
        assert!(state.npcs.iter().any(|npc| {
            npc.id == runtime_quest.giver_npc_id && npc.name == quest.giver_npc_name
        }));
    }
}
