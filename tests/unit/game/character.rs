use super::build_character_sheet_view;
use crate::model::{create_new_state, Condition, Faction, WorldMode};

#[test]
fn character_sheet_view_resolves_only_the_current_characters_known_evidence() {
    let mut state = create_new_state(
        "Evidence Journal",
        WorldMode::New,
        "Ash".to_string(),
        "Wanderer".to_string(),
    );
    let evidence = state
        .campaign_content
        .as_ref()
        .and_then(|content| {
            content
                .world
                .locations
                .iter()
                .flat_map(|location| location.spatial_discoveries.iter())
                .next()
        })
        .expect("base campaign should contain authored spatial evidence")
        .clone();
    state.character.record_evidence(evidence.id.clone());
    state.character.notes.push("A note in my own words.".into());

    let view = build_character_sheet_view(&state);

    assert_eq!(view.evidence.len(), 1);
    assert_eq!(view.evidence[0].id, evidence.id);
    assert_eq!(view.evidence[0].title, evidence.title);
    assert_eq!(view.evidence[0].description, evidence.description);
    assert_eq!(view.notes, state.character.notes);
    assert!(view
        .notes
        .iter()
        .any(|note| note == "A note in my own words."));
}

#[test]
fn character_sheet_view_preserves_derived_character_data() {
    let mut state = create_new_state(
        "Test World",
        WorldMode::New,
        "Ash".to_string(),
        "Wanderer".to_string(),
    );
    state.character.level = 3;
    state.character.experience = 27;
    state.character.hp = 8;
    state.character.attributes.might = 4;
    state.character.conditions = vec![Condition {
        name: "Wounded".to_string(),
        remaining: 2,
        penalty: -1,
        bonus: 0,
    }];
    let mut faction = Faction::new(20, "Wardens");
    faction.reputation = 5;
    faction.memory = vec![
        "A debt remembered.".to_string(),
        "[authored anchor] Wardens are anchored in the northern reach.".to_string(),
        "[generated relationship] alliance with the Ashen Court.".to_string(),
        "A generated faction shaped by a harsh mountain region.".to_string(),
    ];
    state.factions.push(faction);

    let view = build_character_sheet_view(&state);

    assert_eq!(view.character.display_name(), "Ash the Wanderer");
    assert_eq!(view.level, 3);
    assert_eq!(view.experience, 27);
    assert_eq!(view.next_level_experience, 150);
    assert_eq!(view.attributes.might, 4);
    assert_eq!(view.attributes.effective_might, 3);
    assert_eq!(view.conditions[0].remaining, 2);
    assert_eq!(view.factions[0].reputation, 5);
    assert_eq!(view.factions[0].memories, vec!["A debt remembered."]);
    assert_eq!(state.factions[0].memory.len(), 4);
}
