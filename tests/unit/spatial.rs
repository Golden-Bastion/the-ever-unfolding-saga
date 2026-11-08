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
fn props_round_trip_and_older_saves_default_to_no_props() {
    let map = generate_local_map(12, "location.old_shrine", LocalMapConfig::default());
    let json = serde_json::to_string(&map).expect("map with props should serialize");
    let restored: LocalMap =
        serde_json::from_str(&json).expect("map with props should deserialize");
    assert_eq!(restored.props, map.props);

    let mut old_value = serde_json::to_value(&map).expect("map should serialize");
    let object = old_value.as_object_mut().expect("map should be an object");
    object.remove("props");
    object.remove("interactables");
    object.remove("exits");
    object.remove("layout_id");
    let restored_old: LocalMap =
        serde_json::from_value(old_value).expect("pre-prop map should remain loadable");
    assert!(restored_old.props.is_empty());
    assert!(restored_old.interactables.is_empty());
    assert!(restored_old.exits.is_empty());
    assert!(restored_old.layout_id.is_none());
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
fn authored_spatial_discoveries_are_deterministic_reachable_and_unique() {
    let definitions = vec![
        SpatialDiscoveryDefinition {
            id: "poi.old_shrine.black_altar".into(),
            title: "Black Altar".into(),
            description: "Soot clings to the altar.".into(),
        },
        SpatialDiscoveryDefinition {
            id: "poi.old_shrine.fresh_soot".into(),
            title: "Fresh Soot".into(),
            description: "Fresh soot crosses the floor.".into(),
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
