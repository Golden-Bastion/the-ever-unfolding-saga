use super::{
    fast_travel_to, find_known_travel_path, has_location_travel_route, travel_to, travel_via_route,
    MEDITATION_TARGETS,
};
use crate::model::{GameState, Location, RouteDirection, WorldRoute};
use crate::spatial::{GridPosition, LocalMap, Terrain};

#[test]
fn meditation_targets_advance_to_the_next_occurrence() {
    let current = 4;
    let morning = MEDITATION_TARGETS
        .iter()
        .find(|(slot, _)| *slot == 3)
        .map(|(slot, _)| *slot)
        .unwrap();
    let dusk = MEDITATION_TARGETS
        .iter()
        .find(|(slot, _)| *slot == 8)
        .map(|(slot, _)| *slot)
        .unwrap();
    assert_eq!((morning + 12 - current) % 12, 11);
    assert_eq!((dusk + 12 - current) % 12, 4);
}

#[test]
fn meditation_does_not_allow_a_zero_time_advance() {
    let current = 8;
    let dusk = MEDITATION_TARGETS
        .iter()
        .find(|(slot, _)| *slot == 8)
        .map(|(slot, _)| *slot)
        .unwrap();
    assert_eq!(((dusk + 12 - current) % 12).max(1), 1);
}

#[test]
fn travel_requires_reciprocal_world_graph_connections() {
    let mut game = crate::model::create_new_state(
        "Travel Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let source_id = game.character.location_id;
    let source = game
        .world
        .location_by_id(source_id)
        .expect("starting location should exist")
        .clone();
    let destination_id = game.world.allocate_id();
    game.world.locations.push(crate::model::Location {
        id: destination_id,
        name: "Distant Clearing".into(),
        description: "A clearing linked by a narrow path.".into(),
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

    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;
    let initial_history = game.world.history.len();
    assert!(!has_location_travel_route(&game, destination_id, None));
    assert!(travel_to(&mut game, destination_id).is_err());
    assert_eq!(game.character.location_id, source_id);
    assert_eq!(game.character.turn, initial_turn);
    assert_eq!(game.world.time_points, initial_time);
    assert_eq!(game.world.history.len(), initial_history);

    assert!(game.world.add_route(WorldRoute::new(
        "route.test.valid",
        source_id,
        destination_id,
        RouteDirection::Bidirectional,
    )));
    assert!(has_location_travel_route(
        &game,
        destination_id,
        Some("route.test.valid")
    ));
    travel_to(&mut game, destination_id).unwrap();
    assert_eq!(game.character.location_id, destination_id);
    assert_eq!(game.character.turn, initial_turn + 1);
}

#[test]
fn unsupported_route_crossings_block_direct_and_fast_travel() {
    use crate::geography::routes::{
        RouteCrossing, RouteCrossingKind, RouteGeography, RouteInfrastructure, RoutePlanningStatus,
        RouteProfile,
    };

    let (mut game, middle_id, destination_id) = fast_travel_fixture(false, true, true);
    let source_id = game.character.location_id;
    let route = game
        .world
        .routes
        .iter_mut()
        .find(|route| route.connects(source_id, middle_id))
        .expect("fixture route");
    route.geography = Some(RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::WaterCrossing,
        path: Vec::new(),
        crossings: vec![RouteCrossing {
            coordinate: crate::geography::MacroCoordinate::new(4, 4),
            kind: RouteCrossingKind::Ocean,
            infrastructure: RouteInfrastructure::Impassable,
            river_id: None,
            river_width: None,
            river_flow_direction: None,
            river_banks: None,
            lake_id: None,
            lake_banks: None,
        }],
        total_cost: 0,
    });

    let turn = game.character.turn;
    let time = game.world.time_points;
    assert!(!has_location_travel_route(&game, middle_id, None));
    assert!(travel_to(&mut game, middle_id).is_err());
    assert!(find_known_travel_path(&game, destination_id).is_none());
    assert_eq!(game.character.location_id, source_id);
    assert_eq!(game.character.turn, turn);
    assert_eq!(game.world.time_points, time);
}

fn fast_travel_fixture(
    dangerous_middle: bool,
    destination_known: bool,
    bidirectional_first_route: bool,
) -> (GameState, u64, u64) {
    let mut game = crate::model::create_new_state(
        "Fast Travel Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    game.world.routes.clear();
    for location in &mut game.world.locations {
        location.exits.clear();
    }
    game.campaign_content
        .as_mut()
        .expect("campaign content should be loaded")
        .events
        .retain(|event| event.trigger != "travel_arrival");

    let source_id = game.character.location_id;
    let source = game
        .world
        .location_by_id(source_id)
        .expect("starting location should exist")
        .clone();
    let middle_id = game.world.allocate_id();
    let destination_id = game.world.allocate_id();

    game.world.locations.push(Location {
        id: middle_id,
        name: "Known Waystation".into(),
        description: "A familiar stop along the road.".into(),
        region_id: source.region_id,
        dangerous: dangerous_middle,
        corpse_ids: Vec::new(),
        exits: if bidirectional_first_route {
            vec![source_id, destination_id]
        } else {
            vec![destination_id]
        },
        spatial_layout: None,
    });
    game.world.locations.push(Location {
        id: destination_id,
        name: "Known Destination".into(),
        description: "A place visited before.".into(),
        region_id: source.region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: vec![middle_id],
        spatial_layout: None,
    });
    game.world
        .location_by_id_mut(source_id)
        .expect("starting location should still exist")
        .exits = vec![middle_id];
    game.world.migrate_legacy_exits();
    game.world.discovered_route_ids = game
        .world
        .routes
        .iter()
        .map(|route| route.id.clone())
        .collect();

    game.world.local_maps.push(known_map(middle_id));
    if destination_known {
        game.world.local_maps.push(known_map(destination_id));
    }

    (game, middle_id, destination_id)
}

fn known_map(location_id: u64) -> LocalMap {
    LocalMap {
        location_id: format!("location.{location_id}"),
        generator_version: 1,
        layout_id: None,
        seed: location_id,
        width: 8,
        height: 8,
        terrain: vec![Terrain::Floor; 64],
        entry: GridPosition { x: 1, y: 1 },
        points_of_interest: Vec::new(),
        rooms: Vec::new(),
        doors: Vec::new(),
        exits: Vec::new(),
        discovered_tiles: Vec::new(),
        terrain_changes: Vec::new(),
        props: Vec::new(),
        interactables: Vec::new(),
        encounters: Vec::new(),
    }
}

#[test]
fn fast_travel_does_not_use_an_undiscovered_parallel_route() {
    let (mut game, middle_id, destination_id) = fast_travel_fixture(false, true, true);
    let source_id = game.character.location_id;
    assert!(game.world.add_route(WorldRoute::new(
        "route.aaa.undiscovered",
        source_id,
        middle_id,
        RouteDirection::Bidirectional,
    )));
    assert!(!game.world.is_route_discovered("route.aaa.undiscovered"));

    fast_travel_to(&mut game, destination_id).unwrap();

    assert!(!game.world.is_route_discovered("route.aaa.undiscovered"));
}

#[test]
fn physical_route_use_records_route_and_regional_discovery() {
    let (mut game, middle_id, _) = fast_travel_fixture(false, false, true);
    game.world.discovered_route_ids.clear();
    let source_id = game.character.location_id;
    let route_id = game
        .world
        .routes_from(source_id)
        .find(|(_, destination)| *destination == middle_id)
        .map(|(route, _)| route.id.clone())
        .expect("physical passage route should exist");
    let region_id = game.world.location_by_id(middle_id).unwrap().region_id;

    travel_via_route(&mut game, middle_id, &route_id).unwrap();

    assert!(game.world.is_route_discovered(&route_id));
    assert!(game.world.is_region_surveyed(region_id));
}

#[test]
fn physical_route_travel_uses_persisted_distance_and_terrain_cost() {
    use crate::geography::routes::{RouteGeography, RoutePlanningStatus, RouteProfile};
    use crate::geography::MacroCoordinate;

    let (mut game, middle_id, _) = fast_travel_fixture(false, false, true);
    game.world.geography_seed = None;
    game.world.generation = None;
    let route_id = game
        .world
        .routes_from(game.character.location_id)
        .find(|(_, destination)| *destination == middle_id)
        .map(|(route, _)| route.id.clone())
        .expect("fixture route");
    let route = game
        .world
        .routes
        .iter_mut()
        .find(|route| route.id == route_id)
        .unwrap();
    route.geography = Some(RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::Trail,
        path: vec![MacroCoordinate::new(0, 0), MacroCoordinate::new(64, 0)],
        crossings: Vec::new(),
        total_cost: 300,
    });
    let before = game.world.day * 12 + game.world.time_points;
    let initial_turn = game.character.turn;

    travel_via_route(&mut game, middle_id, &route_id).unwrap();

    assert_eq!(game.world.day * 12 + game.world.time_points, before + 6);
    assert_eq!(game.character.turn, initial_turn + 1);
}

#[test]
fn fast_travel_prefers_a_longer_hop_count_when_its_total_time_is_lower() {
    use crate::geography::routes::{RouteGeography, RoutePlanningStatus, RouteProfile};
    use crate::geography::MacroCoordinate;

    let (mut game, middle_id, destination_id) = fast_travel_fixture(false, true, true);
    game.world.geography_seed = None;
    game.world.generation = None;
    let source_id = game.character.location_id;
    assert!(game.world.add_route(WorldRoute::new(
        "route.direct.slow",
        source_id,
        destination_id,
        RouteDirection::Bidirectional,
    )));
    game.world.mark_route_discovered("route.direct.slow");
    let direct_route = game
        .world
        .routes
        .iter_mut()
        .find(|route| route.id == "route.direct.slow")
        .unwrap();
    direct_route.geography = Some(RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::Trail,
        path: vec![MacroCoordinate::new(0, 0), MacroCoordinate::new(160, 0)],
        crossings: Vec::new(),
        total_cost: 500,
    });

    assert_eq!(
        find_known_travel_path(&game, destination_id),
        Some(vec![middle_id, destination_id])
    );
    let before = game.world.day * 12 + game.world.time_points;
    let traveled = fast_travel_to(&mut game, destination_id).unwrap();

    assert_eq!(traveled, vec![middle_id, destination_id]);
    assert_eq!(game.world.day * 12 + game.world.time_points, before + 4);
}

#[test]
fn least_time_fast_travel_selects_the_exact_cheapest_parallel_route() {
    use crate::geography::routes::{RouteGeography, RoutePlanningStatus, RouteProfile};
    use crate::geography::MacroCoordinate;

    let (mut game, middle_id, _) = fast_travel_fixture(false, true, true);
    game.world.geography_seed = None;
    game.world.generation = None;
    let source_id = game.character.location_id;
    let slower_route_id = game
        .world
        .routes_from(source_id)
        .find(|(_, destination)| *destination == middle_id)
        .map(|(route, _)| route.id.clone())
        .expect("fixture route");
    let slower_route = game
        .world
        .routes
        .iter_mut()
        .find(|route| route.id == slower_route_id)
        .unwrap();
    slower_route.geography = Some(RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile: RouteProfile::Trail,
        path: vec![MacroCoordinate::new(0, 0), MacroCoordinate::new(160, 0)],
        crossings: Vec::new(),
        total_cost: 500,
    });
    assert!(game.world.add_route(WorldRoute::new(
        "route.zzz.fast",
        source_id,
        middle_id,
        RouteDirection::Bidirectional,
    )));
    game.world.mark_route_discovered("route.zzz.fast");

    let path = super::find_known_travel_route_path(&game, middle_id)
        .expect("the known parallel route should be available");
    assert_eq!(path.len(), 1);
    assert_eq!(path[0].route_id, "route.zzz.fast");
    assert_eq!(path[0].time_points, 2);

    let before = game.world.day * 12 + game.world.time_points;
    assert_eq!(
        fast_travel_to(&mut game, middle_id).unwrap(),
        vec![middle_id]
    );
    assert_eq!(game.world.day * 12 + game.world.time_points, before + 2);
}

#[test]
fn fast_travel_requires_each_route_to_be_discovered() {
    let (mut game, _, destination_id) = fast_travel_fixture(false, true, true);
    game.world.discovered_route_ids.clear();

    assert!(find_known_travel_path(&game, destination_id).is_none());
    assert!(fast_travel_to(&mut game, destination_id).is_err());
}

#[test]
fn fast_travel_follows_the_shortest_known_route_and_applies_normal_travel_per_hop() {
    let (mut game, middle_id, destination_id) = fast_travel_fixture(false, true, true);
    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;

    assert_eq!(
        find_known_travel_path(&game, destination_id),
        Some(vec![middle_id, destination_id])
    );

    let traveled = fast_travel_to(&mut game, destination_id).unwrap();

    assert_eq!(traveled, vec![middle_id, destination_id]);
    assert_eq!(game.character.location_id, destination_id);
    assert_eq!(game.character.turn, initial_turn + 2);
    assert_eq!(game.world.time_points, initial_time + 4);
    assert_eq!(
        game.world
            .history
            .iter()
            .filter(|entry| entry.text.contains("traveled to"))
            .count(),
        2
    );
}

#[test]
fn fast_travel_rejects_unvisited_destinations_without_changing_world_state() {
    let (mut game, _, destination_id) = fast_travel_fixture(false, false, true);
    let initial_location = game.character.location_id;
    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;
    let initial_history = game.world.history.len();

    assert!(find_known_travel_path(&game, destination_id).is_none());
    assert!(fast_travel_to(&mut game, destination_id).is_err());
    assert_eq!(game.character.location_id, initial_location);
    assert_eq!(game.character.turn, initial_turn);
    assert_eq!(game.world.time_points, initial_time);
    assert_eq!(game.world.history.len(), initial_history);
}

#[test]
fn fast_travel_follows_one_way_routes_only_in_the_allowed_direction() {
    let (mut game, middle_id, destination_id) = fast_travel_fixture(false, true, false);
    let source_id = game.character.location_id;

    assert_eq!(
        find_known_travel_path(&game, destination_id),
        Some(vec![middle_id, destination_id])
    );
    game.character.location_id = middle_id;
    assert!(!has_location_travel_route(&game, source_id, None));
    assert!(travel_to(&mut game, source_id).is_err());
}

#[test]
fn fast_travel_pauses_at_a_consequential_arrival_event() {
    let (mut game, middle_id, destination_id) = fast_travel_fixture(false, true, true);
    game.campaign_content
        .as_mut()
        .expect("campaign content should exist")
        .events
        .push(crate::content::definitions::EventContent {
            id: "test.fast_travel_arrival".into(),
            trigger: "travel_arrival".into(),
            weight: 1,
            chance_percent: Some(100),
            cooldown_turns: None,
            conditions: None,
            effects: vec![crate::content::definitions::EventEffectContent::Message {
                text: "A traveller calls from the roadside.".into(),
            }],
        });
    let initial_turn = game.character.turn;

    let traveled = fast_travel_to(&mut game, destination_id).unwrap();

    assert_eq!(traveled, vec![middle_id]);
    assert_eq!(game.character.location_id, middle_id);
    assert!(game.world.history.iter().any(|entry| {
        entry.turn > initial_turn && entry.entry_type == crate::model::HistoryEntryType::Event
    }));
}

#[test]
fn fast_travel_stops_at_an_intermediate_threat_instead_of_bypassing_it() {
    let (mut game, middle_id, destination_id) = fast_travel_fixture(true, true, true);
    let initial_turn = game.character.turn;
    let initial_time = game.world.time_points;

    let traveled = fast_travel_to(&mut game, destination_id).unwrap();

    assert_eq!(traveled, vec![middle_id]);
    assert_eq!(game.character.location_id, middle_id);
    assert!(game.threat.active);
    assert_eq!(game.character.turn, initial_turn + 1);
    assert_eq!(game.world.time_points, initial_time + 2);
}
