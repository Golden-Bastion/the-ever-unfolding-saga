use super::*;
use crate::model::{
    Location, Quest, QuestObjective, QuestObjectiveKind, Region, RouteDirection, WorldMode,
    WorldRoute,
};
use crate::spatial::{GridPosition, LocalMap, Terrain};

fn test_state() -> GameState {
    let mut state = crate::model::create_new_state(
        "World Map Test",
        WorldMode::New,
        "Tester".to_string(),
        "Wanderer".to_string(),
    );
    state.world.routes.clear();
    state.world.regions = vec![Region {
        id: 1,
        name: "Ashen Vale".to_string(),
        description: "A valley of old roads.".to_string(),
        location_ids: vec![10, 11, 12, 13, 14],
        coordinate: None,
    }];
    state.world.locations = vec![
        location(10, "Crossroads", vec![11, 12]),
        location(11, "Old Shrine", vec![10, 13]),
        location(12, "Cinder Hamlet", vec![10]),
        location(13, "Sunken Watch", vec![11]),
        location(14, "Far Ruin", vec![]),
    ];
    state.world.migrate_legacy_exits();
    state.world.local_maps.clear();
    state.character.location_id = 10;
    state
}

fn location(id: u64, name: &str, exits: Vec<u64>) -> Location {
    Location {
        id,
        name: name.to_string(),
        description: format!("A place called {name}."),
        region_id: 1,
        dangerous: id == 13,
        corpse_ids: Vec::new(),
        exits,
        spatial_layout: None,
    }
}

fn explored_map(location_id: u64) -> LocalMap {
    LocalMap {
        location_id: format!("location.{location_id}"),
        generator_version: 1,
        layout_id: None,
        seed: 0,
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
fn tracked_quest_marks_the_first_incomplete_visit_objective_and_reveals_its_location() {
    let mut state = test_state();
    let mut quest = Quest::new(
        701,
        "quest.multi_step",
        "A Three-Step Journey",
        "Follow the trail",
        14,
        0,
        0,
        "",
        "",
    );
    quest.offered = true;
    let mut completed = QuestObjective::new(QuestObjectiveKind::VisitLocation, "Crossroads", 1);
    completed.progress = 1;
    completed.completed = true;
    quest.objectives = vec![
        completed,
        QuestObjective::new(QuestObjectiveKind::VisitLocation, "Far Ruin", 1),
        QuestObjective::new(QuestObjectiveKind::VisitLocation, "Old Shrine", 1),
    ];
    state.quests.push(quest);
    state.character.tracked_quest_id = Some(701);

    let view = build_view(&state);

    let destination = view
        .nodes
        .iter()
        .find(|node| node.location.id == 14)
        .expect("the tracked destination should be shown on the map");
    assert!(destination.tracked_objective);
    assert!(!destination.explored);
    assert!(destination.fast_travel_index.is_none());
    assert!(view
        .routes
        .iter()
        .all(|route| route.from_id != 14 && route.to_id != 14));
    assert!(
        !view
            .nodes
            .iter()
            .find(|node| node.location.id == 11)
            .unwrap()
            .tracked_objective
    );
}

#[test]
fn tracked_evidence_objective_resolves_to_the_location_that_owns_the_discovery() {
    let mut state = test_state();
    state.campaign_content = Some(crate::content::load_campaign_content());
    let mut quest = Quest::new(
        702,
        "quest.evidence",
        "Read the Signs",
        "Follow the clues",
        12,
        0,
        0,
        "",
        "",
    );
    quest.offered = true;
    quest.objectives = vec![QuestObjective::discover_evidence(
        "poi.old_shrine.black_altar",
        "Black Altar",
    )];
    state.quests.push(quest);
    state.character.tracked_quest_id = Some(702);

    let view = build_view(&state);

    assert!(
        view.nodes
            .iter()
            .find(|node| node.location.id == 11)
            .expect("Old Shrine should be visible as the evidence location")
            .tracked_objective
    );
    assert!(
        !view
            .nodes
            .iter()
            .find(|node| node.location.id == 12)
            .unwrap()
            .tracked_objective
    );
}

#[test]
fn untracked_quests_do_not_add_a_world_map_destination_marker() {
    let mut state = test_state();
    let mut quest = Quest::new(
        704,
        "quest.untracked",
        "Untracked",
        "No marker",
        14,
        0,
        0,
        "",
        "",
    );
    quest.offered = true;
    quest.objectives = vec![QuestObjective::new(
        QuestObjectiveKind::VisitLocation,
        "Far Ruin",
        1,
    )];
    state.quests.push(quest);

    let view = build_view(&state);

    assert!(view.nodes.iter().all(|node| !node.tracked_objective));
    assert!(!view.nodes.iter().any(|node| node.location.id == 14));
}

#[test]
fn unresolved_tracked_objectives_do_not_guess_a_destination() {
    let mut state = test_state();
    let mut quest = Quest::new(
        703,
        "quest.unknown_evidence",
        "Unknown Trail",
        "No authored destination",
        11,
        0,
        0,
        "",
        "",
    );
    quest.offered = true;
    quest.objectives = vec![QuestObjective::discover_evidence(
        "poi.not_in_campaign",
        "Unknown Clue",
    )];
    state.quests.push(quest);
    state.character.tracked_quest_id = Some(703);

    let view = build_view(&state);

    assert!(view.nodes.iter().all(|node| !node.tracked_objective));
}

#[test]
fn map_does_not_reveal_routes_before_their_local_passages_are_explored() {
    let state = test_state();

    let view = build_view(&state);

    let ids = view
        .nodes
        .iter()
        .map(|node| node.location.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![10]);
    assert!(view.nodes.iter().any(|node| node.current && node.explored));
    assert_eq!(view.nodes.iter().filter(|node| node.reachable).count(), 0);
    assert!(!ids.contains(&11));
    assert!(!ids.contains(&12));
    assert!(view.routes.is_empty());
    assert!(!ids.contains(&13));
    assert!(!ids.contains(&14));
    assert!(view.fast_travel_destinations.is_empty());
    assert!(view
        .nodes
        .iter()
        .all(|node| node.fast_travel_index.is_none()));
}

#[test]
fn broad_regional_overview_is_hidden_until_the_region_is_surveyed() {
    let mut state = test_state();
    state.world.geography_seed = Some(123);
    state.world.geography_ruleset_version = crate::geography::GEOGRAPHY_RULESET_VERSION;
    state.world.regions[0].coordinate = Some(crate::geography::RegionCoordinate::ORIGIN);

    assert!(build_view(&state).current_region_overview.is_empty());

    state.world.mark_region_surveyed(1);
    let first = build_view(&state).current_region_overview;
    let second = build_view(&state).current_region_overview;

    assert!(!first.is_empty());
    assert!(first.len() <= 4);
    assert_eq!(first, second);
}

#[test]
fn discovering_a_local_passage_reveals_its_route_but_not_other_connections() {
    let mut state = test_state();
    let mut map = explored_map(10);
    let exit_position = GridPosition { x: 6, y: 6 };
    map.exits.push(crate::spatial::MapExit {
        id: "exit.to.shrine".into(),
        route_id: Some("route.compat.legacy.10.11".into()),
        route_profile: None,
        frontier: None,
        cave_target: None,
        position: exit_position,
        target_location_id: 11,
    });
    assert!(!map.is_tile_discovered(exit_position));
    state.world.local_maps.push(map.clone());

    let hidden = build_view(&state);
    assert_eq!(
        hidden.nodes.iter().map(|node| node.location.id).collect::<Vec<_>>(),
        vec![10]
    );

    map.mark_tile_discovered(exit_position);
    state.world.local_maps[0] = map;
    let revealed = build_view(&state);
    assert_eq!(
        revealed.nodes.iter().map(|node| node.location.id).collect::<Vec<_>>(),
        vec![10, 11]
    );
    assert_eq!(revealed.routes.len(), 1);
    assert_eq!(
        (revealed.routes[0].from_id, revealed.routes[0].to_id),
        (10, 11)
    );
}

#[test]
fn explored_locations_extend_the_map_by_one_route_without_exposing_the_frontier_beyond_it() {
    let mut state = test_state();
    state.world.local_maps.push(explored_map(11));
    state.world.discovered_route_ids = vec![
        "route.compat.legacy.10.11".into(),
        "route.compat.legacy.11.13".into(),
    ];

    let view = build_view(&state);

    let ids = view
        .nodes
        .iter()
        .map(|node| node.location.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![10, 11, 13]);
    assert!(
        view.nodes
            .iter()
            .find(|node| node.location.id == 11)
            .unwrap()
            .explored
    );
    assert!(
        !view
            .nodes
            .iter()
            .find(|node| node.location.id == 13)
            .unwrap()
            .explored
    );
    assert!(!ids.contains(&14));
}

#[test]
fn previously_visited_locations_become_fast_travel_targets_along_known_routes() {
    let mut state = test_state();
    state.world.local_maps.push(explored_map(11));
    state.world.local_maps.push(explored_map(13));
    state.world.discovered_route_ids = vec![
        "route.compat.legacy.10.11".into(),
        "route.compat.legacy.11.13".into(),
    ];

    let view = build_view(&state);

    let destinations = view
        .fast_travel_destinations
        .iter()
        .map(|destination| (destination.location.id, destination.route_steps))
        .collect::<Vec<_>>();
    assert_eq!(destinations, vec![(11, 1), (13, 2)]);
    assert_eq!(
        view.nodes
            .iter()
            .find(|node| node.location.id == 11)
            .unwrap()
            .fast_travel_index,
        Some(0)
    );
    assert_eq!(
        view.nodes
            .iter()
            .find(|node| node.location.id == 13)
            .unwrap()
            .fast_travel_steps,
        Some(2)
    );
}

#[test]
fn unvisited_frontier_locations_are_visible_but_not_fast_travel_targets() {
    let mut state = test_state();
    state.world.local_maps.push(explored_map(11));
    state.world.discovered_route_ids = vec![
        "route.compat.legacy.10.11".into(),
        "route.compat.legacy.11.13".into(),
    ];

    let view = build_view(&state);
    let frontier = view
        .nodes
        .iter()
        .find(|node| node.location.id == 13)
        .unwrap();

    assert!(!frontier.explored);
    assert!(frontier.fast_travel_index.is_none());
    assert!(frontier.fast_travel_steps.is_none());
}

#[test]
fn retained_travel_history_recovers_legacy_visited_locations() {
    let mut state = test_state();
    state
        .world
        .record_history(3, "Tester the Wanderer traveled to Old Shrine.");
    state.world.discovered_route_ids = vec![
        "route.compat.legacy.10.11".into(),
        "route.compat.legacy.11.13".into(),
    ];

    let view = build_view(&state);
    let old_shrine = view
        .nodes
        .iter()
        .find(|node| node.location.id == 11)
        .expect("the visited location should be visible");

    assert!(old_shrine.explored);
    assert_eq!(
        view.fast_travel_destinations
            .iter()
            .map(|destination| destination.location.id)
            .collect::<Vec<_>>(),
        vec![11]
    );
    assert!(view.nodes.iter().any(|node| node.location.id == 13));
}

#[test]
fn retained_travel_history_recovers_visits_across_multiple_inheritance_generations() {
    let mut state = test_state();
    state.world.history.clear();
    state
        .world
        .record_history(0, "First Heir the Wanderer entered the world.");
    state
        .world
        .record_history(3, "First Heir the Wanderer traveled to Old Shrine.");
    state
        .world
        .record_history(10, "Second Heir the Warden inherited the world.");
    state
        .world
        .record_history(15, "Second Heir the Warden traveled to Sunken Watch.");
    state
        .world
        .record_history(20, "Tester the Wanderer inherited the world.");
    state.world.discovered_route_ids = vec![
        "route.compat.legacy.10.11".into(),
        "route.compat.legacy.11.13".into(),
    ];

    let view = build_view(&state);
    let old_shrine = view
        .nodes
        .iter()
        .find(|node| node.location.id == 11)
        .expect("the first-generation visit should remain visible");
    let sunken_watch = view
        .nodes
        .iter()
        .find(|node| node.location.id == 13)
        .expect("the second-generation visit should remain visible");

    assert!(old_shrine.explored);
    assert!(sunken_watch.explored);
    assert_eq!(
        view.fast_travel_destinations
            .iter()
            .map(|destination| (destination.location.id, destination.route_steps))
            .collect::<Vec<_>>(),
        vec![(11, 1), (13, 2)]
    );
}

#[test]
fn unrelated_travel_history_does_not_mark_a_location_as_visited() {
    let mut state = test_state();
    state
        .world
        .record_history(3, "A band of hunters traveled to Old Shrine.");
    state.world.discovered_route_ids.push("route.compat.legacy.10.11".into());

    let view = build_view(&state);
    let old_shrine = view
        .nodes
        .iter()
        .find(|node| node.location.id == 11)
        .expect("the frontier location should remain visible");

    assert!(!old_shrine.explored);
    assert!(view.fast_travel_destinations.is_empty());
}

#[test]
fn map_routes_are_derived_only_from_established_graph_edges() {
    let mut state = test_state();
    state.world.local_maps.push(explored_map(11));
    state.world.discovered_route_ids = vec![
        "route.compat.legacy.10.11".into(),
        "route.compat.legacy.10.12".into(),
        "route.compat.legacy.11.13".into(),
    ];

    let view = build_view(&state);
    let mut routes = view
        .routes
        .iter()
        .map(|route| (route.from_id, route.to_id, route.bidirectional))
        .collect::<Vec<_>>();
    routes.sort_unstable();

    assert_eq!(routes, vec![(10, 11, true), (10, 12, true), (11, 13, true)]);
    assert_eq!(
        view.nodes
            .iter()
            .find(|node| node.location.id == 13)
            .unwrap()
            .location
            .region_name,
        "Ashen Vale"
    );
}

#[test]
fn map_preserves_explicit_one_way_route_direction() {
    let mut state = test_state();
    state.world.remove_routes_between(11, 13);
    assert!(state.world.add_route(WorldRoute::new(
        "route.test.one_way",
        11,
        13,
        RouteDirection::AtoB,
    )));
    state.world.local_maps.push(explored_map(11));
    state.world.mark_route_discovered("route.test.one_way");

    let view = build_view(&state);
    let route = view
        .routes
        .iter()
        .find(|route| route.from_id == 11 && route.to_id == 13)
        .expect("the one-way route should be present");

    assert!(!route.bidirectional);
}
