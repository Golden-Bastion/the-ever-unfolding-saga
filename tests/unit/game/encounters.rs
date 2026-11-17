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
        spawn_group: None,
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
    if let Some(location) = state.world.location_by_id_mut(state.character.location_id) {
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
fn materialized_encounters_retain_the_authored_quest_identity() {
    let (mut state, location, mut map, content_location_id) = fixture();
    let mut authored = profile(
        "spawn.test.quest-linked",
        &content_location_id,
        &location.name,
        "Quest Warden",
        1,
    );
    authored.quest_id = Some("quest.test.persistent_link".into());
    replace_location_profiles(&mut state, &location.name, vec![authored]);
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert_eq!(
        map.encounters[0].quest_id.as_deref(),
        Some("quest.test.persistent_link")
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
    state
        .world
        .completed_quest_ids
        .push("quest.test.completed".into());
    state
        .character
        .conditions
        .push(Condition::new("Wounded", 3, -1));
    assert!(!ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));

    state.world.day = 4;
    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
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

    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut first, None
    ));
    assert!(ensure_spatial_encounter_on_entry(
        &state,
        &location,
        &mut second,
        None
    ));
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
    assert!(!ensure_spatial_encounter_on_entry(
        &state, &location, &mut first, None
    ));
    assert_eq!(first.encounters[0].id, original.id);
    assert_eq!(first.encounters[0].current_hp, original.current_hp - 5);
    assert_ne!(first.encounters[0].enemy_name, "Changed");
}

#[test]
fn independent_spawn_groups_materialize_separately_but_keep_weighted_alternatives() {
    let (mut state, location, mut map, content_location_id) = fixture();
    let pack_a = profile(
        "spawn.test.pack.a",
        &content_location_id,
        &location.name,
        "Ash Hound",
        1,
    );
    let pack_b = profile(
        "spawn.test.pack.b",
        &content_location_id,
        &location.name,
        "Ash Brute",
        4,
    );
    let mut scout_a = profile(
        "spawn.test.scout.a",
        &content_location_id,
        &location.name,
        "Cinder Scout",
        1,
    );
    let mut scout_b = profile(
        "spawn.test.scout.b",
        &content_location_id,
        &location.name,
        "Cinder Lookout",
        2,
    );
    let mut sentinel = profile(
        "spawn.test.sentinel",
        &content_location_id,
        &location.name,
        "Gate Sentinel",
        1,
    );
    scout_a.spawn_group = Some("scouts".into());
    scout_b.spawn_group = Some("scouts".into());
    sentinel.spawn_group = Some("sentinel".into());
    replace_location_profiles(
        &mut state,
        &location.name,
        vec![pack_a, pack_b, scout_a, scout_b, sentinel],
    );
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert_eq!(map.encounters.len(), 3);
    assert_eq!(
        map.encounters
            .iter()
            .filter(|encounter| encounter.spawn_group.is_none())
            .count(),
        1,
        "profiles without a group remain weighted alternatives in the default group"
    );
    assert_eq!(
        map.encounters
            .iter()
            .filter(|encounter| encounter.spawn_group.as_deref() == Some("scouts"))
            .count(),
        1,
        "a named group selects only one of its alternatives"
    );
    assert_eq!(
        map.encounters
            .iter()
            .filter(|encounter| encounter.spawn_group.as_deref() == Some("sentinel"))
            .count(),
        1
    );
    assert!(map
        .encounters
        .iter()
        .all(|encounter| encounter.status == crate::spatial::SpatialEncounterStatus::Active));
    let positions = map
        .encounters
        .iter()
        .map(|encounter| encounter.position)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(positions.len(), map.encounters.len());
    assert!(map.validate_encounters().is_ok());

    let snapshot = map.encounters.clone();
    state.world.day += 5;
    assert!(!ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert_eq!(map.encounters, snapshot);
}

#[test]
fn a_newly_eligible_spawn_group_can_materialize_without_resetting_existing_records() {
    let (mut state, location, mut map, content_location_id) = fixture();
    state.world.day = 1;
    let initial = profile(
        "spawn.test.initial",
        &content_location_id,
        &location.name,
        "Shrine Guard",
        1,
    );
    let mut later = profile(
        "spawn.test.later",
        &content_location_id,
        &location.name,
        "Returning Stalker",
        1,
    );
    later.spawn_group = Some("later".into());
    later.conditions = Some(EncounterSpawnConditions {
        dangerous: Some(true),
        min_day: Some(2),
        ..Default::default()
    });
    replace_location_profiles(&mut state, &location.name, vec![initial, later]);
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert_eq!(map.encounters.len(), 1);
    let initial_record = map.encounters[0].clone();
    map.encounters[0].current_hp -= 3;

    state.world.day = 2;
    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert_eq!(map.encounters.len(), 2);
    let restored_initial = map
        .encounters
        .iter()
        .find(|encounter| encounter.id == initial_record.id)
        .unwrap();
    assert_eq!(restored_initial.current_hp, initial_record.current_hp - 3);
    assert_eq!(restored_initial.enemy_name, initial_record.enemy_name);
    assert!(map
        .encounters
        .iter()
        .any(|encounter| encounter.enemy_name == "Returning Stalker"));
}

#[test]
fn legacy_encounter_records_reserve_the_default_group_but_allow_named_groups() {
    let (mut state, location, mut map, content_location_id) = fixture();
    assert!(crate::spatial::ensure_spatial_encounter(
        &mut map,
        "encounter.legacy.snapshot".into(),
        "Old Wretch".into(),
        "A saved encounter from an older schema.".into(),
        8,
        2,
        "Old Token".into(),
    ));
    map.encounters[0].current_hp = 0;
    map.encounters[0].status = crate::spatial::SpatialEncounterStatus::Defeated;
    let default_profile = profile(
        "spawn.test.default",
        &content_location_id,
        &location.name,
        "Replacement Wretch",
        1,
    );
    let mut named_profile = profile(
        "spawn.test.named",
        &content_location_id,
        &location.name,
        "Cinder Scout",
        1,
    );
    named_profile.spawn_group = Some("scouts".into());
    replace_location_profiles(
        &mut state,
        &location.name,
        vec![default_profile, named_profile],
    );
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert_eq!(map.encounters.len(), 2);
    assert!(map
        .encounters
        .iter()
        .any(|encounter| encounter.enemy_name == "Old Wretch"));
    assert!(!map
        .encounters
        .iter()
        .any(|encounter| encounter.enemy_name == "Replacement Wretch"));
    assert!(map
        .encounters
        .iter()
        .any(|encounter| encounter.enemy_name == "Cinder Scout"));
}

#[test]
fn idless_profiles_in_different_groups_receive_distinct_persistent_ids() {
    let (mut state, location, mut map, _content_location_id) = fixture();
    let mut northern = profile(
        "unused.north",
        "unused",
        &location.name,
        "Nameless Threat",
        1,
    );
    northern.id = None;
    northern.location_id = None;
    northern.spawn_group = Some("north".into());
    let mut southern = profile(
        "unused.south",
        "unused",
        &location.name,
        "Nameless Threat",
        1,
    );
    southern.id = None;
    southern.location_id = None;
    southern.spawn_group = Some("south".into());
    replace_location_profiles(&mut state, &location.name, vec![northern, southern]);
    let location = state.world.location_by_id(location.id).unwrap().clone();

    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert_eq!(map.encounters.len(), 2);
    assert_eq!(map.encounters[0].enemy_name, map.encounters[1].enemy_name);
    assert_ne!(map.encounters[0].id, map.encounters[1].id);
    assert_eq!(
        map.encounters
            .iter()
            .filter(|encounter| encounter.spawn_group.as_deref() == Some("north"))
            .count(),
        1
    );
    assert_eq!(
        map.encounters
            .iter()
            .filter(|encounter| encounter.spawn_group.as_deref() == Some("south"))
            .count(),
        1
    );
    assert!(map.validate_encounters().is_ok());
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
        &state, &location, &mut map, None
    ));
    state.threat.activate(
        location.id,
        "Legacy location threat",
        "The old encounter profile remains available.",
    );
    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
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

    assert!(!ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert!(map.encounters.is_empty());

    state.campaign_content.as_mut().unwrap().encounters.clear();
    state.threat.activate(
        location.id,
        "Legacy threat",
        "The old threat system is active here.",
    );
    assert!(ensure_spatial_encounter_on_entry(
        &state, &location, &mut map, None
    ));
    assert!(!map.encounters[0].enemy_name.is_empty());
}
