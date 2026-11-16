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
