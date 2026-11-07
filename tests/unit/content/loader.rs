use super::*;

fn valid_event(id: &str) -> EventContent {
    EventContent {
        id: id.into(),
        trigger: "travel_arrival".into(),
        weight: 1,
        chance_percent: Some(100),
        cooldown_turns: None,
        conditions: None,
        effects: vec![EventEffectContent::Pause],
    }
}

fn empty_mod_content() -> CampaignContent {
    let mut content = default_campaign_content();
    content.world.locations.clear();
    content.factions.clear();
    content.npcs.clear();
    content.character_parts.clear();
    content.quests.clear();
    content.encounters.clear();
    content.atmospheres.clear();
    content.item_visuals.clear();
    content.events.clear();
    content
}

#[test]
fn mods_extend_and_override_character_parts_by_stable_id() {
    let mut base = default_campaign_content();
    base.character_parts.clear();
    base.character_parts.push(CharacterPartContent {
        id: "portrait.face.base".into(),
        slot: "face".into(),
        asset: "characters/parts/face/base.png".into(),
        weight: 2,
        layer: 10,
        requires_tags: Vec::new(),
        excludes_tags: Vec::new(),
    });
    let mut incoming = empty_mod_content();
    incoming.character_parts.push(CharacterPartContent {
        id: "portrait.face.base".into(),
        slot: "face".into(),
        asset: "characters/parts/face/modded.png".into(),
        weight: 5,
        layer: 10,
        requires_tags: vec!["human".into()],
        excludes_tags: Vec::new(),
    });
    incoming.character_parts.push(CharacterPartContent {
        id: "portrait.hair.long".into(),
        slot: "hair".into(),
        asset: "characters/parts/hair/long.png".into(),
        weight: 1,
        layer: 20,
        requires_tags: Vec::new(),
        excludes_tags: Vec::new(),
    });
    merge_campaign_content(&mut base, incoming);

    assert_eq!(base.character_parts.len(), 2);
    let face = base
        .character_parts
        .iter()
        .find(|part| part.id == "portrait.face.base")
        .unwrap();
    assert_eq!(face.asset, "characters/parts/face/modded.png");
    assert_eq!(face.weight, 5);
    assert_eq!(face.requires_tags, vec!["human"]);
}

#[test]
fn invalid_events_are_rejected_and_reported() {
    let mut base = default_campaign_content();
    let mut invalid = valid_event("bad.event");
    invalid.weight = 0;
    let mut warnings = Vec::new();

    merge_mod_events(
        &mut base,
        vec![(1, "invalid.mod".into(), vec![invalid])],
        &mut warnings,
    );

    assert!(!base.events.iter().any(|event| event.id == "bad.event"));
    assert!(warnings
        .iter()
        .any(|warning| warning.contains("zero weight")));
}

#[test]
fn mod_events_can_reference_definitions_from_other_mods() {
    let mut base = default_campaign_content();
    let mut event_mod = empty_mod_content();
    let mut event = valid_event("events.cross_mod_reference");
    event.conditions = Some(EventConditionContent {
        locations: vec!["Later Reach".into()],
        faction_name: Some("Later Ward".into()),
        prior_event_id: Some("events.later_dependency".into()),
        ..Default::default()
    });
    event_mod.events.push(event);
    let pending = vec![
        (
            10,
            "events.mod".to_string(),
            std::mem::take(&mut event_mod.events),
        ),
        (
            1,
            "definitions.mod".to_string(),
            vec![valid_event("events.later_dependency")],
        ),
    ];

    // The event is supplied before the location and faction it references.
    merge_campaign_content(&mut base, event_mod);

    let mut definitions_mod = empty_mod_content();
    definitions_mod.world.locations.push(LocationContent {
        id: "location.later-reach".into(),
        name: "Later Reach".into(),
        description: "A test location supplied by another mod.".into(),
        dangerous: false,
        exits: Vec::new(),
        scene_art: None,
        scene_image: None,
        spatial_layout: None,
    });
    definitions_mod.factions.push(FactionContent {
        id: "faction.later-ward".into(),
        name: "Later Ward".into(),
    });
    merge_campaign_content(&mut base, definitions_mod);

    let mut warnings = Vec::new();
    merge_mod_events(&mut base, pending, &mut warnings);

    assert!(base
        .events
        .iter()
        .any(|candidate| candidate.id == "events.cross_mod_reference"));
    assert!(base
        .events
        .iter()
        .any(|candidate| candidate.id == "events.later_dependency"));
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
}

#[test]
fn default_campaign_includes_reusable_character_parts_and_assets() {
    let content = default_campaign_content();

    for slot in ["body", "face", "hair", "headgear", "accessory"] {
        assert!(
            content.character_parts.iter().any(|part| part.slot == slot),
            "missing bundled character-part slot: {slot}",
        );
    }

    for part in &content.character_parts {
        let path = std::path::Path::new("data/assets").join(&part.asset);
        assert!(
            path.is_file(),
            "bundled character-part asset does not exist: {}",
            path.display(),
        );
    }
}
