use super::*;
use crate::content::CharacterPartContent;

fn part(id: &str, slot: &str, layer: i32) -> CharacterPartContent {
    CharacterPartContent {
        id: id.to_string(),
        slot: slot.to_string(),
        asset: format!("portraits/{id}.png"),
        weight: 1,
        layer,
        requires_tags: Vec::new(),
        excludes_tags: Vec::new(),
    }
}

#[test]
fn slot_catalogue_groups_parts_and_keeps_a_stable_order() {
    let slots = build_slots(vec![
        part("hair-z", "hair", 3),
        part("face", "face", 2),
        part("hair-a", "hair", 3),
    ]);

    assert_eq!(
        slots
            .iter()
            .map(|slot| slot.name.as_str())
            .collect::<Vec<_>>(),
        vec!["hair", "face"]
    );
    assert_eq!(
        slots[0]
            .parts
            .iter()
            .map(|part| part.id.as_str())
            .collect::<Vec<_>>(),
        vec!["hair-a", "hair-z"]
    );
}

#[test]
fn preview_uses_visible_selected_parts_in_draw_order() {
    let mut slots = build_slots(vec![
        part("body", "body", 3),
        part("face", "face", 1),
        part("hair", "hair", 2),
    ]);

    let portrait = build_preview_portrait(&slots);
    assert_eq!(
        portrait
            .layers
            .iter()
            .map(|layer| layer.id.as_str())
            .collect::<Vec<_>>(),
        vec!["face", "hair", "body"]
    );

    slots
        .iter_mut()
        .find(|slot| slot.name == "body")
        .expect("the body slot should exist")
        .visible = false;
    let portrait = build_preview_portrait(&slots);
    assert_eq!(
        portrait
            .layers
            .iter()
            .map(|layer| layer.id.as_str())
            .collect::<Vec<_>>(),
        vec!["face", "hair"]
    );
}

#[test]
fn slot_and_part_cycling_wraps_and_handles_empty_catalogues() {
    assert_eq!(cycle_index(0, 3, -1), 2);
    assert_eq!(cycle_index(2, 3, 1), 0);
    assert_eq!(cycle_index(0, 0, 1), 0);
    assert_eq!(cycle_index(8, 3, 0), 2);
}

#[test]
fn optional_character_lab_slots_cycle_through_none_and_back() {
    let mut state = DeveloperToolsState {
        slots: build_slots(vec![
            part("body.one", "body", 10),
            part("headgear.hood", "headgear", 20),
        ]),
        ..Default::default()
    };
    let headgear = state
        .slots
        .iter()
        .position(|slot| slot.name == "headgear")
        .unwrap();
    let headgear_count = state.slots[headgear].parts.len();

    cycle_slot_part(&mut state, headgear, -1);
    assert_eq!(state.slots[headgear].selected_part, headgear_count);
    assert!(build_preview_portrait(&state.slots)
        .layers
        .iter()
        .all(|layer| layer.slot != "headgear"));

    cycle_slot_part(&mut state, headgear, 1);
    assert_eq!(state.slots[headgear].selected_part, 0);
    assert!(build_preview_portrait(&state.slots)
        .layers
        .iter()
        .any(|layer| layer.slot == "headgear"));
}

#[test]
fn character_lab_randomizer_can_omit_optional_slots() {
    let mut state = DeveloperToolsState {
        slots: build_slots(vec![
            part("body.one", "body", 10),
            part("headgear.hood", "headgear", 20),
        ]),
        ..Default::default()
    };
    let headgear = state
        .slots
        .iter()
        .position(|slot| slot.name == "headgear")
        .unwrap();
    let body = state
        .slots
        .iter()
        .position(|slot| slot.name == "body")
        .unwrap();
    let mut omitted = 0;

    for _ in 0..256 {
        randomize_combination(&mut state);
        assert_eq!(state.slots[body].selected_part, 0);
        if state.slots[headgear].selected_part == state.slots[headgear].parts.len() {
            omitted += 1;
        }
    }

    assert!(omitted > 0 && omitted < 256);
}

#[test]
fn world_lab_generation_is_reproducible_and_respects_selected_layout() {
    let options = WorldLabOptions {
        seed: 7_654_321,
        size_index: 2,
        location_index: 0,
        shrine_layout: true,
    };

    let first = generate_world_lab_map(options);
    let repeated = generate_world_lab_map(options);

    assert_eq!(first, repeated);
    assert_eq!((first.width, first.height), (40, 24));
    assert_eq!(first.layout_id.as_deref(), Some("shrine_chamber"));

    let different_seed = generate_world_lab_map(WorldLabOptions {
        seed: options.seed + 1,
        ..options
    });
    assert_ne!(first, different_seed);
}

#[test]
fn world_lab_preview_matches_generated_map_dimensions() {
    let options = WorldLabOptions {
        seed: 19,
        size_index: 1,
        location_index: 0,
        shrine_layout: false,
    };
    let map = generate_world_lab_map(options);
    let preview = world_lab_map_preview(&map);
    let rows = preview.lines().collect::<Vec<_>>();

    assert_eq!(rows.len(), usize::from(map.height));
    assert!(rows
        .iter()
        .all(|row| row.chars().count() == usize::from(map.width)));
    assert!(rows.iter().any(|row| row.contains('@')));
    assert!(rows.iter().any(|row| row.contains('?')));
    assert!(rows.iter().any(|row| row.contains('#')));
}

#[test]
fn world_lab_editor_preview_marks_cursor_and_held_prop() {
    let map = generate_world_lab_map(WorldLabOptions::default());
    let prop = map
        .props
        .first()
        .expect("generated map should include a prop");
    let held_id = prop.id.clone();
    let held_position = prop.position;
    let cursor = GridPosition { x: 0, y: 0 };
    let preview = world_lab_map_preview_with_cursor(&map, Some(cursor), Some(&held_id), None);
    let rows = preview.lines().collect::<Vec<_>>();
    assert_eq!(
        rows[usize::from(cursor.y)]
            .chars()
            .nth(usize::from(cursor.x)),
        Some('X')
    );
    assert_eq!(
        rows[usize::from(held_position.y)]
            .chars()
            .nth(usize::from(held_position.x)),
        Some('H')
    );

    let point = map
        .points_of_interest
        .first()
        .expect("map should include a discovery");
    let held_point_preview = world_lab_map_preview_with_cursor(&map, None, None, Some(&point.id));
    let held_point_rows = held_point_preview.lines().collect::<Vec<_>>();
    assert_eq!(
        held_point_rows[usize::from(point.position.y)]
            .chars()
            .nth(usize::from(point.position.x)),
        Some('Q')
    );
}

#[test]
fn world_lab_cursor_movement_stays_inside_the_current_map() {
    let mut spatial = SpatialExplorationState::default();
    let map = generate_world_lab_map(WorldLabOptions::default());
    let lower_right = GridPosition {
        x: map.width - 1,
        y: map.height - 1,
    };
    bevy_spatial::set_sandbox_map(&mut spatial, map);
    let mut state = DeveloperToolsState {
        world_lab_cursor: Some(lower_right),
        ..Default::default()
    };
    move_world_lab_cursor(&mut state, &spatial, 1, 1);
    assert_eq!(state.world_lab_cursor, Some(lower_right));
    move_world_lab_cursor(&mut state, &spatial, -1, -1);
    assert_eq!(
        state.world_lab_cursor,
        Some(GridPosition {
            x: lower_right.x - 1,
            y: lower_right.y - 1
        })
    );
}

#[test]
fn world_lab_point_of_interest_tool_can_place_move_and_remove_a_discovery() {
    let mut spatial = SpatialExplorationState::default();
    let map = generate_world_lab_map(WorldLabOptions::default());
    let find_free_cell = |map: &LocalMap, excluded: Option<GridPosition>| {
        (1..map.height - 1)
            .flat_map(|y| (1..map.width - 1).map(move |x| GridPosition { x, y }))
            .find(|position| {
                Some(*position) != excluded
                    && map.is_walkable(*position)
                    && *position != map.entry
                    && !map
                        .points_of_interest
                        .iter()
                        .any(|point| point.position == *position)
                    && !map.exits.iter().any(|exit| exit.position == *position)
                    && !map
                        .interactables
                        .iter()
                        .any(|actor| actor.position == *position)
            })
    };
    let first_position = find_free_cell(&map, None).expect("map should have a free editor cell");
    let mut state = DeveloperToolsState {
        world_lab_cursor: Some(first_position),
        world_lab_editor_tool: WorldLabEditorTool::PointOfInterest,
        ..Default::default()
    };
    bevy_spatial::set_sandbox_map(&mut spatial, map);

    place_world_lab_object(&mut state, &mut spatial);
    let created = bevy_spatial::sandbox_map(&spatial)
        .and_then(|map| {
            map.points_of_interest
                .iter()
                .find(|point| point.id.contains(".editor_poi."))
                .cloned()
        })
        .expect("editor should place a temporary discovery");
    assert_eq!(created.position, first_position);
    assert!(created.title.starts_with("World Lab Discovery "));
    assert!(!created.discovered);
    assert!(!created.interacted);

    pick_up_or_drop_world_lab_object(&mut state, &mut spatial);
    assert_eq!(
        state.world_lab_held_point_of_interest.as_deref(),
        Some(created.id.as_str())
    );

    let map_after_pickup = bevy_spatial::sandbox_map(&spatial).unwrap().clone();
    let second_position = find_free_cell(&map_after_pickup, Some(first_position))
        .expect("map should have a second free editor cell");
    state.world_lab_cursor = Some(second_position);
    pick_up_or_drop_world_lab_object(&mut state, &mut spatial);
    assert!(state.world_lab_held_point_of_interest.is_none());
    let moved = bevy_spatial::sandbox_map(&spatial)
        .and_then(|map| map.point_of_interest(&created.id))
        .expect("discovery should still exist after moving");
    assert_eq!(moved.position, second_position);

    remove_world_lab_object(&mut state, &mut spatial);
    assert!(bevy_spatial::sandbox_map(&spatial)
        .is_some_and(|map| map.point_of_interest(&created.id).is_none()));
}

#[test]
fn world_lab_session_uses_real_world_graph_and_reproducible_actor_ids() {
    let game = generate_world_lab_game(44_321);
    let npc_location_index = game
        .npcs
        .first()
        .and_then(|npc| {
            game.world
                .locations
                .iter()
                .position(|location| location.id == npc.location_id)
        })
        .expect("generated world should include an NPC at a real location");
    let options = WorldLabOptions {
        seed: 44_321,
        size_index: 1,
        location_index: npc_location_index,
        shrine_layout: false,
    };

    let (map, scenario) = generate_world_lab_session(options);
    let source = scenario
        .world
        .location_by_id(scenario.location_id)
        .expect("selected location should exist in the scenario graph");
    assert_eq!(map.location_id, format!("location.{}", source.id));
    assert_eq!(
        map.exits
            .iter()
            .map(|exit| exit.target_location_id)
            .collect::<Vec<_>>(),
        source.exits
    );
    assert!(!scenario.actors.is_empty());
    assert!(map.interactables.iter().all(|placement| scenario
        .actors
        .iter()
        .any(|actor| actor.entity_id == placement.entity_id && actor.kind == placement.kind)));

    let restored =
        restore_world_lab_scenario(&map).expect("generated fixture should restore its world graph");
    assert_eq!(restored.location_id, scenario.location_id);
    assert_eq!(
        restored
            .actors
            .iter()
            .map(|actor| actor.entity_id)
            .collect::<Vec<_>>(),
        scenario
            .actors
            .iter()
            .map(|actor| actor.entity_id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn world_lab_passage_editor_updates_reciprocal_graph_links() {
    let options = WorldLabOptions {
        seed: 887_001,
        ..WorldLabOptions::default()
    };
    let (map, scenario) = generate_world_lab_session(options);
    let mut spatial = SpatialExplorationState::default();
    bevy_spatial::set_sandbox_map(&mut spatial, map.clone());
    let mut state = DeveloperToolsState {
        world_lab: options,
        world_lab_scenario: Some(scenario),
        world_lab_cursor: Some(map.entry),
        world_lab_editor_tool: WorldLabEditorTool::Passage,
        ..Default::default()
    };
    let (destination_id, _) = selected_world_lab_destination(&state)
        .expect("selected location should have at least one unconnected destination");
    let passage_id = format!("{}.exit.{}", map.location_id, destination_id);
    let edge_positions = (1..map.height - 1)
        .flat_map(|y| {
            [
                GridPosition {
                    x: map.width - 1,
                    y,
                },
                GridPosition { x: 0, y },
            ]
        })
        .chain((1..map.width - 1).flat_map(|x| {
            [
                GridPosition { x, y: 0 },
                GridPosition {
                    x,
                    y: map.height - 1,
                },
            ]
        }))
        .collect::<Vec<_>>();
    let placement = edge_positions
        .iter()
        .copied()
        .find(|position| {
            let mut candidate = map.clone();
            candidate.place_map_exit(
                MapExit {
                    id: passage_id.clone(),
                    position: *position,
                    target_location_id: destination_id,
                },
                &[destination_id],
            )
        })
        .expect("an unoccupied edge should accept a new route");
    state.world_lab_cursor = Some(placement);

    place_world_lab_object(&mut state, &mut spatial);
    assert!(bevy_spatial::sandbox_map(&spatial)
        .is_some_and(|map| { map.exits.iter().any(|exit| exit.id == passage_id) }));
    let scenario = state.world_lab_scenario.as_ref().unwrap();
    assert!(scenario
        .world
        .location_by_id(scenario.location_id)
        .unwrap()
        .exits
        .contains(&destination_id));
    assert!(scenario
        .world
        .location_by_id(destination_id)
        .unwrap()
        .exits
        .contains(&scenario.location_id));

    remove_world_lab_object(&mut state, &mut spatial);
    let scenario = state.world_lab_scenario.as_ref().unwrap();
    assert!(!scenario
        .world
        .location_by_id(scenario.location_id)
        .unwrap()
        .exits
        .contains(&destination_id));
    assert!(!scenario
        .world
        .location_by_id(destination_id)
        .unwrap()
        .exits
        .contains(&scenario.location_id));
}

#[test]
fn world_lab_actor_editor_moves_removes_and_reuses_a_real_entity() {
    let game = generate_world_lab_game(19_992);
    let location_index = game
        .npcs
        .first()
        .and_then(|npc| {
            game.world
                .locations
                .iter()
                .position(|location| location.id == npc.location_id)
        })
        .expect("generated world should include NPCs");
    let options = WorldLabOptions {
        seed: 19_992,
        location_index,
        ..WorldLabOptions::default()
    };
    let (map, scenario) = generate_world_lab_session(options);
    let actor = scenario
        .actors
        .first()
        .expect("selected location should have an actor")
        .clone();
    let actor_position = map
        .interactables
        .iter()
        .find(|placement| placement.entity_id == actor.entity_id)
        .map(|placement| placement.position)
        .expect("real actor should have a deterministic map placement");
    let find_free_cell = |map: &LocalMap| {
        (1..map.height - 1)
            .flat_map(|y| (1..map.width - 1).map(move |x| GridPosition { x, y }))
            .find(|position| {
                map.is_walkable(*position)
                    && *position != map.entry
                    && !map
                        .points_of_interest
                        .iter()
                        .any(|point| point.position == *position)
                    && !map.exits.iter().any(|exit| exit.position == *position)
                    && !map
                        .interactables
                        .iter()
                        .any(|placed| placed.position == *position)
            })
    };
    let mut spatial = SpatialExplorationState::default();
    bevy_spatial::set_sandbox_map(&mut spatial, map.clone());
    let mut state = DeveloperToolsState {
        world_lab: options,
        world_lab_scenario: Some(scenario),
        world_lab_editor_tool: WorldLabEditorTool::Actor,
        world_lab_actor_index: 0,
        world_lab_cursor: Some(actor_position),
        ..Default::default()
    };

    pick_up_or_drop_world_lab_object(&mut state, &mut spatial);
    assert_eq!(
        state.world_lab_held_actor,
        Some((actor.entity_id, actor.kind))
    );
    let destination = find_free_cell(&map).expect("map should contain a second actor position");
    state.world_lab_cursor = Some(destination);
    pick_up_or_drop_world_lab_object(&mut state, &mut spatial);
    assert!(state.world_lab_held_actor.is_none());
    assert!(bevy_spatial::sandbox_map(&spatial).is_some_and(|map| map
        .interactables
        .iter()
        .any(|placed| placed.entity_id == actor.entity_id && placed.position == destination)));

    remove_world_lab_object(&mut state, &mut spatial);
    assert!(bevy_spatial::sandbox_map(&spatial).is_some_and(|map| {
        !map.interactables
            .iter()
            .any(|placed| placed.entity_id == actor.entity_id)
    }));
    place_world_lab_object(&mut state, &mut spatial);
    assert!(bevy_spatial::sandbox_map(&spatial).is_some_and(|map| {
        map.interactables
            .iter()
            .any(|placed| placed.entity_id == actor.entity_id && placed.position == destination)
    }));
}

#[test]
fn character_lab_has_two_part_controls_per_slot_before_its_global_actions() {
    let slot_count = 5;

    assert_eq!(character_lab_action_count(slot_count), 14);
    for selected in 0..slot_count * 2 {
        assert_eq!(
            character_lab_control_target(selected, slot_count),
            Some((selected / 2, if selected % 2 == 0 { -1 } else { 1 }))
        );
    }
    assert_eq!(
        character_lab_control_target(slot_count * 2, slot_count),
        None
    );
    assert_eq!(
        character_lab_control_target(slot_count * 2 + 3, slot_count),
        None
    );
}

#[test]
fn cycling_a_character_lab_slot_part_also_selects_that_slot() {
    fn part(id: &str, slot: &str) -> CharacterPartContent {
        CharacterPartContent {
            id: id.into(),
            slot: slot.into(),
            asset: format!("{id}.png"),
            weight: 1,
            layer: 1,
            requires_tags: Vec::new(),
            excludes_tags: Vec::new(),
        }
    }

    let mut state = DeveloperToolsState {
        slots: vec![
            CharacterLabSlot {
                name: "body".into(),
                parts: vec![part("body.one", "body"), part("body.two", "body")],
                selected_part: 0,
                visible: true,
            },
            CharacterLabSlot {
                name: "face".into(),
                parts: vec![part("face.one", "face"), part("face.two", "face")],
                selected_part: 0,
                visible: true,
            },
        ],
        selected_slot: 0,
        ..default()
    };

    cycle_slot_part(&mut state, 1, -1);

    assert_eq!(state.selected_slot, 1);
    assert_eq!(state.slots[0].selected_part, 0);
    assert_eq!(state.slots[1].selected_part, 1);
    assert!(state.dirty);
}

#[test]
fn developer_tools_screen_only_rebuilds_when_its_own_view_changes() {
    assert!(!developer_tools_screen_requires_render(
        Some(ScreenId::DeveloperTools),
        ScreenId::DeveloperTools,
        false,
    ));
    assert!(!developer_tools_screen_requires_render(
        Some(ScreenId::WorldLab),
        ScreenId::WorldLab,
        false,
    ));
    assert!(developer_tools_screen_requires_render(
        Some(ScreenId::DeveloperTools),
        ScreenId::DeveloperTools,
        true,
    ));
    assert!(developer_tools_screen_requires_render(
        Some(ScreenId::DeveloperTools),
        ScreenId::WorldLab,
        false,
    ));
}

#[test]
fn character_lab_slots_follow_the_portrait_view_order() {
    let parts = ["body", "face", "headgear", "accessory", "hair"]
        .into_iter()
        .map(|slot| CharacterPartContent {
            id: format!("{slot}.default"),
            slot: slot.into(),
            asset: format!("{slot}.png"),
            weight: 1,
            layer: 1,
            requires_tags: Vec::new(),
            excludes_tags: Vec::new(),
        })
        .collect();

    let slots = build_slots(parts);
    let order = slots
        .iter()
        .map(|slot| slot.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(order, vec!["headgear", "hair", "face", "accessory", "body"]);
}
