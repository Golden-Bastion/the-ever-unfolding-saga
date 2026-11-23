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
        InteractionTarget::Encounter("encounter.gate.1".into()).stable_key(),
        "encounter:encounter.gate.1"
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
fn nearby_active_encounters_offer_a_stable_confront_action() {
    let mut game = crate::model::create_new_state(
        "Encounter Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_id = game.character.location_id;
    let mut map = crate::spatial::generate_local_map(
        18,
        format!("location.{location_id}"),
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.exits.clear();
    map.interactables.clear();
    assert!(crate::spatial::ensure_spatial_encounter(
        &mut map,
        "encounter.test.1".into(),
        "Ash Marauder".into(),
        "A hostile figure blocks the way.".into(),
        18,
        3,
        "Marauder Token".into(),
    ));
    let encounter = map.encounters[0].clone();
    let nearby = (
        f32::from(encounter.position.x) + 0.5,
        f32::from(encounter.position.y) + 0.5,
    );

    assert!(discover_available_interactions(&map, &game, nearby)
        .iter()
        .any(|item| item.action == InteractionActionKind::EngageEncounter));

    game.threat.activate(
        location_id,
        "A nearby threat",
        "Something hostile remains here.",
    );
    let available = discover_available_interactions(&map, &game, nearby);
    assert!(available.iter().any(|item| {
        item.action == InteractionActionKind::EngageEncounter
            && item.target == InteractionTarget::Encounter(encounter.id.clone())
            && item.label == "Confront: Ash Marauder"
    }));

    let other_location = game
        .world
        .locations
        .iter()
        .find(|location| location.id != location_id)
        .expect("world should contain another location")
        .id;
    game.threat
        .activate(other_location, "Distant threat", "Elsewhere.");
    assert!(discover_available_interactions(&map, &game, nearby)
        .iter()
        .any(|item| item.action == InteractionActionKind::EngageEncounter));

    let far_away = (f32::from(map.width) - 1.5, f32::from(map.height) - 1.5);
    assert!(!discover_available_interactions(&map, &game, far_away)
        .iter()
        .any(|item| item.action == InteractionActionKind::EngageEncounter));

    map.encounters[0].status = crate::spatial::SpatialEncounterStatus::Defeated;
    map.encounters[0].current_hp = 0;
    assert!(!discover_available_interactions(&map, &game, nearby)
        .iter()
        .any(|item| item.action == InteractionActionKind::EngageEncounter));
}

#[test]
fn multiple_nearby_encounters_have_distinct_confront_targets() {
    let game = crate::model::create_new_state(
        "Multiple Encounter Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_id = game.character.location_id;
    let mut map = crate::spatial::generate_local_map(
        19,
        format!("location.{location_id}"),
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.exits.clear();
    map.interactables.clear();
    assert!(crate::spatial::ensure_spatial_encounter(
        &mut map,
        "encounter.nearby.first".into(),
        "Ash Marauder".into(),
        "A hostile figure blocks the way.".into(),
        18,
        3,
        "Marauder Token".into(),
    ));
    assert!(crate::spatial::ensure_spatial_encounter(
        &mut map,
        "encounter.nearby.second".into(),
        "Cinder Scout".into(),
        "A second hostile figure watches from nearby.".into(),
        11,
        2,
        "Scout's Token".into(),
    ));

    let first = map.encounters[0].clone();
    let adjacent_candidates = [
        first
            .position
            .x
            .checked_add(1)
            .map(|x| crate::spatial::GridPosition {
                x,
                y: first.position.y,
            }),
        first
            .position
            .x
            .checked_sub(1)
            .map(|x| crate::spatial::GridPosition {
                x,
                y: first.position.y,
            }),
        first
            .position
            .y
            .checked_add(1)
            .map(|y| crate::spatial::GridPosition {
                x: first.position.x,
                y,
            }),
        first
            .position
            .y
            .checked_sub(1)
            .map(|y| crate::spatial::GridPosition {
                x: first.position.x,
                y,
            }),
    ];
    let adjacent = adjacent_candidates
        .into_iter()
        .flatten()
        .find(|position| {
            map.contains(*position)
                && map.is_walkable(*position)
                && *position != map.entry
                && *position != map.encounters[1].position
        })
        .expect("the open test map should have an adjacent walkable cell");
    map.encounters[1].position = adjacent;
    assert!(map.validate_encounters().is_ok());

    let nearby = (
        f32::from(first.position.x) + 0.5,
        f32::from(first.position.y) + 0.5,
    );
    let available = discover_available_interactions(&map, &game, nearby)
        .into_iter()
        .filter(|item| item.action == InteractionActionKind::EngageEncounter)
        .collect::<Vec<_>>();
    assert_eq!(available.len(), 2);
    assert!(available.iter().any(|item| {
        item.target == InteractionTarget::Encounter("encounter.nearby.first".into())
    }));
    assert!(available.iter().any(|item| {
        item.target == InteractionTarget::Encounter("encounter.nearby.second".into())
    }));
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
fn discovery_requires_passages_to_match_authoritative_route_records() {
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
    let route_id = "route.test.one_way";
    assert!(game.world.add_route(crate::model::WorldRoute::new(
        route_id,
        source_id,
        destination_id,
        crate::model::RouteDirection::AtoB,
    )));
    map.exits.push(MapExit {
        id: "exit.one-way".into(),
        route_id: Some(route_id.into()),
        route_profile: None,
        frontier: None,
        cave_target: None,
        position: map.entry,
        target_location_id: destination_id,
    });
    let position = (f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);

    let available = discover_available_interactions(&map, &game, position);
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].action, InteractionActionKind::Enter);

    map.exits[0].route_id = Some("route.stale".into());
    assert!(discover_available_interactions(&map, &game, position).is_empty());
}

#[test]
fn resolved_lake_route_passages_are_labeled_as_ferry_landings() {
    use crate::geography::river_geometry::{RiverPosition, RIVER_POSITION_SCALE};
    use crate::geography::routes::{
        LakeBankIntersection, RouteCrossing, RouteCrossingKind, RouteGeography,
        RouteInfrastructure, RoutePlanningStatus, RouteProfile,
    };

    let mut game = crate::model::create_new_state(
        "Ferry Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let source_id = game.character.location_id;
    let source = game.world.location_by_id(source_id).unwrap().clone();
    let mut map = crate::spatial::generate_local_map(
        32,
        format!("location.{source_id}"),
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.interactables.clear();
    map.encounters.clear();

    let crossing = |resolved: bool| RouteCrossing {
        coordinate: crate::geography::MacroCoordinate::new(8, 12),
        kind: RouteCrossingKind::Lake,
        infrastructure: RouteInfrastructure::Ferry,
        river_id: None,
        river_width: None,
        river_flow_direction: None,
        river_banks: None,
        lake_id: Some("lake.interaction.fixture".into()),
        lake_banks: resolved.then_some(LakeBankIntersection {
            entry: RiverPosition {
                x_q16: 8 * RIVER_POSITION_SCALE,
                y_q16: 12 * RIVER_POSITION_SCALE,
            },
            exit: RiverPosition {
                x_q16: 9 * RIVER_POSITION_SCALE,
                y_q16: 12 * RIVER_POSITION_SCALE,
            },
        }),
    };

    let mut resolved_route_id = None;
    for (name, resolved) in [("Resolved Ferry", true), ("Unresolved Crossing", false)] {
        let destination_id = game.world.allocate_id();
        game.world.locations.push(crate::model::Location {
            id: destination_id,
            name: name.into(),
            description: "A route destination.".into(),
            region_id: source.region_id,
            dangerous: false,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
            spatial_layout: None,
        });
        let route_id = format!("route.ferry.{destination_id}");
        let mut route = crate::model::WorldRoute::new(
            route_id.clone(),
            source_id,
            destination_id,
            crate::model::RouteDirection::Bidirectional,
        );
        route.geography = Some(RouteGeography {
            status: RoutePlanningStatus::Planned,
            profile: RouteProfile::WaterCrossing,
            path: Vec::new(),
            crossings: vec![crossing(resolved)],
            total_cost: 0,
        });
        assert!(game.world.add_route(route));
        if resolved {
            resolved_route_id = Some(route_id.clone());
        }
        map.exits.push(MapExit {
            id: format!("exit.{destination_id}"),
            route_id: Some(route_id),
            route_profile: Some(RouteProfile::WaterCrossing),
            frontier: None,
            cave_target: None,
            position: map.entry,
            target_location_id: destination_id,
        });
    }

    let player = (f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    let labels = discover_available_interactions(&map, &game, player)
        .into_iter()
        .map(|interaction| interaction.label)
        .collect::<Vec<_>>();
    assert!(labels.iter().any(|label| {
        label.starts_with("Unavailable: This route requires a constructed ferry service")
    }));
    assert!(labels
        .iter()
        .any(|label| label.starts_with("Unavailable: The lake crossing needs verified shores")));

    let resolved_route_id = resolved_route_id.expect("resolved ferry route should be registered");
    assert!(game.world.add_route_infrastructure(
        crate::model::RouteInfrastructureRecord {
            id: "infrastructure.ferry.interaction.fixture".into(),
            route_id: resolved_route_id,
            crossing_index: Some(0),
            kind: crate::model::InfrastructureKind::Ferry,
            construction: crate::model::InfrastructureConstruction::Built,
            condition: crate::model::InfrastructureCondition::Operational,
            owner: None,
            access: crate::model::InfrastructureAccess::Public,
        }
    ));

    let labels = discover_available_interactions(&map, &game, player)
        .into_iter()
        .map(|interaction| interaction.label)
        .collect::<Vec<_>>();
    assert!(labels
        .iter()
        .any(|label| label == "Board ferry to Resolved Ferry"));
    assert!(labels
        .iter()
        .any(|label| label.starts_with("Unavailable: The lake crossing needs verified shores")));
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

#[test]
fn nearby_pending_frontier_passage_offers_exploration_without_a_world_route() {
    let game = crate::model::create_new_state(
        "Frontier Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let mut map = crate::spatial::generate_local_map(
        91,
        format!("location.{}", game.character.location_id),
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.interactables.clear();
    map.encounters.clear();
    let frontier = crate::geography::frontier::FrontierTarget {
        id: "frontier.interaction.east".into(),
        target_region: crate::geography::RegionCoordinate::new(1, 0),
        direction: crate::geography::frontier::FrontierDirection::East,
    };
    map.exits.push(MapExit {
        id: "location.frontier.interaction.east".into(),
        route_id: None,
        route_profile: None,
        frontier: Some(frontier),
        cave_target: None,
        position: map.entry,
        target_location_id: 0,
    });
    let player = (f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);

    let available = discover_available_interactions(&map, &game, player);
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].action, InteractionActionKind::Enter);
    assert_eq!(available[0].label, "Explore eastern frontier");
    assert_eq!(
        available[0].target,
        InteractionTarget::Passage {
            id: "location.frontier.interaction.east".into(),
            destination_id: 0,
        }
    );
}

#[test]
fn pending_cave_passages_are_discoverable_without_an_existing_world_route() {
    use crate::geography::caves::{CaveConnectionKind, CaveConnectionTarget};
    use crate::geography::MacroCoordinate;

    let game = crate::model::create_new_state(
        "Cave Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_id = game.character.location_id;
    let mut map = crate::spatial::generate_local_map(
        31,
        format!("location.{location_id}"),
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    let target = CaveConnectionTarget {
        id: "vertical.test.entrance".into(),
        destination_anchor: MacroCoordinate {
            x: 20,
            y: 30,
            stratum: -1,
        },
        destination_location_id: None,
        kind: CaveConnectionKind::SurfaceEntrance,
    };
    map.exits.push(crate::spatial::MapExit {
        id: "location.test.cave.entrance".into(),
        route_id: None,
        route_profile: None,
        frontier: None,
        cave_target: Some(target),
        position: map.entry,
        target_location_id: 0,
    });
    let player = (f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    let available = discover_available_interactions(&map, &game, player);
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].action, InteractionActionKind::Enter);
    assert_eq!(available[0].label, "Descend into cave");
}
