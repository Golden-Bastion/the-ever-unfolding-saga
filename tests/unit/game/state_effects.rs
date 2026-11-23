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

#[test]
fn continuous_world_minutes_accumulate_into_existing_two_hour_time_points() {
    let mut state = crate::model::create_new_state(
        "Minute Clock",
        WorldMode::New,
        "Traveller".into(),
        "Wanderer".into(),
    );
    state.character.conditions.push(crate::model::Condition::new("Tired", 2, 0));
    let initial_points = state.world.time_points;

    advance_world_minutes(&mut state, 119);
    assert_eq!(state.world.time_points, initial_points);
    assert_eq!(state.world.minutes_into_time_point, 119);
    assert_eq!(state.character.conditions[0].remaining, 2);

    advance_world_minutes(&mut state, 1);
    assert_eq!(state.world.time_points, initial_points + 1);
    assert_eq!(state.world.minutes_into_time_point, 0);
    assert_eq!(state.character.conditions[0].remaining, 1);
}

#[test]
fn action_time_advancement_preserves_partial_continuous_minutes() {
    let mut state = crate::model::create_new_state(
        "Combined Clock",
        WorldMode::New,
        "Traveller".into(),
        "Wanderer".into(),
    );
    advance_world_minutes(&mut state, 75);
    let initial_points = state.world.time_points;

    advance_time(&mut state, 1);
    assert_eq!(state.world.time_points, initial_points + 1);
    assert_eq!(state.world.minutes_into_time_point, 75);

    advance_world_minutes(&mut state, 45);
    assert_eq!(state.world.time_points, initial_points + 2);
    assert_eq!(state.world.minutes_into_time_point, 0);
}

#[test]
fn continuous_minutes_roll_over_at_the_existing_day_boundary() {
    let mut state = crate::model::create_new_state(
        "Midnight Clock",
        WorldMode::New,
        "Traveller".into(),
        "Wanderer".into(),
    );
    state.world.day = 9;
    state.world.time_points = 11;
    state.world.minutes_into_time_point = 119;

    advance_world_minutes(&mut state, 1);

    assert_eq!(state.world.day, 10);
    assert_eq!(state.world.time_points, 0);
    assert_eq!(state.world.minutes_into_time_point, 0);
}

#[test]
fn world_minute_offset_round_trips_and_legacy_saves_default_to_zero() {
    let mut world = crate::model::World::new("Minute Clock", WorldMode::New);
    world.minutes_into_time_point = 47;

    let value = serde_json::to_value(&world).unwrap();
    let restored: crate::model::World = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(restored.minutes_into_time_point, 47);

    let mut legacy_value = value;
    legacy_value
        .as_object_mut()
        .unwrap()
        .remove("minutes_into_time_point");
    let legacy: crate::model::World = serde_json::from_value(legacy_value).unwrap();
    assert_eq!(legacy.minutes_into_time_point, 0);
}

#[test]
fn large_elapsed_time_advances_wrap_the_clock_without_integer_overflow() {
    let mut state = crate::model::create_new_state(
        "Long Journey",
        WorldMode::New,
        "Traveller".into(),
        "Wanderer".into(),
    );
    state.world.day = u32::MAX - 1;
    state.world.time_points = 11;

    advance_time(&mut state, u32::MAX);

    assert_eq!(state.world.day, u32::MAX);
    assert_eq!(state.world.time_points, 2);
}
