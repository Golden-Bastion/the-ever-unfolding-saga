use super::*;
use crate::content::load_campaign_content;
use crate::model::{EntityId, GameState, WorldGenerationMetadata, WorldMode};
use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};
use crate::procedural_entities::populate_generated_entities;

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
fn generated_relationships_are_deterministic_and_idempotent() {
    let content = load_campaign_content();
    let mut first_state = generated_state();
    populate_generated_entities(&mut first_state, &content);
    let first_added = populate_generated_relationships(&mut first_state);
    let first_memories = relationship_memories(&first_state);

    let mut second_state = generated_state();
    populate_generated_entities(&mut second_state, &content);
    let second_added = populate_generated_relationships(&mut second_state);
    assert_eq!(first_added, second_added);
    assert_eq!(first_memories, relationship_memories(&second_state));

    let repeated = populate_generated_relationships(&mut first_state);
    assert_eq!(repeated, 0);
    assert_eq!(first_memories, relationship_memories(&first_state));
}

#[test]
fn generated_relationships_reference_existing_factions() {
    let content = load_campaign_content();
    let mut state = generated_state();
    populate_generated_entities(&mut state, &content);
    let added = populate_generated_relationships(&mut state);
    assert!(added > 0);

    for faction in &state.factions {
        for memory in faction
            .memory
            .iter()
            .filter(|entry| entry.starts_with(RELATIONSHIP_MARKER))
        {
            let Some(target) = memory.split(" with ").nth(1) else {
                panic!("relationship memory should contain a target");
            };
            let target = target.split(" (strength").next().unwrap_or_default();
            assert!(state
                .factions
                .iter()
                .any(|candidate| candidate.name == target));
        }
    }
}

#[test]
fn generated_factions_with_locations_have_representative_npcs() {
    let content = load_campaign_content();
    let mut state = generated_state();
    populate_generated_entities(&mut state, &content);

    for faction in state
        .factions
        .iter()
        .filter(|faction| is_generated_faction(faction))
    {
        let Some(region) = state
            .world
            .regions
            .iter()
            .find(|region| faction.name.ends_with(&format!("of {}", region.name)))
        else {
            continue;
        };
        if !state
            .world
            .locations
            .iter()
            .any(|location| location.region_id == region.id)
        {
            continue;
        }
        assert!(state
            .npcs
            .iter()
            .any(|npc| npc.faction_id == Some(faction.id)));
    }
}

#[test]
fn authored_and_generated_factions_can_share_relationships() {
    let mut state = generated_state();
    crate::game::world::bootstrap_campaign_content(&mut state);

    let content = state.campaign_content.clone().expect("content is loaded");
    let authored_index = state
        .factions
        .iter()
        .position(|faction| {
            content
                .factions
                .iter()
                .any(|authored| authored.name == faction.name)
                && faction
                    .memory
                    .iter()
                    .any(|entry| entry.starts_with("[authored anchor]"))
        })
        .expect("bootstrap should create an anchored authored faction");
    let authored_name = state.factions[authored_index].name.clone();
    let authored_region_name = state.factions[authored_index]
        .memory
        .iter()
        .find_map(|entry| {
            entry.strip_prefix(&format!(
                "[authored anchor] {authored_name} is anchored in "
            ))
        })
        .map(|region| region.trim_end_matches('.').to_string())
        .expect("authored faction should identify its anchor region");

    let generated_name = format!("Test Generated of {authored_region_name}");
    let generated_id = state.world.allocate_id();
    let mut generated_faction = Faction::new(generated_id, generated_name.clone());
    generated_faction
        .memory
        .push("A generated faction shaped by a test fixture.".to_string());
    state.factions.push(generated_faction);

    let added = populate_generated_relationships(&mut state);
    assert!(added > 0);

    let authored = &state.factions[authored_index];
    assert!(authored.memory.iter().any(|entry| {
        entry.starts_with(RELATIONSHIP_MARKER) && entry.contains(&generated_name)
    }));
    let generated = state
        .factions
        .iter()
        .find(|faction| faction.name == generated_name)
        .expect("generated faction should remain present");
    assert!(generated
        .memory
        .iter()
        .any(|entry| { entry.starts_with(RELATIONSHIP_MARKER) && entry.contains(&authored_name) }));
}

fn relationship_memories(state: &GameState) -> Vec<(EntityId, Vec<String>)> {
    state
        .factions
        .iter()
        .filter(|faction| {
            is_generated_faction(faction)
                || faction
                    .memory
                    .iter()
                    .any(|entry| entry.starts_with("[authored anchor]"))
        })
        .map(|faction| {
            (
                faction.id,
                faction
                    .memory
                    .iter()
                    .filter(|entry| entry.starts_with(RELATIONSHIP_MARKER))
                    .cloned()
                    .collect(),
            )
        })
        .collect()
}
