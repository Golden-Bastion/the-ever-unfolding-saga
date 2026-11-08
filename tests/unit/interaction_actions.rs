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
