use super::*;
use crate::model::{create_new_state, Quest, WorldMode};

#[test]
fn campaign_quest_templates_add_evidence_goals_without_replacing_existing_objectives() {
    let mut state = create_new_state(
        "Authored Evidence Quest",
        WorldMode::New,
        "Tester".into(),
        "Ash Walker".into(),
    );
    let mut content = state
        .campaign_content
        .clone()
        .expect("new game should retain campaign content");
    let quest_content = content
        .quests
        .iter_mut()
        .find(|quest| quest.id == "quest.quiet_old_shrine")
        .expect("base campaign should contain the Old Shrine quest");
    assert_eq!(
        quest_content.required_evidence_ids,
        vec!["poi.old_shrine.black_altar"]
    );
    let quest_id = quest_content.id.clone();
    state.campaign_content = Some(content);

    // Re-seed only the authored quest templates. Existing campaign factions
    // and NPCs are retained; the empty quest list allows the template to seed.
    state.quests.clear();
    crate::game::world::bootstrap_campaign_content(&mut state);

    let quest = state
        .quests
        .iter()
        .find(|quest| quest.content_id == quest_id)
        .expect("authored quest should be seeded");
    assert!(quest
        .objectives
        .iter()
        .any(|objective| { objective.kind == QuestObjectiveKind::VisitLocation }));
    let evidence = quest
        .objectives
        .iter()
        .find(|objective| objective.kind == QuestObjectiveKind::DiscoverEvidence)
        .expect("authored evidence should become a quest objective");
    assert_eq!(evidence.target, "poi.old_shrine.black_altar");
    assert_eq!(evidence.display_label(), "Discover Black Altar");
}

#[test]
fn evidence_objectives_follow_the_current_characters_known_evidence() {
    let mut state = create_new_state(
        "Evidence Quest",
        WorldMode::New,
        "Tester".into(),
        "Ash Walker".into(),
    );
    state.quests.push(crate::model::Quest {
        id: 909,
        title: "Understand the marks".into(),
        offered: true,
        objectives: vec![crate::model::QuestObjective::discover_evidence(
            "poi.old_shrine.black_altar",
            "Black Altar",
        )],
        ..Default::default()
    });

    sync_objectives(&mut state, 0);
    assert!(!state.quests[0].objectives[0].completed);
    assert_eq!(
        state.quests[0].objectives[0].display_label(),
        "Discover Black Altar"
    );

    state
        .character
        .record_evidence("poi.old_shrine.black_altar");
    sync_objectives(&mut state, 0);
    assert!(state.quests[0].objectives[0].completed);
    assert_eq!(state.quests[0].objectives[0].progress, 1);
}


#[test]
fn alternate_encounter_profiles_complete_their_shared_authored_quest() {
    let mut state = create_new_state(
        "Alternate Encounter Quest",
        WorldMode::New,
        "Tester".into(),
        "Ash Walker".into(),
    );
    let quest_index = state
        .quests
        .iter()
        .position(|quest| quest.content_id == "quest.quiet_old_shrine")
        .expect("base campaign should seed the Old Shrine quest");
    let location_id = state.quests[quest_index].target_location_id;
    state.quests[quest_index].objectives.clear();
    state.quests[quest_index].offered = true;
    let location_name = state
        .world
        .location_by_id(location_id)
        .expect("quest target location should exist")
        .name
        .clone();
    let alternate = {
        let content = state.campaign_content.as_ref().unwrap();
        let original = content
            .encounters
            .iter()
            .find(|profile| profile.quest_id.as_deref() == Some("quest.quiet_old_shrine"))
            .expect("base quest should have an encounter profile");
        let mut alternate = original.clone();
        alternate.id = Some("encounter.old_shrine.alternate_test".into());
        alternate.enemy_name = "Shrine Echo".into();
        alternate.trophy_item_name = "Echo Shard".into();
        alternate
    };
    state
        .campaign_content
        .as_mut()
        .unwrap()
        .encounters
        .push(alternate);

    assert!(normalize_quest(&mut state, quest_index));
    let objective = state.quests[quest_index]
        .objectives
        .iter()
        .find(|objective| objective.kind == QuestObjectiveKind::DefeatEnemy)
        .expect("quest should contain a defeat objective");
    assert_eq!(objective.target, "quest.quiet_old_shrine");
    assert_eq!(
        objective.display_label(),
        format!("Defeat the threat at {location_name}")
    );

    record_enemy_defeat(&mut state, "Shrine Echo", location_id);
    let objective = state.quests[quest_index]
        .objectives
        .iter()
        .find(|objective| objective.kind == QuestObjectiveKind::DefeatEnemy)
        .unwrap();
    assert!(objective.completed);
    assert_eq!(objective.progress, 1);
}


#[test]
fn snapshotted_encounter_quest_id_survives_campaign_profile_changes() {
    let mut state = create_new_state(
        "Persistent Encounter Quest",
        WorldMode::New,
        "Tester".into(),
        "Ash Walker".into(),
    );
    let quest_index = state
        .quests
        .iter()
        .position(|quest| quest.content_id == "quest.quiet_old_shrine")
        .expect("base campaign should seed the Old Shrine quest");
    let location_id = state.quests[quest_index].target_location_id;
    state.quests[quest_index].objectives.clear();
    state.quests[quest_index].offered = true;
    assert!(normalize_quest(&mut state, quest_index));
    assert!(state.quests[quest_index]
        .objectives
        .iter()
        .any(|objective| objective.target == "quest.quiet_old_shrine"));

    let profile = state
        .campaign_content
        .as_mut()
        .unwrap()
        .encounters
        .iter_mut()
        .find(|profile| profile.quest_id.as_deref() == Some("quest.quiet_old_shrine"))
        .expect("base content should link the encounter to its quest");
    profile.quest_id = None;
    profile.enemy_name = "A changed campaign enemy".into();

    record_spatial_encounter_defeat(
        &mut state,
        "Ashen Wretch",
        location_id,
        Some("quest.quiet_old_shrine"),
    );
    let objective = state.quests[quest_index]
        .objectives
        .iter()
        .find(|objective| objective.kind == QuestObjectiveKind::DefeatEnemy)
        .expect("snapshotted quest identity should retain its defeat objective");
    assert!(objective.completed);
    assert_eq!(objective.progress, 1);
}

#[test]
fn legacy_quest_is_expanded_into_explicit_objectives() {
    let mut state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".into(),
        "Ash Walker".into(),
    );
    let location_id = state.world.locations[0].id;
    let faction_id = 100;
    state
        .factions
        .push(crate::model::Faction::new(faction_id, "Test Faction"));
    state.quests.push(Quest::new(
        101,
        "quest.test",
        "Test Quest",
        "Test",
        location_id,
        faction_id,
        0,
        "Test Trophy",
        "Test Reward",
    ));
    assert!(normalize_quest(&mut state, 0));
    assert!(state.quests[0]
        .objectives
        .iter()
        .any(|objective| objective.kind == QuestObjectiveKind::AcquireItem));
    assert!(state.quests[0]
        .objectives
        .iter()
        .any(|objective| objective.kind == QuestObjectiveKind::VisitLocation));
}

#[test]
fn acquired_items_and_location_visits_update_progress() {
    let mut state = create_new_state(
        "Test World",
        WorldMode::New,
        "Tester".into(),
        "Ash Walker".into(),
    );
    let location_id = state.world.locations[0].id;
    let faction_id = 100;
    state
        .factions
        .push(crate::model::Faction::new(faction_id, "Test Faction"));
    state.quests.push(Quest::new(
        101,
        "quest.test",
        "Test Quest",
        "Test",
        location_id,
        faction_id,
        0,
        "Test Trophy",
        "Test Reward",
    ));
    state.quests[0].offered = true;
    normalize_quest(&mut state, 0);
    state.character.inventory.push(Item {
        id: 102,
        name: "Test Trophy".into(),
        description: "Proof".into(),
    });
    sync_objectives(&mut state, 0);
    assert!(state.quests[0]
        .objectives
        .iter()
        .filter(|objective| objective.kind == QuestObjectiveKind::AcquireItem)
        .all(|objective| objective.completed));
    assert!(state.quests[0]
        .objectives
        .iter()
        .filter(|objective| objective.kind == QuestObjectiveKind::VisitLocation)
        .all(|objective| objective.completed));
}
