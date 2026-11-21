use super::*;

#[test]
fn same_seed_and_location_generate_identical_maps() {
    let a = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
    let b = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
    assert_eq!(a, b);
}

#[test]
fn different_location_ids_generate_distinct_maps() {
    let a = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
    let b = generate_local_map(42, "location.ashen_gate", LocalMapConfig::default());
    assert_ne!(a.seed, b.seed);
    assert_ne!(a, b);
}

#[test]
fn generated_props_are_deterministic_and_block_their_footprints() {
    let a = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
    let b = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
    assert_eq!(a.props, b.props);
    assert!(!a.props.is_empty());
    assert!(a
        .props
        .iter()
        .any(|prop| prop.kind == SpatialPropKind::Pillar));
    assert!(a
        .props
        .iter()
        .any(|prop| prop.kind == SpatialPropKind::Table));

    for prop in &a.props {
        for offset_x in 0..prop.kind.footprint_width() {
            let occupied = GridPosition {
                x: prop.position.x + offset_x,
                y: prop.position.y,
            };
            assert!(
                !a.is_walkable(occupied),
                "prop footprint must block {occupied:?}"
            );
        }
    }
    assert!(has_walkable_path(
        &a,
        a.entry,
        a.points_of_interest[0].position
    ));
}

#[test]
fn props_round_trip_and_optional_map_fields_default_when_absent() {
    let map = generate_local_map(12, "location.old_shrine", LocalMapConfig::default());
    let json = serde_json::to_string(&map).expect("map with props should serialize");
    let restored: LocalMap =
        serde_json::from_str(&json).expect("map with props should deserialize");
    assert_eq!(restored.props, map.props);

    let mut old_value = serde_json::to_value(&map).expect("map should serialize");
    let object = old_value.as_object_mut().expect("map should be an object");
    object.remove("props");
    object.remove("interactables");
    object.remove("encounters");
    object.remove("exits");
    object.remove("layout_id");
    object.remove("rooms");
    object.remove("doors");
    let restored_old: LocalMap = serde_json::from_value(old_value)
        .expect("map with omitted optional fields should deserialize");
    assert!(restored_old.props.is_empty());
    assert!(restored_old.interactables.is_empty());
    assert!(restored_old.encounters.is_empty());
    assert!(restored_old.exits.is_empty());
    assert!(restored_old.layout_id.is_none());
    assert!(restored_old.rooms.is_empty());
    assert!(restored_old.doors.is_empty());
}

#[test]
fn pre_topology_local_maps_default_to_empty_rooms_and_doors() {
    let map = generate_local_map(52, "location.legacy_save", LocalMapConfig::default());
    let mut value = serde_json::to_value(&map).expect("map should serialize");
    let object = value.as_object_mut().expect("map should be an object");
    object.remove("rooms");
    object.remove("doors");

    let restored: LocalMap = serde_json::from_value(value)
        .expect("a pre-topology save should load without room and door fields");

    assert!(restored.rooms.is_empty());
    assert!(restored.doors.is_empty());
    assert_eq!(restored.terrain, map.terrain);
    assert_eq!(restored.entry, map.entry);
    assert_eq!(restored.location_id, map.location_id);
}

#[test]
fn generated_map_has_valid_bounds_and_reachable_point_of_interest() {
    for seed in 0..64 {
        let map = generate_local_map(seed, "location.old_shrine", LocalMapConfig::default());
        assert_eq!(
            map.terrain.len(),
            usize::from(map.width) * usize::from(map.height)
        );
        assert!(map.contains(map.entry));
        assert!(map.is_walkable(map.entry));
        let poi = &map.points_of_interest[0];
        assert!(map.contains(poi.position));
        assert!(map.is_walkable(poi.position));
        assert!(has_walkable_path(&map, map.entry, poi.position));
    }
}

#[test]
fn authored_shrine_chamber_has_a_clear_doorway_and_deterministic_props() {
    let map = generate_local_map_with_layout(
        42,
        "location.old_shrine",
        LocalMapConfig::default(),
        Some("shrine_chamber"),
    );
    let repeated = generate_local_map_with_layout(
        42,
        "location.old_shrine",
        LocalMapConfig::default(),
        Some("shrine_chamber"),
    );
    assert_eq!(map, repeated);

    // The room has an enclosing wall ring with one explicit opening.
    assert_eq!(
        map.terrain_at(GridPosition { x: 2, y: 2 }),
        Some(Terrain::Wall)
    );
    assert_eq!(
        map.terrain_at(GridPosition { x: 3, y: 2 }),
        Some(Terrain::Wall)
    );
    assert_eq!(
        map.terrain_at(GridPosition { x: 3, y: 4 }),
        Some(Terrain::Floor)
    );
    assert_eq!(
        map.terrain_at(GridPosition { x: 5, y: 7 }),
        Some(Terrain::Floor)
    );
    assert_eq!(
        map.terrain_at(GridPosition { x: 4, y: 7 }),
        Some(Terrain::Wall)
    );

    let pillar = map
        .props
        .iter()
        .find(|prop| prop.id.ends_with(".room.pillar"))
        .unwrap();
    let table = map
        .props
        .iter()
        .find(|prop| prop.id.ends_with(".room.table"))
        .unwrap();
    assert_eq!(pillar.position, GridPosition { x: 3, y: 5 });
    assert_eq!(table.position, GridPosition { x: 5, y: 5 });
    assert!(!map.is_walkable(pillar.position));
    assert!(!map.is_walkable(table.position));
    assert!(!map.is_walkable(GridPosition { x: 6, y: 5 }));

    assert!(has_walkable_path(
        &map,
        map.entry,
        map.points_of_interest[0].position
    ));
    assert!(has_walkable_path(
        &map,
        GridPosition { x: 5, y: 8 },
        GridPosition { x: 4, y: 5 }
    ));
}

#[test]
fn authored_gate_and_market_layouts_preserve_room_thresholds_and_exits() {
    for (location_id, layout, barrier_x, targets) in [
        (
            "location.ashen_gate",
            "ashen_gate_causeway",
            12,
            vec![101, 102],
        ),
        (
            "location.hollow_market",
            "hollow_market_square",
            4,
            vec![201, 202, 203],
        ),
    ] {
        let mut map = generate_local_map_with_layout(
            84,
            location_id,
            LocalMapConfig::default(),
            Some(layout),
        );
        let repeated = generate_local_map_with_layout(
            84,
            location_id,
            LocalMapConfig::default(),
            Some(layout),
        );

        assert_eq!(map, repeated, "{layout} must be deterministic");
        assert_eq!(map.layout_id.as_deref(), Some(layout));
        assert_eq!(map.rooms.len(), 2);
        assert_eq!(map.doors.len(), 1);
        assert_eq!(map.doors[0].state, DoorState::Open);
        assert!(map.validate_room_topology().is_ok());

        let door_id = map.doors[0].id.clone();
        let west_threshold = GridPosition {
            x: barrier_x - 1,
            y: map.entry.y,
        };
        let east_threshold = GridPosition {
            x: barrier_x,
            y: map.entry.y,
        };
        assert!(map.can_cross_edge(west_threshold, east_threshold));

        configure_map_exits(&mut map, &targets);
        assert!(map.validate_room_topology().is_ok());
        if layout == "hollow_market_square" {
            let shrine_exit = map
                .exits
                .iter()
                .find(|exit| exit.target_location_id == 201)
                .expect("first authored connection leads to Old Shrine");
            let gate_exit = map
                .exits
                .iter()
                .find(|exit| exit.target_location_id == 202)
                .expect("second authored connection leads back to Ashen Gate");
            let crossing_exit = map
                .exits
                .iter()
                .find(|exit| exit.target_location_id == 203)
                .expect("third authored connection leads to Sootbound Crossing");
            assert_eq!(shrine_exit.position.x, map.width - 1);
            assert_eq!(gate_exit.position.x, 0);
            assert_eq!(crossing_exit.position.y, 0);
        }
        for exit in &map.exits {
            assert!(
                has_walkable_path(&map, map.entry, exit.position),
                "{layout} exit {} must remain reachable through its authored threshold",
                exit.id
            );
        }

        let far_side = GridPosition {
            x: barrier_x + 3,
            y: map.entry.y,
        };
        assert!(has_walkable_path(&map, map.entry, far_side));
        assert!(map.set_door_state(&door_id, DoorState::Closed));
        assert!(
            !has_walkable_path(&map, map.entry, far_side),
            "{layout} threshold must be the only route between its two rooms"
        );
    }
}

#[test]
fn authored_room_does_not_block_generated_edge_passages() {
    for target_count in [2, 3, 4, 6, 8] {
        let mut map = generate_local_map_with_layout(
            42,
            "location.old_shrine",
            LocalMapConfig::default(),
            Some("shrine_chamber"),
        );
        let targets = (0..target_count)
            .map(|index| 100 + index as u64)
            .collect::<Vec<_>>();
        configure_map_exits(&mut map, &targets);
        assert!(has_walkable_path(
            &map,
            map.entry,
            map.points_of_interest[0].position
        ));
        assert_eq!(
            map.terrain_at(GridPosition { x: 2, y: 5 }),
            Some(Terrain::Wall)
        );
        assert_eq!(
            map.terrain_at(GridPosition { x: 7, y: 5 }),
            Some(Terrain::Wall)
        );
        assert!(map
            .props
            .iter()
            .any(|prop| prop.id.ends_with(".room.pillar")));
        assert!(map
            .props
            .iter()
            .any(|prop| prop.id.ends_with(".room.table")));
        for exit in &map.exits {
            assert!(has_walkable_path(&map, map.entry, exit.position));
        }
    }
}

#[test]
fn authored_gate_and_market_layouts_fall_back_when_map_is_too_short() {
    for layout in ["ashen_gate_causeway", "hollow_market_square"] {
        let plain = generate_local_map(
            42,
            "location.layout_size_test",
            LocalMapConfig {
                width: DEFAULT_MAP_WIDTH,
                height: 12,
            },
        );
        let undersized = generate_local_map_with_layout(
            42,
            "location.layout_size_test",
            LocalMapConfig {
                width: DEFAULT_MAP_WIDTH,
                height: 12,
            },
            Some(layout),
        );
        assert_eq!(undersized, plain, "{layout} needs the standard map height");
    }
}

#[test]
fn unknown_spatial_layout_falls_back_to_procedural_generation() {
    let plain = generate_local_map(42, "location.test", LocalMapConfig::default());
    let unknown = generate_local_map_with_layout(
        42,
        "location.test",
        LocalMapConfig::default(),
        Some("modded_layout_not_registered"),
    );
    assert_eq!(plain, unknown);
}

#[test]
fn dimensions_are_normalized_and_map_is_bounded() {
    let map = generate_local_map(
        1,
        "location.test",
        LocalMapConfig {
            width: 1,
            height: u16::MAX,
        },
    );
    assert_eq!(map.width, MIN_MAP_DIMENSION);
    assert_eq!(map.height, MAX_MAP_DIMENSION);
    assert_eq!(
        map.terrain.len(),
        usize::from(map.width) * usize::from(map.height)
    );
}

#[test]
fn connected_world_locations_get_reachable_edge_passages() {
    let mut map = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
    configure_map_exits(&mut map, &[11, 22, 33, 44, 22]);

    assert_eq!(map.exits.len(), 4);
    assert_eq!(
        map.exits
            .iter()
            .map(|exit| exit.target_location_id)
            .collect::<Vec<_>>(),
        vec![11, 22, 33, 44]
    );
    for exit in &map.exits {
        assert!(
            exit.position.x == 0
                || exit.position.x == map.width - 1
                || exit.position.y == 0
                || exit.position.y == map.height - 1,
            "passage must be on a map edge"
        );
        assert!(map.is_walkable(exit.position));
        assert!(has_walkable_path(&map, map.entry, exit.position));
    }

    let repeated = {
        let mut repeated = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
        configure_map_exits(&mut repeated, &[11, 22, 33, 44, 22]);
        repeated
    };
    assert_eq!(map.exits, repeated.exits);
    assert_eq!(map.terrain, repeated.terrain);
}

#[test]
fn generated_passages_remain_reachable_and_persistent_across_seeds() {
    for seed in 0..32 {
        for target_count in [1, 2, 3, 4, 6] {
            let targets = (0..target_count)
                .map(|index| 100 + index as u64)
                .collect::<Vec<_>>();
            let mut map = generate_local_map(
                seed,
                "location.spatial_exit_test",
                LocalMapConfig::default(),
            );
            configure_map_exits(&mut map, &targets);

            assert_eq!(map.exits.len(), target_count);
            assert!(has_walkable_path(
                &map,
                map.entry,
                map.points_of_interest[0].position
            ));
            for exit in &map.exits {
                assert!(has_walkable_path(&map, map.entry, exit.position));
                assert!(
                    !map.props
                        .iter()
                        .any(|prop| { prop.footprint_contains(exit.position, map.width) }),
                    "an exit corridor must clear blocking prop footprints"
                );
            }

            let json = serde_json::to_string(&map).expect("map with passages should serialize");
            let restored: LocalMap =
                serde_json::from_str(&json).expect("map with passages should deserialize");
            assert_eq!(restored.exits, map.exits);
            assert_eq!(restored.terrain, map.terrain);
        }
    }
}

#[test]
fn map_and_session_round_trip_through_serde() {
    let mut map = generate_local_map(123, "location.old_shrine", LocalMapConfig::default());
    let poi_id = map.points_of_interest[0].id.clone();
    map.point_of_interest_mut(&poi_id).unwrap().discovered = true;
    map.mark_tile_discovered(map.entry);
    map.set_terrain(map.entry, Terrain::Rubble);
    let json = serde_json::to_string(&map).expect("map should serialize");
    let restored: LocalMap = serde_json::from_str(&json).expect("map should deserialize");
    assert_eq!(restored, map);

    let session = ExplorationSession {
        location_id: map.location_id.clone(),
        position: map.entry,
    };
    let json = serde_json::to_string(&session).expect("session should serialize");
    let restored: ExplorationSession =
        serde_json::from_str(&json).expect("session should deserialize");
    assert_eq!(restored, session);
}

#[test]
fn spatial_entity_placements_are_stable_unique_and_backward_compatible() {
    let mut map = generate_local_map(82, "location.actor_test", LocalMapConfig::default());
    let requested = vec![
        (12, SpatialEntityKind::Npc),
        (14, SpatialEntityKind::Corpse),
        (10, SpatialEntityKind::Npc),
    ];
    ensure_spatial_entity_placements(&mut map, &requested);
    let first = map.interactables.clone();
    ensure_spatial_entity_placements(&mut map, &requested);

    assert_eq!(map.interactables, first);
    assert_eq!(map.interactables.len(), requested.len());
    for (index, placement) in map.interactables.iter().enumerate() {
        assert!(map.is_walkable(placement.position));
        assert!(reachable_walkable_tiles(&map).contains(&placement.position));
        assert_ne!(placement.position, map.entry);
        assert!(!map
            .points_of_interest
            .iter()
            .any(|poi| poi.position == placement.position));
        assert!(!map
            .exits
            .iter()
            .any(|exit| exit.position == placement.position));
        assert!(!map.interactables[index + 1..]
            .iter()
            .any(|other| other.position == placement.position));
    }

    let json = serde_json::to_string(&map).expect("map with actor placements should serialize");
    let restored: LocalMap =
        serde_json::from_str(&json).expect("map with actor placements should deserialize");
    assert_eq!(restored.interactables, map.interactables);
}

#[test]
fn spatial_encounters_are_deterministic_persistent_and_backward_compatible() {
    let mut map = generate_local_map(84, "location.encounter_test", LocalMapConfig::default());
    let created = ensure_spatial_encounter(
        &mut map,
        "encounter.test.1".into(),
        "Ash Marauder".into(),
        "A hostile figure watches the road.".into(),
        18,
        3,
        "Marauder Token".into(),
    );
    assert!(created);
    let original = map.encounters[0].clone();
    let mut legacy_record = serde_json::to_value(&original).expect("encounter should serialize");
    legacy_record
        .as_object_mut()
        .expect("encounter should be an object")
        .remove("quest_id");
    legacy_record
        .as_object_mut()
        .expect("encounter should be an object")
        .remove("spawn_group");
    let legacy_record: SpatialEncounter =
        serde_json::from_value(legacy_record).expect("older encounters should omit quest IDs");
    assert!(legacy_record.quest_id.is_none());
    assert!(legacy_record.spawn_group.is_none());

    assert!(map.is_walkable(original.position));
    assert!(reachable_walkable_tiles(&map).contains(&original.position));
    assert_ne!(original.position, map.entry);
    assert!(map.validate_encounters().is_ok());

    let mut with_actor = map.clone();
    ensure_spatial_entity_placements(&mut with_actor, &[(700, SpatialEntityKind::Npc)]);
    let actor_position = with_actor
        .interactables
        .iter()
        .find(|placement| placement.entity_id == 700)
        .expect("new actor should receive a placement")
        .position;
    assert_ne!(actor_position, original.position);

    assert!(!ensure_spatial_encounter(
        &mut map,
        "encounter.test.1".into(),
        "Different Enemy".into(),
        "This must not replace the saved encounter.".into(),
        99,
        9,
        "Different Trophy".into(),
    ));
    assert!(ensure_spatial_encounter(
        &mut map,
        "encounter.test.2".into(),
        "Second Enemy".into(),
        "Independent encounters can share one persistent map.".into(),
        10,
        2,
        "Second Trophy".into(),
    ));
    assert_eq!(map.encounters.len(), 2);
    assert_eq!(map.encounters[0], original);
    assert_ne!(map.encounters[0].position, map.encounters[1].position);
    assert!(map
        .encounters
        .iter()
        .all(|encounter| encounter.status == SpatialEncounterStatus::Active));
    assert!(map.validate_encounters().is_ok());

    let mut legacy = serde_json::to_value(&map).expect("map should serialize");
    legacy
        .as_object_mut()
        .expect("map should be an object")
        .remove("encounters");
    let restored: LocalMap =
        serde_json::from_value(legacy).expect("older maps should omit encounter data");
    assert!(restored.encounters.is_empty());
}

#[test]
fn encounter_spawn_avoids_the_players_saved_tile() {
    let mut map = generate_local_map(
        85,
        "location.encounter_player_overlap",
        LocalMapConfig::default(),
    );
    let player_position = reachable_walkable_tiles(&map)
        .into_iter()
        .find(|position| position.x.abs_diff(map.entry.x) + position.y.abs_diff(map.entry.y) >= 2)
        .expect("generated map should have a free player tile");
    assert!(ensure_spatial_encounter_avoiding(
        &mut map,
        SpatialEncounterDefinition {
            id: "encounter.player_overlap".into(),
            quest_id: None,
            spawn_group: None,
            enemy_name: "Ash Marauder".into(),
            description: "A hostile figure approaches.".into(),
            max_hp: 12,
            power: 2,
            trophy_item_name: "Marauder Token".into(),
        },
        Some(player_position),
    ));
    assert_ne!(map.encounters[0].position, player_position);
}

#[test]
fn existing_spatial_entity_placements_are_preserved_when_new_actors_arrive() {
    let mut map = generate_local_map(83, "location.actor_arrival", LocalMapConfig::default());
    ensure_spatial_entity_placements(&mut map, &[(1, SpatialEntityKind::Npc)]);
    let original = map.interactables[0].clone();

    ensure_spatial_entity_placements(
        &mut map,
        &[(1, SpatialEntityKind::Npc), (2, SpatialEntityKind::Npc)],
    );

    assert_eq!(map.interactables[0], original);
    assert_eq!(map.interactables.len(), 2);
    assert_ne!(map.interactables[0].position, map.interactables[1].position);
}

#[test]
fn legacy_points_of_interest_default_to_no_evidence_prerequisites() {
    let legacy = r#"{
        "id": "poi.legacy",
        "position": {"x": 2, "y": 3},
        "title": "Old Discovery",
        "description": "A discovery from an earlier save.",
        "discovered": true,
        "interacted": true
    }"#;
    let poi: PointOfInterest =
        serde_json::from_str(legacy).expect("legacy POIs should load without prerequisites");
    assert!(poi.prerequisites.is_empty());

    let mut current = poi;
    current.prerequisites = vec!["evidence.first".into()];
    let serialized = serde_json::to_string(&current).expect("POI should serialize");
    let restored: PointOfInterest =
        serde_json::from_str(&serialized).expect("current POI should deserialize");
    assert_eq!(restored.prerequisites, vec!["evidence.first"]);
}

#[test]
fn authored_spatial_discoveries_are_deterministic_reachable_and_unique() {
    let definitions = vec![
        SpatialDiscoveryDefinition {
            id: "poi.old_shrine.black_altar".into(),
            title: "Black Altar".into(),
            description: "Soot clings to the altar.".into(),
            prerequisites: vec!["poi.old_shrine.fresh_soot".into()],
        },
        SpatialDiscoveryDefinition {
            id: "poi.old_shrine.fresh_soot".into(),
            title: "Fresh Soot".into(),
            description: "Fresh soot crosses the floor.".into(),
            prerequisites: vec!["poi.hollow_market.unlit_lantern".into()],
        },
    ];

    let mut first = generate_local_map_with_layout(
        42,
        "location.old_shrine",
        LocalMapConfig::default(),
        Some("shrine_chamber"),
    );
    configure_map_exits(&mut first, &[101, 202]);
    let mut second = first.clone();

    assert_eq!(
        apply_authored_spatial_discoveries(&mut first, &definitions),
        2
    );
    assert_eq!(
        apply_authored_spatial_discoveries(&mut second, &definitions),
        2
    );
    assert_eq!(first.points_of_interest, second.points_of_interest);
    assert_eq!(first.points_of_interest.len(), definitions.len());
    assert!(first
        .points_of_interest
        .iter()
        .all(|poi| !poi.discovered && !poi.interacted));
    assert_eq!(
        first
            .point_of_interest("poi.old_shrine.fresh_soot")
            .unwrap()
            .prerequisites,
        vec!["poi.hollow_market.unlit_lantern"]
    );
    assert_eq!(
        first
            .point_of_interest("poi.old_shrine.black_altar")
            .unwrap()
            .prerequisites,
        vec!["poi.old_shrine.fresh_soot"]
    );

    let positions = first
        .points_of_interest
        .iter()
        .map(|poi| poi.position)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(positions.len(), definitions.len());
    for poi in &first.points_of_interest {
        assert!(first.is_walkable(poi.position));
        assert!(has_walkable_path(&first, first.entry, poi.position));
        assert_ne!(poi.position, first.entry);
        assert!(!first.exits.iter().any(|exit| exit.position == poi.position));
        assert!(!first
            .interactables
            .iter()
            .any(|actor| actor.position == poi.position));
    }
}

#[test]
fn no_authored_discoveries_preserve_the_legacy_fallback() {
    let mut map = generate_local_map(42, "location.legacy_discovery", LocalMapConfig::default());
    let original = map.points_of_interest.clone();

    assert_eq!(apply_authored_spatial_discoveries(&mut map, &[]), 0);
    assert_eq!(map.points_of_interest, original);
}

#[test]
fn editor_operations_validate_terrain_and_complete_prop_footprints() {
    let mut map = generate_local_map(73, "developer.world_lab", LocalMapConfig::default());
    let poi = map.points_of_interest[0].position;
    assert!(!map.set_terrain_for_editor(map.entry, Terrain::Wall));
    assert!(!map.set_terrain_for_editor(poi, Terrain::Wall));
    assert!(!map.place_prop("editor.entry", SpatialPropKind::Pillar, map.entry));
    assert!(!map.place_prop("editor.poi", SpatialPropKind::Pillar, poi));

    let is_free_cell = |map: &LocalMap, position: GridPosition| {
        map.is_walkable(position)
            && position != map.entry
            && !map
                .points_of_interest
                .iter()
                .any(|point| point.position == position)
            && !map.exits.iter().any(|exit| exit.position == position)
            && !map
                .interactables
                .iter()
                .any(|actor| actor.position == position)
    };
    let find_table_position = |map: &LocalMap| {
        (1..map.height - 1)
            .flat_map(|y| (1..map.width - 2).map(move |x| GridPosition { x, y }))
            .find(|position| {
                is_free_cell(map, *position)
                    && is_free_cell(
                        map,
                        GridPosition {
                            x: position.x + 1,
                            y: position.y,
                        },
                    )
            })
    };
    let table_position = find_table_position(&map).expect("generated map should fit a table");

    assert!(map.place_prop("editor.table", SpatialPropKind::Table, table_position));
    let second_table_cell = GridPosition {
        x: table_position.x + 1,
        y: table_position.y,
    };
    assert!(!map.is_walkable(table_position));
    assert!(!map.is_walkable(second_table_cell));
    assert!(!map.place_prop("editor.overlap", SpatialPropKind::Pillar, second_table_cell));
    assert!(!map.set_terrain_for_editor(table_position, Terrain::Ash));
    assert!(!map.move_prop("editor.table", map.entry));

    let destination = find_table_position(&map).expect("generated map should fit a second table");
    assert!(map.move_prop("editor.table", destination));
    assert!(!map.is_walkable(destination));
    assert!(map
        .remove_prop_at(GridPosition {
            x: destination.x + 1,
            y: destination.y
        })
        .is_some());
    assert!(map.props.iter().all(|prop| prop.id != "editor.table"));

    let terrain_position = (1..map.height - 1)
        .flat_map(|y| (1..map.width - 1).map(move |x| GridPosition { x, y }))
        .find(|position| is_free_cell(&map, *position))
        .expect("generated map should have an editable cell");
    assert!(map.set_terrain_for_editor(terrain_position, Terrain::Wall));
    assert_eq!(map.terrain_at(terrain_position), Some(Terrain::Wall));
    assert!(map
        .terrain_changes
        .iter()
        .any(|change| { change.position == terrain_position && change.terrain == Terrain::Wall }));
}

#[test]
fn editor_operations_place_move_and_remove_points_of_interest_safely() {
    let mut map = generate_local_map(91, "developer.world_lab_poi", LocalMapConfig::default());
    let original = map.points_of_interest[0].clone();

    let is_free_cell = |map: &LocalMap, position: GridPosition| {
        map.is_walkable(position)
            && position != map.entry
            && !map
                .points_of_interest
                .iter()
                .any(|point| point.position == position)
            && !map.exits.iter().any(|exit| exit.position == position)
            && !map
                .interactables
                .iter()
                .any(|actor| actor.position == position)
    };
    let find_free_cell = |map: &LocalMap, excluded: Option<GridPosition>| {
        (1..map.height - 1)
            .flat_map(|y| (1..map.width - 1).map(move |x| GridPosition { x, y }))
            .find(|position| Some(*position) != excluded && is_free_cell(map, *position))
    };

    let first_position = find_free_cell(&map, None).expect("map should have a free walkable cell");
    let point = PointOfInterest {
        id: "developer.test.discovery".into(),
        position: first_position,
        title: "Test Discovery".into(),
        description: "An editable discovery.".into(),
        discovered: true,
        interacted: true,
        prerequisites: Vec::new(),
    };
    assert!(!map.place_point_of_interest(PointOfInterest {
        position: map.entry,
        ..point.clone()
    }));
    assert!(map.place_point_of_interest(point.clone()));
    assert!(!map.place_point_of_interest(point.clone()));

    let second_position = find_free_cell(&map, Some(first_position))
        .expect("map should have a second free walkable cell");
    assert!(!map.move_point_of_interest(&point.id, map.entry));
    assert!(!map.move_point_of_interest(&point.id, GridPosition { x: 0, y: 0 }));
    assert!(map.move_point_of_interest(&point.id, second_position));

    let moved = map
        .point_of_interest(&point.id)
        .expect("moved point should remain in the map");
    assert_eq!(moved.position, second_position);
    assert!(moved.discovered);
    assert!(moved.interacted);
    assert_eq!(moved.title, point.title);
    assert_eq!(moved.description, point.description);
    assert_eq!(map.points_of_interest[0], original);

    let mut expected_removed = point.clone();
    expected_removed.position = second_position;
    assert_eq!(
        map.remove_point_of_interest_at(second_position),
        Some(expected_removed)
    );
    assert!(map.point_of_interest(&point.id).is_none());
    assert!(map.remove_point_of_interest_at(second_position).is_none());
}

#[test]
fn editor_operations_place_move_and_remove_passages_using_valid_world_destinations() {
    let mut map = generate_local_map(111, "location.10", LocalMapConfig::default());
    let destination_id = 20_u64;
    let exit = MapExit {
        id: "location.10.exit.20".into(),
        route_id: None,
        route_profile: None,
        frontier: None,
        position: GridPosition {
            x: map.width - 1,
            y: 1,
        },
        target_location_id: destination_id,
    };

    assert!(!map.place_map_exit(exit.clone(), &[21]));
    assert!(!map.place_map_exit(
        MapExit {
            position: map.entry,
            ..exit.clone()
        },
        &[destination_id]
    ));

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
    let find_placement = |map: &LocalMap, exit: &MapExit| {
        edge_positions.iter().copied().find(|position| {
            let mut candidate = map.clone();
            candidate.place_map_exit(
                MapExit {
                    position: *position,
                    ..exit.clone()
                },
                &[destination_id],
            )
        })
    };
    let first_position = find_placement(&map, &exit)
        .expect("at least one east-edge passage should have a clear route");

    let mut self_link_map = map.clone();
    assert!(!self_link_map.place_map_exit(
        MapExit {
            id: "location.10.exit.self".into(),
            route_id: None,
            route_profile: None,
            frontier: None,
            position: first_position,
            target_location_id: 10,
        },
        &[10],
    ));

    let exit = MapExit {
        position: first_position,
        ..exit
    };
    assert!(map.place_map_exit(exit.clone(), &[destination_id]));
    assert_eq!(map.exits, vec![exit.clone()]);

    let second_position = edge_positions
        .iter()
        .copied()
        .filter(|position| *position != first_position)
        .find(|position| {
            let mut candidate = map.clone();
            candidate.move_map_exit(&exit.id, *position)
        })
        .expect("a second edge should provide a valid passage route");
    assert!(!map.move_map_exit(&exit.id, map.entry));
    assert!(map.move_map_exit(&exit.id, second_position));
    assert_eq!(map.exits[0].position, second_position);
    assert_eq!(map.exits[0].target_location_id, destination_id);

    assert_eq!(
        map.remove_map_exit_at(second_position),
        Some(MapExit {
            position: second_position,
            ..exit
        })
    );
    assert!(map.exits.is_empty());
}

#[test]
fn editor_operations_place_move_and_remove_real_world_actor_placements() {
    let mut map = generate_local_map(121, "location.10", LocalMapConfig::default());
    let valid_entities = [
        (101_u64, SpatialEntityKind::Npc),
        (202_u64, SpatialEntityKind::Corpse),
    ];
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
    let first_position = find_free_cell(&map, None).expect("map should have a free actor cell");
    let actor = SpatialEntityPlacement {
        entity_id: 101,
        kind: SpatialEntityKind::Npc,
        position: first_position,
    };

    assert!(!map.place_spatial_entity(actor.clone(), &[(999, SpatialEntityKind::Npc)]));
    assert!(!map.place_spatial_entity(
        SpatialEntityPlacement {
            position: map.entry,
            ..actor.clone()
        },
        &valid_entities
    ));
    assert!(map.place_spatial_entity(actor.clone(), &valid_entities));
    assert!(!map.place_spatial_entity(actor.clone(), &valid_entities));

    let second_position =
        find_free_cell(&map, Some(first_position)).expect("map should have another actor cell");
    assert!(!map.move_spatial_entity(actor.entity_id, actor.kind, map.entry));
    assert!(map.move_spatial_entity(actor.entity_id, actor.kind, second_position));
    assert_eq!(map.interactables[0].position, second_position);
    assert_eq!(map.interactables[0].entity_id, actor.entity_id);

    let mut expected_removed = actor;
    expected_removed.position = second_position;
    assert_eq!(
        map.remove_spatial_entity_at(second_position),
        Some(expected_removed)
    );
    assert!(map.interactables.is_empty());
}

#[test]
fn interior_door_states_gate_pathfinding_without_blocking_floor_cells() {
    let mut map = generate_local_map_with_layout(
        42,
        "location.old_shrine",
        LocalMapConfig::default(),
        Some("shrine_chamber"),
    );
    let door = map
        .doors
        .first()
        .expect("shrine should have its threshold door")
        .clone();
    assert!(map.validate_room_topology().is_ok());
    assert!(map.can_cross_edge(door.edge.a, door.edge.b));
    assert!(map.is_walkable(door.edge.a));
    assert!(map.is_walkable(door.edge.b));

    assert!(map.set_door_state(&door.id, DoorState::Closed));
    assert!(!map.can_cross_edge(door.edge.a, door.edge.b));
    assert!(!map.can_cross_edge(door.edge.b, door.edge.a));
    assert!(map.set_door_state(&door.id, DoorState::Open));
    assert!(map.can_cross_edge(door.edge.a, door.edge.b));

    assert!(map.set_door_state(&door.id, DoorState::Locked));
    assert!(!map.set_door_state(&door.id, DoorState::Open));
    assert!(!map.can_cross_edge(door.edge.a, door.edge.b));
    assert!(map.unlock_door(&door.id));
    assert_eq!(map.doors[0].state, DoorState::Closed);
    assert!(map.set_door_state(&door.id, DoorState::Open));
    assert!(map.validate_room_topology().is_ok());
}

#[test]
fn grid_edges_are_canonical_and_require_cardinal_adjacency() {
    let a = GridPosition { x: 4, y: 5 };
    let b = GridPosition { x: 5, y: 5 };
    assert_eq!(GridEdge::new(a, b), GridEdge::new(b, a));
    assert!(GridEdge::new(a, GridPosition { x: 5, y: 6 }).is_none());
    assert!(GridEdge::new(a, GridPosition { x: 6, y: 5 }).is_none());
}

#[test]
fn closed_shrine_threshold_blocks_room_to_room_pathfinding() {
    let mut map = generate_local_map_with_layout(
        42,
        "location.old_shrine",
        LocalMapConfig::default(),
        Some("shrine_chamber"),
    );
    let door = map
        .doors
        .first()
        .expect("shrine should define its threshold")
        .clone();
    let chamber_position = GridPosition { x: 5, y: 6 };
    let approach_position = GridPosition { x: 5, y: 8 };
    assert!(has_walkable_path(&map, chamber_position, approach_position));
    assert!(map.set_door_state(&door.id, DoorState::Closed));
    assert!(!has_walkable_path(
        &map,
        chamber_position,
        approach_position
    ));
    assert!(map.set_door_state(&door.id, DoorState::Open));
    assert!(has_walkable_path(&map, chamber_position, approach_position));
}

#[test]
fn terrain_editor_cannot_turn_a_room_cell_into_a_wall() {
    let mut map = generate_local_map_with_layout(
        42,
        "location.old_shrine",
        LocalMapConfig::default(),
        Some("shrine_chamber"),
    );
    let room_cell = map.rooms[0].cells[0];
    assert!(!map.set_terrain_for_editor(room_cell, Terrain::Wall));
    assert!(map.is_walkable(room_cell));
}

#[test]
fn map_exit_profile_defaults_when_loading_an_older_saved_passage() {
    let mut map = generate_local_map(817, "location.10", LocalMapConfig::default());
    let exit = MapExit {
        id: "location.10.exit.20".into(),
        route_id: None,
        route_profile: None,
        frontier: None,
        position: GridPosition {
            x: map.width - 1,
            y: 2,
        },
        target_location_id: 20,
    };
    let mut value = serde_json::to_value(exit).unwrap();
    value.as_object_mut().unwrap().remove("route_profile");
    value.as_object_mut().unwrap().remove("frontier");
    let loaded: MapExit = serde_json::from_value(value).unwrap();
    assert!(loaded.route_profile.is_none());
    map.exits.push(loaded);
    assert_eq!(map.exits.len(), 1);
}

#[test]
fn route_reconciliation_uses_profile_and_preferred_side_for_new_passages() {
    use crate::geography::routes::RouteProfile;
    use crate::spatial::MapExitSide;

    let mut map = generate_local_map(818, "location.10", LocalMapConfig::default());
    map.terrain.fill(Terrain::Floor);
    map.props.clear();
    map.points_of_interest.clear();
    map.rooms.clear();
    map.doors.clear();
    map.interactables.clear();
    map.encounters.clear();
    configure_map_routes(
        &mut map,
        &[MapRouteDestination {
            route_id: "route.northbound".into(),
            target_location_id: 20,
            profile: Some(RouteProfile::MountainPass),
            preferred_side: Some(MapExitSide::North),
            frontier_id: None,
        }],
    );

    assert_eq!(map.exits.len(), 1);
    assert_eq!(map.exits[0].route_profile, Some(RouteProfile::MountainPass));
    assert_eq!(map.exits[0].position.y, 0);
}

#[test]
fn route_reconciliation_backfills_profile_without_moving_existing_passage() {
    use crate::geography::routes::RouteProfile;
    use crate::spatial::MapExitSide;

    let mut map = generate_local_map(819, "location.10", LocalMapConfig::default());
    map.terrain.fill(Terrain::Floor);
    map.props.clear();
    map.points_of_interest.clear();
    map.rooms.clear();
    map.doors.clear();
    map.interactables.clear();
    map.encounters.clear();
    let position = GridPosition {
        x: map.width - 1,
        y: 2,
    };
    let existing = MapExit {
        id: "location.10.route.route.east".into(),
        route_id: Some("route.east".into()),
        route_profile: None,
        frontier: None,
        position,
        target_location_id: 20,
    };
    assert!(map.place_map_exit(existing, &[20]));

    configure_map_routes(
        &mut map,
        &[MapRouteDestination {
            route_id: "route.east".into(),
            target_location_id: 20,
            profile: Some(RouteProfile::RiverCrossing),
            preferred_side: Some(MapExitSide::North),
            frontier_id: None,
        }],
    );

    assert_eq!(map.exits.len(), 1);
    assert_eq!(map.exits[0].position, position);
    assert_eq!(
        map.exits[0].route_profile,
        Some(RouteProfile::RiverCrossing)
    );
}

#[test]
fn route_reconciliation_preserves_existing_passages_and_adds_parallel_routes_idempotently() {
    let mut map = generate_local_map(818, "location.10", LocalMapConfig::default());
    let routes = vec![
        MapRouteDestination {
            route_id: "route.path.alpha".into(),
            target_location_id: 20,
            profile: None,
            preferred_side: None,
            frontier_id: None,
        },
        MapRouteDestination {
            route_id: "route.path.beta".into(),
            target_location_id: 20,
            profile: None,
            preferred_side: None,
            frontier_id: None,
        },
    ];

    configure_map_routes(&mut map, &routes);
    assert_eq!(map.exits.len(), 2);
    assert_eq!(
        map.exits
            .iter()
            .map(|exit| exit.route_id.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("route.path.alpha"), Some("route.path.beta")]
    );
    assert_ne!(map.exits[0].position, map.exits[1].position);
    let established = map.exits[0].clone();

    let mut expanded_routes = routes.clone();
    expanded_routes.push(MapRouteDestination {
        route_id: "route.path.gamma".into(),
        target_location_id: 21,
        profile: None,
        preferred_side: None,
        frontier_id: None,
    });
    configure_map_routes(&mut map, &expanded_routes);
    assert_eq!(map.exits.len(), 3);
    assert_eq!(map.exits[0], established);

    let before_repeat = map.exits.clone();
    configure_map_routes(&mut map, &expanded_routes);
    assert_eq!(map.exits, before_repeat);
}

#[test]
fn route_reconciliation_distributes_new_passages_round_robin_by_compass_side() {
    let mut map = generate_local_map(820, "location.10", LocalMapConfig::default());
    map.props.clear();
    map.points_of_interest.clear();
    let routes = (0..6)
        .map(|slot| MapRouteDestination {
            route_id: format!("route.slot.{slot:02}"),
            target_location_id: 20 + slot,
            profile: None,
            preferred_side: None,
            frontier_id: None,
        })
        .collect::<Vec<_>>();

    configure_map_routes(&mut map, &routes);

    let sides = map
        .exits
        .iter()
        .map(|exit| {
            if exit.position.x == map.width - 1 {
                "east"
            } else if exit.position.x == 0 {
                "west"
            } else if exit.position.y == 0 {
                "north"
            } else {
                "south"
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sides,
        vec!["east", "west", "north", "south", "east", "west"]
    );
}

#[test]
fn route_reconciliation_attaches_legacy_passage_without_relocating_it() {
    let mut map = generate_local_map(819, "location.10", LocalMapConfig::default());
    let position = GridPosition {
        x: map.width - 1,
        y: 1,
    };
    assert!(map.set_terrain(position, Terrain::Floor));
    map.exits.push(MapExit {
        id: "location.10.exit.20".into(),
        route_id: None,
        route_profile: None,
        frontier: None,
        position,
        target_location_id: 20,
    });

    configure_map_routes(
        &mut map,
        &[MapRouteDestination {
            route_id: "route.migrated.20".into(),
            target_location_id: 20,
            profile: None,
            preferred_side: None,
            frontier_id: None,
        }],
    );

    assert_eq!(map.exits.len(), 1);
    assert_eq!(map.exits[0].id, "location.10.exit.20");
    assert_eq!(map.exits[0].position, position);
    assert_eq!(map.exits[0].route_id.as_deref(), Some("route.migrated.20"));
}

#[test]
fn pending_frontier_passages_are_idempotent_and_upgrade_in_place() {
    let mut map = generate_local_map(2_731, "location.10", LocalMapConfig::default());
    map.props.clear();
    map.points_of_interest.clear();
    map.interactables.clear();
    map.encounters.clear();
    let frontier = FrontierTarget {
        id: "frontier.fixture.east".into(),
        target_region: crate::geography::RegionCoordinate::new(1, 0),
        direction: FrontierDirection::East,
    };

    configure_map_frontiers(&mut map, std::slice::from_ref(&frontier));
    assert_eq!(map.exits.len(), 1);
    let pending = map.exits[0].clone();
    assert_eq!(pending.target_location_id, 0);
    assert_eq!(pending.route_id, None);
    assert_eq!(pending.frontier, Some(frontier.clone()));

    configure_map_frontiers(&mut map, std::slice::from_ref(&frontier));
    assert_eq!(map.exits, vec![pending.clone()]);

    configure_map_routes(
        &mut map,
        &[MapRouteDestination {
            route_id: "route.frontier.fixture.east".into(),
            target_location_id: 44,
            profile: Some(RouteProfile::Trail),
            preferred_side: Some(MapExitSide::East),
            frontier_id: Some(frontier.id.clone()),
        }],
    );

    assert_eq!(map.exits.len(), 1);
    assert_eq!(map.exits[0].id, pending.id);
    assert_eq!(map.exits[0].position, pending.position);
    assert_eq!(map.exits[0].target_location_id, 44);
    assert_eq!(
        map.exits[0].route_id.as_deref(),
        Some("route.frontier.fixture.east")
    );
    assert_eq!(map.exits[0].route_profile, Some(RouteProfile::Trail));
    assert_eq!(map.exits[0].frontier, None);

    let established = map.exits.clone();
    configure_map_routes(
        &mut map,
        &[MapRouteDestination {
            route_id: "route.frontier.fixture.east".into(),
            target_location_id: 44,
            profile: Some(RouteProfile::Trail),
            preferred_side: Some(MapExitSide::East),
            frontier_id: Some(frontier.id),
        }],
    );
    assert_eq!(map.exits, established);
}
