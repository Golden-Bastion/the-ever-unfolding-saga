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
fn legacy_location_content_defaults_authored_exit_sides_to_empty() {
    let legacy = serde_json::json!({
        "id": "location.legacy",
        "name": "Legacy",
        "description": "Older content.",
        "dangerous": false,
        "exits": ["location.other"]
    });
    let location: LocationContent =
        serde_json::from_value(legacy).expect("legacy content should deserialize");
    assert!(location.exit_sides.is_empty());
    assert!(location.exit_placements.is_empty());
}

#[test]
fn authored_exit_side_must_reference_a_declared_exit() {
    let mut content = default_campaign_content();
    let location = content
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == "location.ashen_gate")
        .unwrap();
    location.exit_sides.insert(
        "location.not_an_exit".into(),
        crate::spatial::MapExitSide::East,
    );
    assert!(content.validate().iter().any(|issue| {
        issue.contains("location.not_an_exit") && issue.contains("does not declare that exit")
    }));
}

#[test]
fn authored_exit_placement_must_reference_a_declared_exit() {
    let mut content = default_campaign_content();
    let location = content
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == "location.ashen_gate")
        .unwrap();
    location.exit_placements.insert(
        "location.not_an_exit".into(),
        AuthoredExitPlacementContent {
            side: crate::spatial::MapExitSide::East,
            offset_q16: 32_768,
        },
    );
    assert!(content.validate().iter().any(|issue| {
        issue.contains("location.not_an_exit") && issue.contains("does not declare that exit")
    }));
}

#[test]
fn authored_exit_placement_overrides_a_declared_legacy_side() {
    let mut content = default_campaign_content();
    let location = content
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == "location.ashen_gate")
        .unwrap();
    location.exit_sides.insert(
        "location.hollow_market".into(),
        crate::spatial::MapExitSide::West,
    );
    location.exit_placements.insert(
        "location.hollow_market".into(),
        AuthoredExitPlacementContent {
            side: crate::spatial::MapExitSide::East,
            offset_q16: 49_152,
        },
    );
    assert_eq!(
        location.exit_preference("location.hollow_market"),
        Some((crate::spatial::MapExitSide::East, Some(49_152)))
    );
    assert!(!content.validate().iter().any(|issue| {
        issue.contains("location.ashen_gate") && issue.contains("location.hollow_market")
    }));
}

#[test]
fn authored_exit_side_remains_a_legacy_fallback() {
    let mut content = default_campaign_content();
    let location = content
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == "location.ashen_gate")
        .unwrap();
    location.exit_placements.clear();
    location.exit_sides.insert(
        "location.hollow_market".into(),
        crate::spatial::MapExitSide::East,
    );

    assert_eq!(
        location.exit_preference("location.hollow_market"),
        Some((crate::spatial::MapExitSide::East, None))
    );
    assert!(!content.validate().iter().any(|issue| {
        issue.contains("location.ashen_gate") && issue.contains("location.hollow_market")
    }));
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
fn legacy_encounter_definitions_receive_safe_default_spawn_metadata() {
    let legacy = serde_json::json!({
        "location_name": "Old Shrine",
        "enemy_name": "Old Marauder",
        "enemy_hp": 12,
        "enemy_power": 3,
        "trophy_item_name": "Old Token",
        "description": "A threat from an older campaign definition."
    });
    let profile: EncounterContent =
        serde_json::from_value(legacy).expect("legacy profiles should still deserialize");
    assert!(profile.id.is_none());
    assert!(profile.location_id.is_none());
    assert!(profile.quest_id.is_none());
    assert!(profile.spawn_group.is_none());
    assert_eq!(profile.weight, 1);
    assert!(profile.chance_percent.is_none());
    assert!(profile.conditions.is_none());
}

#[test]
fn encounter_profiles_merge_by_stable_id_and_legacy_mods_preserve_spawn_metadata() {
    let mut base = default_campaign_content();
    let original = {
        let profile = base
            .encounters
            .iter_mut()
            .find(|profile| profile.location_id.as_deref() == Some("location.old_shrine"))
            .expect("base content should define the Old Shrine encounter");
        profile.spawn_group = Some("shrine_watch".into());
        profile.clone()
    };

    let mut legacy_mod = empty_mod_content();
    legacy_mod.encounters.push(EncounterContent {
        id: None,
        location_id: None,
        quest_id: None,
        spawn_group: None,
        location_name: original.location_name.clone(),
        enemy_name: "Modded Wretch".into(),
        enemy_hp: 14,
        enemy_power: 4,
        trophy_item_name: original.trophy_item_name.clone(),
        description: "A modded description.".into(),
        weight: 1,
        chance_percent: None,
        conditions: None,
    });
    merge_campaign_content(&mut base, legacy_mod);

    let overridden = base
        .encounters
        .iter()
        .find(|profile| profile.id == original.id)
        .expect("legacy override should preserve the original stable profile");
    assert_eq!(overridden.enemy_name, "Modded Wretch");
    assert_eq!(overridden.location_id, original.location_id);
    assert_eq!(overridden.quest_id, original.quest_id);
    assert_eq!(overridden.spawn_group, original.spawn_group);
    assert_eq!(overridden.weight, original.weight);
    assert_eq!(
        overridden
            .conditions
            .as_ref()
            .and_then(|value| value.dangerous),
        Some(true)
    );

    let mut additional_mod = empty_mod_content();
    additional_mod.encounters.push(EncounterContent {
        id: Some("encounter.old_shrine.alternate".into()),
        location_id: original.location_id.clone(),
        quest_id: original.quest_id.clone(),
        spawn_group: Some("night_watch".into()),
        location_name: original.location_name.clone(),
        enemy_name: "Shrine Echo".into(),
        enemy_hp: 9,
        enemy_power: 2,
        trophy_item_name: "Echo Shard".into(),
        description: "An echo rises beneath the altar.".into(),
        weight: 2,
        chance_percent: Some(75),
        conditions: Some(EncounterSpawnConditions {
            night: Some(true),
            ..Default::default()
        }),
    });
    merge_campaign_content(&mut base, additional_mod);
    let alternate = base
        .encounters
        .iter()
        .find(|profile| profile.id.as_deref() == Some("encounter.old_shrine.alternate"))
        .expect("mod profile should be appended");
    assert_eq!(alternate.spawn_group.as_deref(), Some("night_watch"));
    assert!(
        base.validate().is_empty(),
        "invalid content: {:?}",
        base.validate()
    );
}

#[test]
fn invalid_encounter_spawn_metadata_is_rejected() {
    let mut content = default_campaign_content();
    let profile = content
        .encounters
        .first_mut()
        .expect("base content should contain encounter profiles");
    profile.weight = 0;
    profile.chance_percent = Some(101);
    profile.spawn_group = Some("  ".into());
    profile.location_id = Some("location.missing".into());
    profile.quest_id = Some("quest.missing".into());
    profile.conditions = Some(EncounterSpawnConditions {
        min_day: Some(9),
        max_day: Some(3),
        required_completed_quest_id: Some("quest.unknown".into()),
        ..Default::default()
    });

    let issues = content.validate();
    assert!(issues.iter().any(|issue| issue.contains("zero weight")));
    assert!(issues.iter().any(|issue| issue.contains("invalid chance")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("empty spawn group")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("unknown location ID")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("unknown quest ID")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("min_day greater than max_day")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("unknown completed quest")));
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
        archetype: None,
        exits: Vec::new(),
        scene_art: None,
        scene_image: None,
        spatial_layout: None,
        geography: None,
        spatial_discoveries: Vec::new(),
        exit_sides: Default::default(),
        exit_placements: Default::default(),
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

#[test]
fn mods_extend_and_override_spatial_discoveries_by_stable_id() {
    let mut base = default_campaign_content();
    let mut old_shrine = base
        .world
        .locations
        .iter()
        .find(|location| location.id == "location.old_shrine")
        .expect("base Old Shrine should exist")
        .clone();
    old_shrine.spatial_discoveries = vec![
        SpatialDiscoveryContent {
            id: "poi.old_shrine.altar".into(),
            title: "Old Altar".into(),
            description: "Base description.".into(),
            requires_evidence: Vec::new(),
        },
        SpatialDiscoveryContent {
            id: "poi.old_shrine.footprints".into(),
            title: "Footprints".into(),
            description: "Footprints cross the dust.".into(),
            requires_evidence: vec!["poi.old_shrine.altar".into()],
        },
    ];
    base.world
        .locations
        .retain(|location| location.id != old_shrine.id);
    base.world.locations.push(old_shrine.clone());

    let mut incoming = empty_mod_content();
    let mut override_location = old_shrine;
    override_location.spatial_discoveries = vec![
        SpatialDiscoveryContent {
            id: "poi.old_shrine.altar".into(),
            title: "Black Altar".into(),
            description: "A modded description.".into(),
            requires_evidence: vec!["poi.old_shrine.scratches".into()],
        },
        SpatialDiscoveryContent {
            id: "poi.old_shrine.scratches".into(),
            title: "Scratches".into(),
            description: "Marks appear in the stone.".into(),
            requires_evidence: Vec::new(),
        },
    ];
    incoming.world.locations.push(override_location);

    merge_campaign_content(&mut base, incoming);

    let discoveries = &base
        .world
        .locations
        .iter()
        .find(|location| location.id == "location.old_shrine")
        .unwrap()
        .spatial_discoveries;
    assert_eq!(discoveries.len(), 3);
    assert_eq!(
        discoveries
            .iter()
            .find(|discovery| discovery.id == "poi.old_shrine.altar")
            .unwrap()
            .description,
        "A modded description."
    );
    assert_eq!(
        discoveries
            .iter()
            .find(|discovery| discovery.id == "poi.old_shrine.altar")
            .unwrap()
            .requires_evidence,
        vec!["poi.old_shrine.scratches"]
    );
    assert!(discoveries
        .iter()
        .any(|discovery| discovery.id == "poi.old_shrine.footprints"));
    assert!(discoveries
        .iter()
        .any(|discovery| discovery.id == "poi.old_shrine.scratches"));
}

#[test]
fn base_campaign_links_gate_market_and_shrine_clues_into_one_investigation() {
    let content = default_campaign_content();
    let chain: [(&str, &str, &[&str]); 6] = [
        ("location.ashen_gate", "poi.ashen_gate.cracked_gate", &[]),
        (
            "location.ashen_gate",
            "poi.ashen_gate.dead_road",
            &["poi.ashen_gate.cracked_gate"],
        ),
        (
            "location.hollow_market",
            "poi.hollow_market.abandoned_stall",
            &["poi.ashen_gate.dead_road"],
        ),
        (
            "location.hollow_market",
            "poi.hollow_market.unlit_lantern",
            &["poi.hollow_market.abandoned_stall"],
        ),
        (
            "location.old_shrine",
            "poi.old_shrine.fresh_soot",
            &["poi.hollow_market.unlit_lantern"],
        ),
        (
            "location.old_shrine",
            "poi.old_shrine.black_altar",
            &["poi.old_shrine.fresh_soot"],
        ),
    ];

    for (location_id, evidence_id, prerequisites) in chain {
        let discovery = content
            .world
            .locations
            .iter()
            .find(|location| location.id == location_id)
            .and_then(|location| {
                location
                    .spatial_discoveries
                    .iter()
                    .find(|discovery| discovery.id == evidence_id)
            })
            .unwrap_or_else(|| panic!("missing authored discovery {evidence_id}"));
        let expected = prerequisites
            .iter()
            .map(|id| (*id).to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            discovery.requires_evidence, expected,
            "unexpected prerequisites for {evidence_id}"
        );
    }

    let shrine_quest = content
        .quests
        .iter()
        .find(|quest| quest.id == "quest.quiet_old_shrine")
        .expect("Quiet the Old Shrine quest should exist");
    assert_eq!(
        shrine_quest.required_evidence_ids,
        vec!["poi.old_shrine.black_altar"]
    );

    let issues = content.validate();
    assert!(
        issues.is_empty(),
        "the authored evidence chain should pass campaign validation: {issues:#?}"
    );
}

#[test]
fn invalid_spatial_discoveries_are_reported_by_content_validation() {
    let mut content = default_campaign_content();
    let location = content
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == "location.old_shrine")
        .unwrap();
    location.spatial_discoveries = vec![
        SpatialDiscoveryContent {
            id: String::new(),
            title: "Missing ID".into(),
            description: "Invalid discovery.".into(),
            requires_evidence: Vec::new(),
        },
        SpatialDiscoveryContent {
            id: "poi.old_shrine.duplicate".into(),
            title: String::new(),
            description: "Invalid discovery.".into(),
            requires_evidence: Vec::new(),
        },
        SpatialDiscoveryContent {
            id: "poi.old_shrine.duplicate".into(),
            title: "Duplicate".into(),
            description: String::new(),
            requires_evidence: Vec::new(),
        },
    ];

    let issues = content.validate();
    assert!(issues.iter().any(|issue| issue.contains("empty id")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("duplicate spatial discovery id")));
    assert!(issues.iter().any(|issue| issue.contains("empty title")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("empty description")));
}

#[test]
fn quest_evidence_objectives_must_reference_known_unique_discoveries() {
    let mut content = default_campaign_content();
    let quest = content
        .quests
        .first_mut()
        .expect("base campaign should contain a quest template");
    quest.required_evidence_ids = vec![
        "poi.old_shrine.black_altar".into(),
        "missing.evidence".into(),
        "poi.old_shrine.black_altar".into(),
        String::new(),
    ];

    let issues = content.validate();
    assert!(issues
        .iter()
        .any(|issue| issue.contains("unknown spatial evidence missing.evidence")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("repeats required evidence poi.old_shrine.black_altar")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("empty required evidence ID")));
}

#[test]
fn spatial_evidence_requirements_must_exist_and_form_an_acyclic_graph() {
    let mut content = default_campaign_content();
    let old_shrine = content
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == "location.old_shrine")
        .unwrap();
    old_shrine.spatial_discoveries = vec![
        SpatialDiscoveryContent {
            id: "evidence.first".into(),
            title: "First".into(),
            description: "First evidence.".into(),
            requires_evidence: vec!["evidence.second".into(), "missing.evidence".into()],
        },
        SpatialDiscoveryContent {
            id: "evidence.second".into(),
            title: "Second".into(),
            description: "Second evidence.".into(),
            requires_evidence: vec!["evidence.first".into()],
        },
    ];

    let issues = content.validate();
    assert!(issues
        .iter()
        .any(|issue| issue.contains("unknown evidence")));
    assert!(issues.iter().any(|issue| issue.contains("contain a cycle")));
}

#[test]
fn location_archetype_mod_overrides_preserve_omitted_base_values() {
    let mut base = default_campaign_content();
    let market = base
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == "location.hollow_market")
        .unwrap();
    market.archetype = Some(crate::geography::locations::LocationArchetype::Settlement);

    let mut legacy_mod = empty_mod_content();
    legacy_mod.world.locations.push(LocationContent {
        id: "location.hollow_market".into(),
        name: "Hollow Market".into(),
        description: "A modded description.".into(),
        dangerous: false,
        archetype: None,
        exits: Vec::new(),
        scene_art: None,
        scene_image: None,
        spatial_layout: None,
        geography: None,
        spatial_discoveries: Vec::new(),
        exit_sides: Default::default(),
        exit_placements: Default::default(),
    });
    merge_campaign_content(&mut base, legacy_mod);

    let market = base
        .world
        .locations
        .iter()
        .find(|location| location.id == "location.hollow_market")
        .unwrap();
    assert_eq!(
        market.archetype,
        Some(crate::geography::locations::LocationArchetype::Settlement)
    );

    let mut override_mod = empty_mod_content();
    override_mod.world.locations.push(LocationContent {
        id: "location.hollow_market".into(),
        name: "Hollow Market".into(),
        description: "A landmark now.".into(),
        dangerous: false,
        archetype: Some(crate::geography::locations::LocationArchetype::Landmark),
        exits: Vec::new(),
        scene_art: None,
        scene_image: None,
        spatial_layout: None,
        geography: None,
        spatial_discoveries: Vec::new(),
        exit_sides: Default::default(),
        exit_placements: Default::default(),
    });
    merge_campaign_content(&mut base, override_mod);

    let market = base
        .world
        .locations
        .iter()
        .find(|location| location.id == "location.hollow_market")
        .unwrap();
    assert_eq!(
        market.archetype,
        Some(crate::geography::locations::LocationArchetype::Landmark)
    );
}
