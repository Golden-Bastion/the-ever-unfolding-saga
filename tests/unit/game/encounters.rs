use super::*;
use crate::content::definitions::{EncounterContent, EncounterSpawnConditions};
use crate::model::{Condition, WorldMode};
use crate::spatial::{generate_local_map, LocalMapConfig};

fn fixture() -> (GameState, Location, LocalMap, String) {
    let state = crate::model::create_new_state(
        "Encounter Spawning Test",
        WorldMode::New,
        "Tester".into(),
        "Wanderer".into(),
    );
    let location = state
        .world
        .location_by_id(state.character.location_id)
        .expect("starting location should exist")
        .clone();
    let content_location_id = state
        .campaign_content
        .as_ref()
        .and_then(|content| {
            content
                .world
                .locations
                .iter()
                .find(|candidate| candidate.name == location.name)
        })
        .expect("starting location should have authored content")
        .id
        .clone();
    let map = generate_local_map(
        731,
        format!("location.{}", location.id),
        LocalMapConfig::default(),
    );
    (state, location, map, content_location_id)
}

fn profile(
    id: &str,
    location_id: &str,
    location_name: &str,
    enemy_name: &str,
    weight: u32,
) -> EncounterContent {
    EncounterContent {
        id: Some(id.into()),
        location_id: Some(location_id.into()),
        quest_id: None,
        location_name: location_name.into(),
        enemy_name: enemy_name.into(),
        enemy_hp: 20,
        enemy_power: 3,
        trophy_item_name: format!("{enemy_name} trophy"),
        description: format!("{enemy_name} waits in the dark."),
        weight,
        chance_percent: Some(100),
        conditions: Some(EncounterSpawnConditions {
            dangerous: Some(true),
            ..Default::default()
        }),
    }
}

fn replace_location_profiles(
    state: &mut GameState,
    location_name: &str,
    profiles: Vec<EncounterContent>,
) {
    let content = state
        .campaign_content
        .as_mut()
        .expect("test campaign should have content");
    content
        .encounters
        .retain(|existing| existing.location_name != location_name);
    content.encounters.extend(profiles);
    if let Some(location) = state
        .world
        .location_by_id_mut(state.character.location_id)
    {
        location.dangerous = true;
    }
}

#[test]
fn authored_encounters_spawn_without_the_legacy_global_threat_flag() {
    let (mut state, location, mut map, content_location_id) = fixture();
    replace_location_profiles(
        &mut state,
        &location.name,
        vec![profile(
            "spawn.test.marauder",
            &content_location_id,
            &location.name,
            "Ash Marauder",
            4,
        )],
    );
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(!state.threat.active);
    let player_tile = map.entry;
    assert!(ensure_spatial_encounter_on_entry(
        &state,
        &location,
        &mut map,
        Some(player_tile),
    ));
    let encounter = &map.encounters[0];
    assert_eq!(encounter.enemy_name, "Ash Marauder");
    assert_eq!(encounter.max_hp, 20);
    assert_eq!(
        encounter.id,
        format!("location.{}.encounter.spawn.test.marauder", location.id)
    );
}

#[test]
fn profile_conditions_require_matching_world_time_quests_and_character_state() {
    let (mut state, location, mut map, content_location_id) = fixture();
    let mut conditional = profile(
        "spawn.test.conditional",
        &content_location_id,
        &location.name,
        "Hollow Stalker",
        1,
    );
    conditional.conditions = Some(EncounterSpawnConditions {
        dangerous: Some(true),
        night: Some(true),
        min_day: Some(4),
        max_day: Some(8),
        required_completed_quest_id: Some("quest.test.completed".into()),
        required_condition_name: Some("Wounded".into()),
    });
    replace_location_profiles(&mut state, &location.name, vec![conditional]);
    let location = state.world.location_by_id(location.id).unwrap().clone();

    state.world.day = 3;
    state.world.time_points = 11;
    state.world.completed_quest_ids.push("quest.test.completed".into());
    state.character.conditions.push(Condition::new("Wounded", 3, -1));
    assert!(!ensure_spatial_encounter_on_entry(&state, &location, &mut map, None));

    state.world.day = 4;
    assert!(ensure_spatial_encounter_on_entry(
        &state,
        &location,
        &mut map,
        None
    ));
    assert_eq!(map.encounters[0].enemy_name, "Hollow Stalker");
}

#[test]
fn weighted_selection_is_deterministic_and_materialized_records_are_never_rerolled() {
    let (mut state, location, map, content_location_id) = fixture();
    replace_location_profiles(
        &mut state,
        &location.name,
        vec![
            profile("spawn.test.a", &content_location_id, &location.name, "A", 1),
            profile("spawn.test.b", &content_location_id, &location.name, "B", 7),
        ],
    );
    let location = state.world.location_by_id(location.id).unwrap().clone();
    let mut first = map.clone();
    let mut second = map.clone();

    assert!(ensure_spatial_encounter_on_entry(&state, &location, &mut first, None));
    assert!(ensure_spatial_encounter_on_entry(&state, &location, &mut second, None));
    assert_eq!(first.encounters, second.encounters);
    assert!(["A", "B"].contains(&first.encounters[0].enemy_name.as_str()));

    let original = first.encounters[0].clone();
    first.encounters[0].current_hp = first.encounters[0].current_hp.saturating_sub(5);
    state.world.day += 10;
    state
        .campaign_content
        .as_mut()
        .unwrap()
        .encounters
        .iter_mut()
        .find(|profile| profile.id.as_deref() == Some("spawn.test.a"))
        .unwrap()
        .enemy_name = "Changed".into();
    assert!(!ensure_spatial_encounter_on_entry(&state, &location, &mut first, None));
    assert_eq!(first.encounters[0].id, original.id);
    assert_eq!(first.encounters[0].current_hp, original.current_hp - 5);
    assert_ne!(first.encounters[0].enemy_name, "Changed");
}


#[test]
fn legacy_location_only_profiles_keep_their_global_threat_gate() {
    let (mut state, location, mut map, content_location_id) = fixture();
    let mut legacy = profile(
        "ignored.test.id",
        &content_location_id,
        &location.name,
        "Legacy Marauder",
        1,
    );
    legacy.id = None;
    legacy.location_id = None;
    legacy.quest_id = None;
    legacy.conditions = None;
    replace_location_profiles(&mut state, &location.name, vec![legacy]);
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(!ensure_spatial_encounter_on_entry(
        &state,
        &location,
        &mut map,
        None
    ));
    state.threat.activate(
        location.id,
        "Legacy location threat",
        "The old encounter profile remains available.",
    );
    assert!(ensure_spatial_encounter_on_entry(
        &state,
        &location,
        &mut map,
        None
    ));
    assert_eq!(map.encounters[0].enemy_name, "Legacy Marauder");
}

#[test]
fn zero_chance_suppresses_a_profile_but_legacy_threats_keep_the_old_fallback() {
    let (mut state, location, mut map, content_location_id) = fixture();
    let mut unavailable = profile(
        "spawn.test.never",
        &content_location_id,
        &location.name,
        "Never Spawn",
        1,
    );
    unavailable.chance_percent = Some(0);
    replace_location_profiles(&mut state, &location.name, vec![unavailable]);
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(!ensure_spatial_encounter_on_entry(&state, &location, &mut map, None));
    assert!(map.encounters.is_empty());

    state.campaign_content.as_mut().unwrap().encounters.clear();
    state.threat.activate(
        location.id,
        "Legacy threat",
        "The old threat system is active here.",
    );
    assert!(ensure_spatial_encounter_on_entry(&state, &location, &mut map, None));
    assert!(!map.encounters[0].enemy_name.is_empty());
}
