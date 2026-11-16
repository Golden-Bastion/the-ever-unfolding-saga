use super::*;

#[test]
fn interaction_targets_have_stable_distinct_keys() {
    assert_eq!(InteractionTarget::Npc(7).stable_key(), "npc:7");
    assert_eq!(InteractionTarget::Corpse(7).stable_key(), "corpse:7");
    assert_eq!(
        InteractionTarget::PointOfInterest("poi.shrine".into()).stable_key(),
        "poi:poi.shrine"
    );
    assert_eq!(
        InteractionTarget::Door {
            id: "door.threshold".into()
        }
        .stable_key(),
        "door:door.threshold"
    );
    assert_eq!(
        InteractionTarget::Passage {
            id: "exit.north".into(),
            destination_id: 12
        }
        .stable_key(),
        "passage:exit.north:12"
    );
}

#[test]
fn discovery_filters_stale_npcs_and_unsearchable_remains() {
    let mut map = crate::spatial::generate_local_map(
        17,
        "location.interaction_filter",
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.exits.clear();

    let mut game = crate::model::create_new_state(
        "Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_id = game.character.location_id;
    game.corpses.push(crate::model::Corpse {
        id: 701,
        former_name: "Empty Wanderer".into(),
        former_title: "Pilgrim".into(),
        location_id,
        turn_of_death: 0,
        inventory: Vec::new(),
        epitaph: String::new(),
        scavenged: false,
    });
    map.interactables = vec![
        crate::spatial::SpatialEntityPlacement {
            entity_id: 700,
            kind: crate::spatial::SpatialEntityKind::Npc,
            position: map.entry,
        },
        crate::spatial::SpatialEntityPlacement {
            entity_id: 701,
            kind: crate::spatial::SpatialEntityKind::Corpse,
            position: map.entry,
        },
    ];

    let position = (f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    let available = discover_available_interactions(&map, &game, position);

    assert!(
        available.is_empty(),
        "stale actors and corpses without searchable inventory must not be offered",
    );
}

#[test]
fn undiscovered_points_of_interest_do_not_offer_an_examine_action() {
    let mut map = crate::spatial::generate_local_map(
        23,
        "location.hidden_discovery",
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.exits.clear();
    map.interactables.clear();
    let poi = &map.points_of_interest[0];
    let position = (
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );

    let game = crate::model::create_new_state(
        "Discovery Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );

    assert!(discover_available_interactions(&map, &game, position).is_empty());

    map.points_of_interest[0].discovered = true;
    let available = discover_available_interactions(&map, &game, position);
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].action, InteractionActionKind::Examine);
    assert_eq!(
        available[0].target,
        InteractionTarget::PointOfInterest(map.points_of_interest[0].id.clone())
    );
}

#[test]
fn evidence_prerequisites_gate_examine_actions_per_character() {
    let mut map = crate::spatial::generate_local_map(
        51,
        "location.evidence_filter",
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.exits.clear();
    map.interactables.clear();
    let poi = map
        .points_of_interest
        .first_mut()
        .expect("map has a prototype POI");
    poi.id = "evidence.second".into();
    poi.title = "Second clue".into();
    poi.discovered = true;
    poi.interacted = true;
    poi.prerequisites = vec!["evidence.first".into()];
    let position = (
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );

    let mut game = crate::model::create_new_state(
        "Evidence Filter Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );

    assert!(
        discover_available_interactions(&map, &game, position).is_empty(),
        "a revealed clue remains uninterpretable until its prerequisite is known"
    );

    game.character.record_evidence("evidence.first");
    let available = discover_available_interactions(&map, &game, position);
    assert_eq!(available.len(), 1);
    assert_eq!(
        available[0].target,
        InteractionTarget::PointOfInterest("evidence.second".into())
    );

    game.character.record_evidence("evidence.second");
    assert!(
        discover_available_interactions(&map, &game, position).is_empty(),
        "the same character should not be offered evidence they already understand"
    );
}

#[test]
fn discovery_hides_stale_and_one_way_world_passages() {
    let mut map = crate::spatial::generate_local_map(
        31,
        "location.route_filter",
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.interactables.clear();
    map.exits.clear();

    let mut game = crate::model::create_new_state(
        "Route Filter Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let source_id = game.character.location_id;
    let source = game.world.location_by_id(source_id).unwrap().clone();
    let destination_id = game.world.allocate_id();
    game.world.locations.push(crate::model::Location {
        id: destination_id,
        name: "One-way Destination".into(),
        description: "A destination without a return link.".into(),
        region_id: source.region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: Vec::new(),
        spatial_layout: None,
    });
    game.world
        .location_by_id_mut(source_id)
        .unwrap()
        .exits
        .push(destination_id);
    map.exits.push(MapExit {
        id: "exit.one-way".into(),
        position: map.entry,
        target_location_id: destination_id,
    });
    let position = (f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);

    assert!(discover_available_interactions(&map, &game, position).is_empty());

    game.world
        .location_by_id_mut(destination_id)
        .unwrap()
        .exits
        .push(source_id);
    let available = discover_available_interactions(&map, &game, position);
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].action, InteractionActionKind::Enter);
}

#[test]
fn nearby_interior_doors_offer_state_appropriate_contextual_actions() {
    let mut map = crate::spatial::generate_local_map_with_layout(
        42,
        "location.door_interaction",
        crate::spatial::LocalMapConfig::default(),
        Some("shrine_chamber"),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.exits.clear();
    map.interactables.clear();
    let door_id = map.doors[0].id.clone();
    let game = crate::model::create_new_state(
        "Door Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let player = (
        f32::from(map.doors[0].edge.a.x) + 0.5,
        f32::from(map.doors[0].edge.a.y) + 0.5,
    );

    map.doors[0].state = crate::spatial::DoorState::Closed;
    let available = discover_available_interactions(&map, &game, player);
    assert!(available.iter().any(|item| {
        item.action == InteractionActionKind::OpenDoor
            && item.target
                == (InteractionTarget::Door {
                    id: door_id.clone(),
                })
    }));

    map.doors[0].state = crate::spatial::DoorState::Open;
    let available = discover_available_interactions(&map, &game, player);
    assert!(available
        .iter()
        .any(|item| item.action == InteractionActionKind::CloseDoor));

    map.doors[0].state = crate::spatial::DoorState::Locked;
    let available = discover_available_interactions(&map, &game, player);
    assert!(available
        .iter()
        .any(|item| item.action == InteractionActionKind::TryLockedDoor));
}
