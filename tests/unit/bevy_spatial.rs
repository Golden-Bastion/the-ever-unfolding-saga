use super::*;
use crate::spatial::generate_local_map;

#[test]
fn spatial_hud_condition_summary_has_a_clear_empty_state() {
    assert_eq!(format_condition_summary(&[]), "Conditions: none");
    let conditions = vec![
        Condition::new("Wounded", 3, 1),
        Condition::new("Exhausted", 1, 1),
    ];
    assert_eq!(
        format_condition_summary(&conditions),
        "Conditions: Wounded, Exhausted"
    );
}

#[test]
fn spatial_world_update_initializes_with_disjoint_sprite_queries() {
    let mut app = App::new();
    app.insert_resource(Time::default())
        .init_resource::<SpatialExplorationState>()
        .init_resource::<NavigationState>()
        .init_resource::<crate::bevy_gameplay::GameplayRuntimeState>()
        .init_resource::<SpatialRenderAssets>()
        .add_systems(Update, update_spatial_world);

    // Bevy validates query access during system initialization/execution. This
    // smoke test catches mutable Sprite query overlaps before platform startup.
    app.update();
}

#[test]
fn moving_sprite_depth_tracks_continuous_ground_position() {
    let center = Vec2::new(7.5, 8.5);
    let behind = Vec2::new(7.1, 8.2);
    let ahead = Vec2::new(7.9, 8.8);
    assert!(depth_for_position(behind) < depth_for_position(center));
    assert!(depth_for_position(center) < depth_for_position(ahead));
    assert!(depth_for_position(center) + PLAYER_DEPTH_OFFSET > depth_for(center));
}

#[test]
fn ground_terrain_never_occludes_actors_at_different_map_depths() {
    for terrain in [Terrain::Floor, Terrain::Ash, Terrain::Rubble] {
        for actor_position in [
            Vec2::new(0.5, 0.5),
            Vec2::new(6.5, 7.5),
            Vec2::new(22.5, 14.5),
        ] {
            let terrain_center = actor_position + Vec2::new(0.5, 0.5);
            assert!(
                terrain_depth(terrain, terrain_center)
                    < depth_for_position(actor_position) + PLAYER_DEPTH_OFFSET,
                "{terrain:?} terrain must render behind actor at {actor_position:?}"
            );
        }
    }
}

#[test]
fn wall_tops_and_faces_keep_depth_based_occlusion() {
    let wall_center = Vec2::new(6.5, 7.5);
    let top_depth = terrain_depth(Terrain::Wall, wall_center);
    let face_depth = wall_face_depth(wall_center);
    let actor_behind_top = wall_center - Vec2::splat(0.1);
    let actor_in_front_of_top = wall_center + Vec2::splat(0.1);
    let actor_behind_face = wall_center + Vec2::splat(0.4);
    let actor_in_front_of_face = wall_center + Vec2::splat(0.6);

    assert!(depth_for_position(actor_behind_top) + PLAYER_DEPTH_OFFSET < top_depth);
    assert!(depth_for_position(actor_in_front_of_top) + PLAYER_DEPTH_OFFSET > top_depth);
    assert!(
        top_depth < depth_for_position(wall_center) + PLAYER_DEPTH_OFFSET,
        "an actor and wall top at equal ground depth must render with the actor in front"
    );
    assert!(depth_for_position(actor_behind_face) + PLAYER_DEPTH_OFFSET < face_depth);
    assert!(depth_for_position(actor_in_front_of_face) + PLAYER_DEPTH_OFFSET > face_depth);
}

#[test]
fn selected_folder_spatial_pngs_decode_with_expected_dimensions() {
    for (reference, width, height) in [
        ("spatial/terrain_floor.png", 48, 24),
        ("spatial/terrain_ash.png", 48, 24),
        ("spatial/terrain_rubble.png", 48, 24),
        ("spatial/terrain_wall_top.png", 48, 24),
        ("spatial/player_oathbound.png", 16, 24),
        ("spatial/passage.png", 24, 32),
        ("spatial/wall_face_left.png", 24, 36),
        ("spatial/wall_face_right.png", 24, 36),
        ("spatial/prop_pillar.png", 32, 40),
        ("spatial/prop_table.png", 72, 48),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/assets")
            .join(reference);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("asset {} should exist: {error}", path.display()));
        let image = Image::from_buffer(
            &bytes,
            bevy::image::ImageType::Extension("png"),
            bevy::image::CompressedImageFormats::NONE,
            true,
            bevy::image::ImageSampler::nearest(),
            bevy::asset::RenderAssetUsages::default(),
        )
        .unwrap_or_else(|error| panic!("asset {} should decode: {error}", path.display()));
        assert_eq!(
            image.texture_descriptor.size.width, width,
            "{reference} width"
        );
        assert_eq!(
            image.texture_descriptor.size.height, height,
            "{reference} height"
        );
    }
}

#[test]
fn prop_depth_occludes_actors_behind_but_not_ahead_or_at_equal_depth() {
    let prop_position = Vec2::new(8.5, 8.5);
    let prop_z = prop_depth(prop_position);
    let behind = prop_position - Vec2::splat(0.2);
    let ahead = prop_position + Vec2::splat(0.2);

    assert!(depth_for_position(behind) + PLAYER_DEPTH_OFFSET < prop_z);
    assert!(depth_for_position(ahead) + PLAYER_DEPTH_OFFSET > prop_z);
    assert!(depth_for_position(prop_position) + PLAYER_DEPTH_OFFSET > prop_z);
}

#[test]
fn prop_footprints_block_interaction_line_of_sight() {
    let mut map = generate_local_map(4, "location.prop_los_test", LocalMapConfig::default());
    map.props.clear();
    for x in 5..=9 {
        assert!(map.set_terrain(GridPosition { x, y: 7 }, Terrain::Floor));
    }
    map.props.push(crate::spatial::SpatialProp {
        id: "test.pillar".to_string(),
        kind: SpatialPropKind::Pillar,
        position: GridPosition { x: 7, y: 7 },
    });

    let start = Vec2::new(5.5, 7.5);
    let target = Vec2::new(9.5, 7.5);
    assert!(!has_line_of_sight(&map, start, target));

    map.props.clear();
    assert!(has_line_of_sight(&map, start, target));
}

#[test]
fn angled_projection_offsets_neighboring_tiles_consistently() {
    let map = generate_local_map(4, "location.test", LocalMapConfig::default());
    let origin = map_to_world(Vec2::new(5.5, 5.5), Some(&map));
    let x_neighbor = map_to_world(Vec2::new(6.5, 5.5), Some(&map));
    let y_neighbor = map_to_world(Vec2::new(5.5, 6.5), Some(&map));
    assert!((x_neighbor.x - origin.x - TILE_HALF_WIDTH).abs() < 0.001);
    assert!((x_neighbor.y - origin.y + TILE_HALF_HEIGHT).abs() < 0.001);
    assert!((y_neighbor.x - origin.x + TILE_HALF_WIDTH).abs() < 0.001);
    assert!((y_neighbor.y - origin.y + TILE_HALF_HEIGHT).abs() < 0.001);
}

#[test]
fn projected_input_axes_match_screen_directions() {
    for screen_direction in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        let world = screen_direction_to_world(screen_direction);
        let projected = Vec2::new(
            (world.x - world.y) * TILE_HALF_WIDTH,
            -(world.x + world.y) * TILE_HALF_HEIGHT,
        );
        assert!((projected - screen_direction).length() < 0.001);
    }
}

#[test]
fn camera_dead_zone_does_not_follow_small_player_movements() {
    let camera = Vec2::new(20.0, -10.0);
    let viewport = Vec2::new(800.0, 450.0);
    let player = camera + Vec2::new(40.0, -20.0);
    assert_eq!(
        camera_target_with_dead_zone(player, camera, viewport),
        camera
    );
}

#[test]
fn camera_dead_zone_moves_only_by_the_excess_offset() {
    let camera = Vec2::ZERO;
    let viewport = Vec2::new(800.0, 450.0);
    let player = Vec2::new(200.0, -100.0);
    let target = camera_target_with_dead_zone(player, camera, viewport);
    assert!((target.x - 140.0).abs() < 0.001);
    assert!((target.y + 66.25).abs() < 0.001);
}

#[test]
fn camera_follow_is_consistent_across_frame_rates() {
    let map = generate_local_map(4, "location.camera_follow_test", LocalMapConfig::default());
    let player = Vec2::new(200.0, 100.0);
    let viewport = Vec2::new(320.0, 180.0);

    let mut camera_30 = Vec2::ZERO;
    for _ in 0..30 {
        camera_30 = camera_follow_position(player, camera_30, viewport, &map, 1.0 / 30.0);
    }

    let mut camera_60 = Vec2::ZERO;
    for _ in 0..60 {
        camera_60 = camera_follow_position(player, camera_60, viewport, &map, 1.0 / 60.0);
    }

    assert!(
        camera_30.distance(camera_60) < 0.001,
        "30 FPS and 60 FPS camera paths should converge identically: {camera_30:?} vs {camera_60:?}"
    );
}

#[test]
fn camera_follow_clamps_the_smoothed_position_to_map_bounds() {
    let map = generate_local_map(4, "location.camera_bounds_test", LocalMapConfig::default());
    let viewport = Vec2::new(320.0, 180.0);
    let camera = Vec2::new(10_000.0, -10_000.0);
    let followed = camera_follow_position(Vec2::ZERO, camera, viewport, &map, 0.0);

    assert_eq!(followed, clamp_camera_target(camera, &map, viewport));
}

#[test]
fn camera_bounds_center_maps_smaller_than_the_viewport() {
    let map = generate_local_map(
        4,
        "location.test",
        LocalMapConfig {
            width: 8,
            height: 8,
        },
    );
    let target = clamp_camera_target(Vec2::new(200.0, -100.0), &map, Vec2::new(2000.0, 1200.0));
    assert_eq!(target, Vec2::new(0.0, -6.0));
}

#[test]
fn camera_bounds_keep_the_viewport_inside_large_map_extents() {
    let map = generate_local_map(4, "location.test", LocalMapConfig::default());
    let viewport = Vec2::new(320.0, 180.0);
    let target = clamp_camera_target(Vec2::new(10000.0, -10000.0), &map, viewport);
    let half_width = (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_WIDTH * 0.5;
    let half_height = (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_HEIGHT * 0.5;
    assert!(target.x <= half_width - viewport.x * 0.5 + 0.001);
    assert!(target.x >= -half_width + viewport.x * 0.5 - 0.001);
    assert!(target.y <= half_height - viewport.y * 0.5 + 0.001);
    assert!(target.y >= -half_height - (WALL_HEIGHT - TILE_HALF_HEIGHT) + viewport.y * 0.5 - 0.001);
}

#[test]
fn contextual_interaction_requires_range_and_clear_approach() {
    let mut map = generate_local_map(4, "location.test", LocalMapConfig::default());
    map.points_of_interest[0].discovered = true;
    let poi = &map.points_of_interest[0];
    let near = Vec2::new(
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );
    assert!(poi_interaction_available_for(&map, near, poi));
    assert!(!poi_interaction_available_for(
        &map,
        near + Vec2::new(3.0, 0.0),
        poi,
    ));
}

#[test]
fn contextual_interaction_prefers_a_closer_point_of_interest() {
    let mut map = generate_local_map(4, "location.test", LocalMapConfig::default());
    configure_map_exits(&mut map, &[99]);
    let exit_position = map.exits[0].position;
    map.points_of_interest[0].position = GridPosition {
        x: map.width - 2,
        y: exit_position.y,
    };
    map.points_of_interest[0].discovered = true;
    let poi_position = map.points_of_interest[0].position;
    let player_position = Vec2::new(
        f32::from(poi_position.x) + 0.5,
        f32::from(poi_position.y) + 0.5,
    );
    let exit = &map.exits[0];

    assert!(poi_interaction_available_for(
        &map,
        player_position,
        &map.points_of_interest[0],
    ));
    assert!(exit_interaction_available(&map, player_position, exit));
    let game = crate::model::create_new_state(
        "Test World",
        crate::model::WorldMode::New,
        "Ash".to_string(),
        "Wanderer".to_string(),
    );
    assert_eq!(
        nearest_interaction_target(&map, &game, player_position)
            .map(|interaction| interaction.action),
        Some(InteractionActionKind::Examine)
    );
}

#[test]
fn eligible_interactions_preserve_multiple_nearby_targets_and_bindings() {
    let mut map = generate_local_map(4, "location.interaction_choices", LocalMapConfig::default());
    map.props.clear();
    map.points_of_interest[0].position = map.entry;
    map.points_of_interest[0].discovered = true;
    let poi_id = map.points_of_interest[0].id.clone();
    map.exits = vec![MapExit {
        id: "exit.test".into(),
        position: map.entry,
        target_location_id: 999,
    }];

    let mut game = crate::model::create_new_state(
        "Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_id = game.character.location_id;
    let source_location = game.world.location_by_id(location_id).unwrap().clone();
    let destination_id = game.world.allocate_id();
    game.world.locations.push(crate::model::Location {
        id: destination_id,
        name: "Passage Destination".into(),
        description: "A connected place.".into(),
        region_id: source_location.region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: vec![location_id],
        spatial_layout: None,
    });
    game.world
        .location_by_id_mut(location_id)
        .unwrap()
        .exits
        .push(destination_id);
    map.exits[0].target_location_id = destination_id;

    game.npcs.push(crate::model::Npc::new(
        70,
        "Mira",
        "Scout",
        location_id,
        None,
    ));
    game.npcs.push(crate::model::Npc::new(
        71,
        "Bram",
        "Keeper",
        location_id,
        None,
    ));
    game.corpses.push(crate::model::Corpse {
        id: 80,
        former_name: "Unknown Wanderer".into(),
        former_title: "Pilgrim".into(),
        location_id,
        turn_of_death: 0,
        inventory: vec![crate::model::Item {
            id: 81,
            name: "Rusty Key".into(),
            description: "A key darkened by age.".into(),
        }],
        epitaph: String::new(),
        scavenged: false,
    });
    for (entity_id, kind) in [
        (70, SpatialEntityKind::Npc),
        (71, SpatialEntityKind::Npc),
        (80, SpatialEntityKind::Corpse),
    ] {
        map.interactables
            .push(crate::spatial::SpatialEntityPlacement {
                entity_id,
                kind,
                position: map.entry,
            });
    }

    let position = Vec2::new(f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    let items = available_interactions(&map, &game, position);
    assert_eq!(
        items
            .iter()
            .filter(|item| item.action == InteractionActionKind::Talk)
            .count(),
        2
    );
    assert!(items
        .iter()
        .any(|item| item.target == InteractionTarget::Npc(70)));
    assert!(items
        .iter()
        .any(|item| item.target == InteractionTarget::Npc(71)));
    assert!(items
        .iter()
        .any(|item| item.target == InteractionTarget::Corpse(80)));
    assert!(items
        .iter()
        .any(|item| item.target == InteractionTarget::PointOfInterest(poi_id.clone())));
    assert!(items.iter().any(|item| {
        item.target
            == InteractionTarget::Passage {
                id: "exit.test".into(),
                destination_id,
            }
    }));
    for (index, item) in items.iter().enumerate() {
        assert!(
            !items[..index]
                .iter()
                .any(|previous| previous.stable_key() == item.stable_key()),
            "each wheel entry must keep a unique action/target identity",
        );
    }
}

#[test]
fn passage_interaction_requires_approach_and_range() {
    let mut map = generate_local_map(4, "location.test", LocalMapConfig::default());
    configure_map_exits(&mut map, &[99]);
    let exit = &map.exits[0];
    let at_passage = Vec2::new(
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    assert!(exit_interaction_available(&map, at_passage, exit));

    let entry = Vec2::new(f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    assert!(!exit_interaction_available(&map, entry, exit));
}

#[test]
fn passage_targets_are_independent_of_destination_danger() {
    // Danger belongs to the target world location; LocalMap stores only its
    // stable world ID, so both dangerous and safe destinations use the
    // same physical-exit representation.
    let mut map = generate_local_map(4, "location.test", LocalMapConfig::default());
    configure_map_exits(&mut map, &[99, 100]);
    assert_eq!(map.exits.len(), 2);
    assert_eq!(map.exits[0].target_location_id, 99);
    assert_eq!(map.exits[1].target_location_id, 100);
}

#[test]
fn virtual_joystick_dead_zone_suppresses_small_drift() {
    assert_eq!(virtual_joystick_direction(Vec2::new(5.0, 5.0)), Vec2::ZERO);
}

#[test]
fn virtual_joystick_scales_partial_deflection() {
    let direction = virtual_joystick_direction(Vec2::new(JOYSTICK_RADIUS_PX * 0.5, 0.0));
    assert!(direction.length() > 0.0);
    assert!(direction.length() < 1.0);
}

#[test]
fn virtual_joystick_direction_is_normalized_at_full_diagonal_deflection() {
    let direction = virtual_joystick_direction(Vec2::new(50.0, -50.0));
    assert!((direction.length() - 1.0).abs() < 0.001);
    assert!(direction.x > 0.0);
    assert!(direction.y > 0.0);
}

#[test]
fn virtual_joystick_vertical_touch_motion_matches_game_direction() {
    let upward = virtual_joystick_direction(Vec2::new(0.0, -JOYSTICK_RADIUS_PX));
    let downward = virtual_joystick_direction(Vec2::new(0.0, JOYSTICK_RADIUS_PX));
    assert!(upward.y > 0.99);
    assert!(downward.y < -0.99);
}

#[test]
fn movement_is_continuous_within_walkable_tiles() {
    let map = generate_local_map(4, "location.test", LocalMapConfig::default());
    let start = Vec2::new(f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    // Supply a local-world-units-per-pixel scale, not a raw tile-sized vector.
    let world_units_per_pixel = Vec2::X / 45.0;
    let moved = advance_position(&map, start, world_units_per_pixel, 0.1, PLAYER_SPEED);
    assert!(moved.x > start.x);
    assert!((moved.x - start.x - 0.4).abs() < 0.001);
}

#[test]
fn movement_catches_up_through_short_frame_drops() {
    let mut map = generate_local_map(
        4,
        "location.movement_stutter_test",
        LocalMapConfig::default(),
    );
    map.props.clear();
    for x in map.entry.x..=map.entry.x + 2 {
        map.set_terrain(GridPosition { x, y: map.entry.y }, Terrain::Floor);
    }

    let start = Vec2::new(f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    let world_units_per_pixel = Vec2::X / 45.0;
    let moved = advance_position(&map, start, world_units_per_pixel, 0.2, PLAYER_SPEED);

    assert!((moved.x - start.x - 0.8).abs() < 0.001);
    assert_eq!(moved.y, start.y);
}

#[test]
fn movement_substeps_do_not_allow_crossing_blocked_terrain_after_a_slow_frame() {
    let mut map = generate_local_map(
        4,
        "location.movement_collision_stutter_test",
        LocalMapConfig::default(),
    );
    map.props.clear();
    let start = Vec2::new(f32::from(map.entry.x) + 0.8, f32::from(map.entry.y) + 0.5);
    let wall = GridPosition {
        x: map.entry.x + 1,
        y: map.entry.y,
    };
    map.set_terrain(wall, Terrain::Wall);
    let world_units_per_pixel = Vec2::X / 45.0;
    let moved = advance_position(&map, start, world_units_per_pixel, 0.2, PLAYER_SPEED);

    assert!(moved.x <= start.x);
    assert_eq!(moved.y, start.y);
}

#[test]
fn movement_does_not_cross_blocked_terrain() {
    let mut map = generate_local_map(4, "location.test", LocalMapConfig::default());
    let start = Vec2::new(f32::from(map.entry.x) + 0.8, f32::from(map.entry.y) + 0.5);
    let wall = GridPosition {
        x: map.entry.x + 1,
        y: map.entry.y,
    };
    map.set_terrain(wall, Terrain::Wall);
    // One screen-space pixel must map to a small local displacement; a raw
    // Vec2::X would represent 18 whole tiles at this speed and timestep.
    let world_units_per_pixel = Vec2::X / 45.0;
    let moved = advance_position(&map, start, world_units_per_pixel, 0.1, PLAYER_SPEED);
    assert_eq!(moved, start);
}

#[test]
fn radial_wheel_directional_selection_uses_the_closest_entry() {
    assert_eq!(interaction_wheel_index_for_direction(0, Vec2::Y), None);
    assert_eq!(interaction_wheel_index_for_direction(4, Vec2::Y), Some(0));
    assert_eq!(interaction_wheel_index_for_direction(4, Vec2::X), Some(1));
    assert_eq!(
        interaction_wheel_index_for_direction(4, Vec2::NEG_Y),
        Some(2)
    );
    assert_eq!(
        interaction_wheel_index_for_direction(4, Vec2::NEG_X),
        Some(3)
    );
    assert_eq!(interaction_wheel_index_for_direction(1, Vec2::X), Some(0));
}

#[test]
fn radial_wheel_scale_eases_between_open_and_closed_states() {
    let opening = SpatialInteractionWheel {
        entries: Vec::new(),
        selected: 0,
        phase: InteractionWheelPhase::Opening,
        elapsed: 0.0,
        selection_elapsed: INTERACTION_WHEEL_SELECTION_ANIMATION_SECONDS,
        pending: None,
    };
    assert!((interaction_wheel_scale(&opening) - 0.78).abs() < 0.001);

    let open = SpatialInteractionWheel {
        phase: InteractionWheelPhase::Open,
        ..opening
    };
    assert_eq!(interaction_wheel_scale(&open), 1.0);
}

#[test]
fn same_target_actions_have_distinct_wheel_identities() {
    let talk = AvailableInteraction {
        action: InteractionActionKind::Talk,
        target: InteractionTarget::Npc(70),
        label: "Talk — Mira".into(),
    };
    let examine = AvailableInteraction {
        action: InteractionActionKind::Examine,
        target: InteractionTarget::Npc(70),
        label: "Examine — Mira".into(),
    };
    assert_ne!(talk.stable_key(), examine.stable_key());
}

#[test]
fn radial_wheel_selection_feedback_pulses_smoothly() {
    assert_eq!(interaction_wheel_entry_scale(false, 0.0), 1.0);
    assert_eq!(interaction_wheel_entry_scale(true, 0.0), 1.0);

    let midpoint =
        interaction_wheel_entry_scale(true, INTERACTION_WHEEL_SELECTION_ANIMATION_SECONDS * 0.5);
    assert!(midpoint > 1.0);
    assert!(midpoint <= 1.091);

    let settled =
        interaction_wheel_entry_scale(true, INTERACTION_WHEEL_SELECTION_ANIMATION_SECONDS);
    assert!((settled - 1.0).abs() < 0.001);
}

#[test]
fn radial_wheel_size_and_anchor_adapt_to_landscape_viewports() {
    assert_eq!(interaction_wheel_size(Vec2::new(1280.0, 720.0)), 300.0);
    assert_eq!(interaction_wheel_size(Vec2::new(800.0, 360.0)), 300.0);
    assert_eq!(interaction_wheel_size(Vec2::new(320.0, 260.0)), 244.0);

    let viewport = Vec2::new(320.0, 260.0);
    let size = interaction_wheel_size(viewport);
    let top_left = interaction_wheel_root_position(Vec2::ZERO, viewport, size);
    let bottom_right = interaction_wheel_root_position(viewport, viewport, size);

    assert_eq!(top_left, Vec2::splat(8.0));
    assert!(bottom_right.x + size <= viewport.x - 8.0);
    assert!(bottom_right.y + size <= viewport.y - 8.0);
}

#[test]
fn crowded_radial_wheels_use_two_rings_without_clipping_entries() {
    for count in 9..=14 {
        for size in [INTERACTION_WHEEL_SIZE_PX, 244.0] {
            for index in 0..count {
                let layout = interaction_wheel_entry_layout(index, count, size);
                assert!(
                    layout.position.x >= 0.0,
                    "entry {index} of {count} clips left at {size}"
                );
                assert!(
                    layout.position.y >= 0.0,
                    "entry {index} of {count} clips top at {size}"
                );
                assert!(
                    layout.position.x + layout.width <= size,
                    "entry {index} of {count} clips right at {size}"
                );
                assert!(
                    layout.position.y + INTERACTION_WHEEL_BUTTON_HEIGHT_PX <= size,
                    "entry {index} of {count} clips bottom at {size}"
                );

                let expected_radius = if index < INTERACTION_WHEEL_OUTER_RING_ENTRIES {
                    INTERACTION_WHEEL_OUTER_RING_RADIUS_PX * size / INTERACTION_WHEEL_SIZE_PX
                } else {
                    INTERACTION_WHEEL_INNER_RING_RADIUS_PX * size / INTERACTION_WHEEL_SIZE_PX
                };
                assert!((layout.center_offset.length() - expected_radius).abs() < 0.01);
            }
        }
    }
}
#[test]
fn crowded_wheel_directional_selection_matches_visible_cardinal_entries() {
    assert_eq!(interaction_wheel_index_for_direction(9, Vec2::Y), Some(0));
    assert_eq!(interaction_wheel_index_for_direction(9, Vec2::X), Some(2));
    assert_eq!(
        interaction_wheel_index_for_direction(9, Vec2::NEG_Y),
        Some(4)
    );
    assert_eq!(
        interaction_wheel_index_for_direction(9, Vec2::NEG_X),
        Some(6)
    );
}

#[test]
fn points_of_interest_reveal_when_their_tiles_enter_the_explored_area() {
    let mut map = generate_local_map(17, "location.poi_reveal", LocalMapConfig::default());
    let entry = map.entry;
    let near = GridPosition {
        x: entry.x.saturating_add(2),
        y: entry.y,
    };
    let far = GridPosition {
        x: map.width.saturating_sub(2),
        y: map.height.saturating_sub(2),
    };
    map.points_of_interest = vec![
        crate::spatial::PointOfInterest {
            id: "poi.near".into(),
            position: near,
            title: "Near clue".into(),
            description: "Close enough to notice.".into(),
            discovered: false,
            interacted: false,
        },
        crate::spatial::PointOfInterest {
            id: "poi.far".into(),
            position: far,
            title: "Far clue".into(),
            description: "Still beyond the explored area.".into(),
            discovered: false,
            interacted: false,
        },
    ];
    let location_id = map.location_id.clone();
    let position = Vec2::new(f32::from(entry.x) + 0.5, f32::from(entry.y) + 0.5);
    let mut state = SpatialExplorationState {
        maps: HashMap::from([(location_id.clone(), map)]),
        session: Some(ExplorationSession {
            location_id,
            position: entry,
        }),
        position,
        ..default()
    };

    assert_eq!(map_mark_discovered(&mut state, position), vec!["Near clue"]);
    let map = state.maps.values().next().expect("map should be present");
    assert!(map.point_of_interest("poi.near").unwrap().discovered);
    assert!(!map.point_of_interest("poi.far").unwrap().discovered);
    assert!(map_mark_discovered(&mut state, position).is_empty());
}

fn connected_spatial_fixture() -> (
    LifecycleState,
    SpatialExplorationState,
    NavigationState,
    EntityId,
    String,
) {
    let mut game = crate::model::create_new_state(
        "Passage Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    // Spatial travel tests cover passage and map behavior, not random arrival events.
    game.campaign_content
        .as_mut()
        .expect("campaign content should be loaded")
        .events
        .retain(|event| event.trigger != "travel_arrival");

    let source_location_id = game.character.location_id;
    let source_location = game
        .world
        .location_by_id(source_location_id)
        .expect("starting location should exist")
        .clone();
    let destination_id = game.world.allocate_id();
    game.world.locations.push(crate::model::Location {
        id: destination_id,
        name: "Connected Clearing".into(),
        description: "A quiet place beyond the passage.".into(),
        region_id: source_location.region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: vec![source_location_id],
        spatial_layout: None,
    });
    game.world
        .location_by_id_mut(source_location_id)
        .expect("starting location should still exist")
        .exits
        .push(destination_id);
    if let Some(region) = game
        .world
        .regions
        .iter_mut()
        .find(|region| region.id == source_location.region_id)
    {
        region.location_ids.push(destination_id);
    }

    let destination_key = format!("location.{destination_id}");
    let mut destination_map = generate_local_map(91, destination_key, LocalMapConfig::default());
    destination_map.set_terrain(destination_map.entry, Terrain::Rubble);
    destination_map
        .points_of_interest
        .push(crate::spatial::PointOfInterest {
            id: "poi.destination.persisted".into(),
            position: destination_map.entry,
            title: "Old Waystone".into(),
            description: "A waystone left by an earlier traveller.".into(),
            discovered: true,
            interacted: false,
        });
    game.world.local_maps.push(destination_map);

    let mut lifecycle = LifecycleState::default();
    lifecycle.phase = LifecyclePhase::Complete;
    lifecycle.session = Some(crate::bevy_lifecycle::GameSession {
        state: game,
        save_path: std::path::PathBuf::new(),
    });
    let mut spatial = SpatialExplorationState::default();
    let mut navigation = NavigationState::default();
    enter(
        &mut spatial,
        &lifecycle
            .session
            .as_ref()
            .expect("game session should exist")
            .state,
        &mut navigation,
    );
    (
        lifecycle,
        spatial,
        navigation,
        destination_id,
        format!("location.{source_location_id}"),
    )
}

fn extend_known_spatial_route(
    lifecycle: &mut LifecycleState,
    middle_id: EntityId,
    dangerous_middle: bool,
) -> EntityId {
    let game = &mut lifecycle.session.as_mut().unwrap().state;
    let middle = game
        .world
        .location_by_id_mut(middle_id)
        .expect("known intermediate location should exist");
    middle.dangerous = dangerous_middle;
    let region_id = middle.region_id;
    let destination_id = game.world.allocate_id();
    game.world.locations.push(crate::model::Location {
        id: destination_id,
        name: "Remote Hollow".into(),
        description: "A familiar place farther along the old road.".into(),
        region_id,
        dangerous: false,
        corpse_ids: Vec::new(),
        exits: vec![middle_id],
        spatial_layout: None,
    });
    game.world
        .location_by_id_mut(middle_id)
        .expect("known intermediate location should still exist")
        .exits
        .push(destination_id);
    if let Some(region) = game
        .world
        .regions
        .iter_mut()
        .find(|region| region.id == region_id)
    {
        region.location_ids.push(destination_id);
    }
    let map = generate_local_map(
        92,
        format!("location.{destination_id}"),
        LocalMapConfig::default(),
    );
    game.world.local_maps.push(map);
    destination_id
}

#[test]
fn map_fast_travel_follows_multiple_known_route_steps_and_restores_the_final_map() {
    let (mut lifecycle, mut spatial, mut navigation, middle_id, _) = connected_spatial_fixture();
    let target_id = extend_known_spatial_route(&mut lifecycle, middle_id, false);
    let before_turn = lifecycle.session.as_ref().unwrap().state.character.turn;
    let before_time = lifecycle.session.as_ref().unwrap().state.world.time_points;

    fast_travel_to_known_location(&mut spatial, &mut lifecycle, &mut navigation, target_id)
        .expect("known destinations should be reachable through the visited route graph");

    let game = &lifecycle.session.as_ref().unwrap().state;
    let target_key = format!("location.{target_id}");
    assert_eq!(game.character.location_id, target_id);
    assert_eq!(game.character.turn, before_turn + 2);
    assert_eq!(game.world.time_points, before_time + 4);
    assert_eq!(spatial.session.as_ref().unwrap().location_id, target_key);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert!(spatial
        .location_notice
        .as_deref()
        .is_some_and(|message| message.contains("Remote Hollow")));
    assert!(game
        .world
        .local_maps
        .iter()
        .any(|map| map.location_id == target_key));
}

#[test]
fn map_fast_travel_stops_at_an_intermediate_dangerous_location() {
    let (mut lifecycle, mut spatial, mut navigation, middle_id, _) = connected_spatial_fixture();
    let target_id = extend_known_spatial_route(&mut lifecycle, middle_id, true);
    let before_turn = lifecycle.session.as_ref().unwrap().state.character.turn;
    let before_time = lifecycle.session.as_ref().unwrap().state.world.time_points;

    fast_travel_to_known_location(&mut spatial, &mut lifecycle, &mut navigation, target_id)
        .expect("the route should stop safely at the dangerous intermediate location");

    let game = &lifecycle.session.as_ref().unwrap().state;
    assert_eq!(game.character.location_id, middle_id);
    assert_eq!(game.character.turn, before_turn + 1);
    assert_eq!(game.world.time_points, before_time + 2);
    assert!(game.threat.active);
    assert!(spatial
        .message
        .as_deref()
        .is_some_and(|message| message.contains("interrupted")));
    assert_eq!(
        spatial.session.as_ref().unwrap().location_id,
        format!("location.{middle_id}")
    );
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
}

#[test]
fn spatial_entry_announces_location_without_persisting_it_in_the_header() {
    let game = crate::model::create_new_state(
        "Location Notice Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_name = game
        .world
        .location_by_id(game.character.location_id)
        .expect("starting location should exist")
        .name
        .clone();
    let mut spatial = SpatialExplorationState::default();
    let mut navigation = NavigationState::default();

    enter(&mut spatial, &game, &mut navigation);

    let expected_notice = format!("You arrive at {location_name}.");
    assert_eq!(
        spatial.location_notice.as_deref(),
        Some(expected_notice.as_str())
    );
    assert_eq!(
        spatial.message.as_deref(),
        Some("Explore the area. Move close to people or objects to interact.")
    );
    assert_eq!(spatial_header_label(false, 0), None);
}

#[test]
fn spatial_location_notice_expires_without_erasing_regular_feedback() {
    let mut spatial = SpatialExplorationState::default();
    announce_location(&mut spatial, "Connected Clearing");
    spatial.message = Some("A distant bell rings.".to_string());

    advance_location_notice(&mut spatial, LOCATION_NOTICE_DURATION_SECONDS - 0.5);
    assert!(spatial.location_notice.is_some());

    advance_location_notice(&mut spatial, 0.5);
    assert_eq!(spatial.location_notice, None);
    assert_eq!(spatial.location_notice_remaining_seconds, 0.0);
    assert_eq!(spatial.message.as_deref(), Some("A distant bell rings."));
}

#[test]
fn sandbox_header_keeps_its_world_lab_identity() {
    assert_eq!(
        spatial_header_label(true, 91).as_deref(),
        Some("WORLD LAB - PLAY TEST / SEED 91")
    );
}

#[test]
fn physical_passage_travel_keeps_spatial_mode_and_restores_destination_map() {
    let (mut lifecycle, mut spatial, mut navigation, destination_id, source_key) =
        connected_spatial_fixture();
    let exit = spatial
        .maps
        .get(&source_key)
        .expect("source map should be loaded")
        .exits
        .iter()
        .find(|exit| exit.target_location_id == destination_id)
        .expect("source map should contain the connected passage")
        .clone();
    let source_entry = spatial.maps[&source_key].entry;
    spatial
        .maps
        .get_mut(&source_key)
        .unwrap()
        .set_terrain(source_entry, Terrain::Rubble);
    spatial.position = Vec2::new(
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    spatial.session.as_mut().unwrap().position = exit.position;
    let before_turn = lifecycle.session.as_ref().unwrap().state.character.turn;

    use_map_exit(
        &mut spatial,
        &mut lifecycle,
        &mut navigation,
        &exit.id,
        destination_id,
    );

    let game = &lifecycle.session.as_ref().unwrap().state;
    let destination_key = format!("location.{destination_id}");
    assert_eq!(game.character.location_id, destination_id);
    assert_eq!(game.character.turn, before_turn + 1);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(
        spatial.session.as_ref().unwrap().location_id,
        destination_key
    );
    assert_eq!(
        spatial.position,
        Vec2::new(
            f32::from(spatial.maps[&destination_key].entry.x) + 0.5,
            f32::from(spatial.maps[&destination_key].entry.y) + 0.5,
        )
    );
    assert!(spatial.maps[&destination_key]
        .point_of_interest("poi.destination.persisted")
        .is_some_and(|poi| poi.discovered));
    assert!(game.world.local_maps.iter().any(|map| {
        map.location_id == source_key && map.terrain_at(source_entry) == Some(Terrain::Rubble)
    }));
    assert!(game.world.local_maps.iter().any(|map| {
        map.location_id == destination_key
            && map
                .point_of_interest("poi.destination.persisted")
                .is_some_and(|poi| poi.discovered)
    }));
    let character_session = game.character.exploration_session.as_ref().unwrap();
    assert_eq!(character_session.location_id, destination_key);
    assert_eq!(
        game.character.exploration_position,
        Some(ExplorationPosition {
            x: spatial.position.x,
            y: spatial.position.y,
        })
    );
    assert!(spatial
        .location_notice
        .as_deref()
        .is_some_and(|message| message.contains("Connected Clearing")));
    assert!(!spatial
        .message
        .as_deref()
        .is_some_and(|message| message.contains("Connected Clearing")));
}

#[test]
fn physical_passage_rejects_a_stale_world_connection_without_traveling() {
    let (mut lifecycle, mut spatial, mut navigation, destination_id, source_key) =
        connected_spatial_fixture();
    let exit = spatial
        .maps
        .get(&source_key)
        .unwrap()
        .exits
        .iter()
        .find(|exit| exit.target_location_id == destination_id)
        .unwrap()
        .clone();
    spatial.position = Vec2::new(
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    spatial.session.as_mut().unwrap().position = exit.position;
    let source_location_id = lifecycle
        .session
        .as_ref()
        .unwrap()
        .state
        .character
        .location_id;
    lifecycle
        .session
        .as_mut()
        .unwrap()
        .state
        .world
        .location_by_id_mut(source_location_id)
        .unwrap()
        .exits
        .retain(|target| *target != destination_id);
    let before_turn = lifecycle.session.as_ref().unwrap().state.character.turn;

    use_map_exit(
        &mut spatial,
        &mut lifecycle,
        &mut navigation,
        &exit.id,
        destination_id,
    );

    let game = &lifecycle.session.as_ref().unwrap().state;
    assert_eq!(game.character.location_id, source_location_id);
    assert_eq!(game.character.turn, before_turn);
    assert_eq!(spatial.session.as_ref().unwrap().location_id, source_key);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert!(spatial
        .message
        .as_deref()
        .is_some_and(|message| message.contains("no longer leads")));
}

#[test]
fn physical_passage_rejects_a_world_connection_without_a_return_route() {
    let (mut lifecycle, mut spatial, mut navigation, destination_id, source_key) =
        connected_spatial_fixture();
    let exit = spatial
        .maps
        .get(&source_key)
        .unwrap()
        .exits
        .iter()
        .find(|exit| exit.target_location_id == destination_id)
        .unwrap()
        .clone();
    spatial.position = Vec2::new(
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    spatial.session.as_mut().unwrap().position = exit.position;
    lifecycle
        .session
        .as_mut()
        .unwrap()
        .state
        .world
        .location_by_id_mut(destination_id)
        .unwrap()
        .exits
        .clear();
    let source_location_id = lifecycle
        .session
        .as_ref()
        .unwrap()
        .state
        .character
        .location_id;
    let before_turn = lifecycle.session.as_ref().unwrap().state.character.turn;

    use_map_exit(
        &mut spatial,
        &mut lifecycle,
        &mut navigation,
        &exit.id,
        destination_id,
    );

    let game = &lifecycle.session.as_ref().unwrap().state;
    assert_eq!(game.character.location_id, source_location_id);
    assert_eq!(game.character.turn, before_turn);
    assert_eq!(spatial.session.as_ref().unwrap().location_id, source_key);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert!(spatial
        .message
        .as_deref()
        .is_some_and(|message| message.contains("no longer leads")));
}

#[test]
fn physical_passage_generates_a_destination_map_on_first_arrival() {
    let (mut lifecycle, mut spatial, mut navigation, destination_id, source_key) =
        connected_spatial_fixture();
    let destination_key = format!("location.{destination_id}");
    lifecycle
        .session
        .as_mut()
        .unwrap()
        .state
        .world
        .local_maps
        .retain(|map| map.location_id != destination_key);
    let exit = spatial
        .maps
        .get(&source_key)
        .unwrap()
        .exits
        .iter()
        .find(|exit| exit.target_location_id == destination_id)
        .unwrap()
        .clone();
    spatial.position = Vec2::new(
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    spatial.session.as_mut().unwrap().position = exit.position;
    let source_location_id = lifecycle
        .session
        .as_ref()
        .unwrap()
        .state
        .character
        .location_id;

    use_map_exit(
        &mut spatial,
        &mut lifecycle,
        &mut navigation,
        &exit.id,
        destination_id,
    );

    let game = &lifecycle.session.as_ref().unwrap().state;
    let destination_map = spatial
        .maps
        .get(&destination_key)
        .expect("first arrival should create the destination map");
    assert!(destination_map
        .exits
        .iter()
        .any(|map_exit| { map_exit.target_location_id == source_location_id }));
    assert!(game
        .world
        .local_maps
        .iter()
        .any(|map| map.location_id == destination_key));
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
}

#[test]
fn known_location_fast_travel_saves_the_source_map_and_restores_the_destination_map() {
    let (mut lifecycle, mut spatial, mut navigation, destination_id, source_key) =
        connected_spatial_fixture();
    let source_entry = spatial.maps[&source_key].entry;
    spatial
        .maps
        .get_mut(&source_key)
        .unwrap()
        .set_terrain(source_entry, Terrain::Rubble);
    spatial.position = Vec2::new(
        f32::from(source_entry.x) + 0.5,
        f32::from(source_entry.y) + 0.5,
    );
    spatial.session.as_mut().unwrap().position = source_entry;
    let before_turn = lifecycle.session.as_ref().unwrap().state.character.turn;

    fast_travel_to_known_location(
        &mut spatial,
        &mut lifecycle,
        &mut navigation,
        destination_id,
    )
    .expect("connected destinations should use the normal world travel action");

    let game = &lifecycle.session.as_ref().unwrap().state;
    let destination_key = format!("location.{destination_id}");
    assert_eq!(game.character.location_id, destination_id);
    assert!(game.character.turn > before_turn);
    assert!(game.world.local_maps.iter().any(|map| {
        map.location_id == source_key && map.terrain_at(source_entry) == Some(Terrain::Rubble)
    }));
    assert_eq!(
        spatial.session.as_ref().unwrap().location_id,
        destination_key
    );
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert!(spatial
        .location_notice
        .as_deref()
        .is_some_and(|message| message.contains("Connected Clearing")));
    assert!(!spatial
        .message
        .as_deref()
        .is_some_and(|message| message.contains("Connected Clearing")));
}

#[test]
fn known_location_fast_travel_rejects_a_stale_connection_without_leaving_spatial_play() {
    let (mut lifecycle, mut spatial, mut navigation, destination_id, source_key) =
        connected_spatial_fixture();
    let source_location_id = lifecycle
        .session
        .as_ref()
        .unwrap()
        .state
        .character
        .location_id;
    lifecycle
        .session
        .as_mut()
        .unwrap()
        .state
        .world
        .location_by_id_mut(source_location_id)
        .unwrap()
        .exits
        .retain(|target| *target != destination_id);
    let before_turn = lifecycle.session.as_ref().unwrap().state.character.turn;

    let result = fast_travel_to_known_location(
        &mut spatial,
        &mut lifecycle,
        &mut navigation,
        destination_id,
    );

    assert_eq!(
        result,
        Err("That location is not reachable through known routes.")
    );
    let game = &lifecycle.session.as_ref().unwrap().state;
    assert_eq!(game.character.location_id, source_location_id);
    assert_eq!(game.character.turn, before_turn);
    assert_eq!(spatial.session.as_ref().unwrap().location_id, source_key);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
}

#[test]
fn world_lab_playtest_is_disposable_and_returns_updated_map_to_editor() {
    let map = generate_local_map(123, "developer.world_lab", LocalMapConfig::default());
    let mut state = SpatialExplorationState::default();
    set_sandbox_map(&mut state, map);
    let mut navigation = NavigationState::default();

    assert!(start_sandbox_playtest(&mut state, &mut navigation));
    assert!(state.sandbox_mode);
    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert!(state.session.is_some());

    // The normal synchronization path must be a no-op: sandbox playtests do not
    // create a campaign or write their temporary map into world state.
    let mut lifecycle = LifecycleState::default();
    let game = crate::model::create_new_state(
        "Sandbox Isolation Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    lifecycle.session = Some(crate::bevy_lifecycle::GameSession {
        state: game,
        save_path: std::path::PathBuf::new(),
    });
    let original_local_maps = lifecycle
        .session
        .as_ref()
        .expect("campaign fixture should exist")
        .state
        .world
        .local_maps
        .clone();
    let original_exploration = lifecycle
        .session
        .as_ref()
        .expect("campaign fixture should exist")
        .state
        .character
        .exploration_session
        .clone();

    sync_spatial_to_game(&state, &mut lifecycle);

    let game_after_sync = &lifecycle.session.as_ref().unwrap().state;
    assert_eq!(game_after_sync.world.local_maps, original_local_maps);
    assert_eq!(
        game_after_sync.character.exploration_session,
        original_exploration
    );

    let current_position = state.position;
    map_mark_discovered(&mut state, current_position);
    let discovered_before_exit = state
        .maps
        .values()
        .next()
        .expect("playtest map should exist")
        .discovered_tiles
        .clone();
    assert!(discovered_before_exit.iter().any(|discovered| *discovered));

    exit_spatial(&mut state, &mut navigation);

    assert!(!state.sandbox_mode);
    assert!(state.session.is_none());
    assert!(state.maps.is_empty());
    assert_eq!(navigation.current_screen, Some(ScreenId::WorldLab));
    assert_eq!(
        sandbox_map(&state)
            .expect("updated map should return to the editor")
            .discovered_tiles,
        discovered_before_exit
    );
}

#[test]
fn sandbox_map_cannot_be_replaced_during_an_active_playtest() {
    let original = generate_local_map(123, "developer.world_lab", LocalMapConfig::default());
    let replacement = generate_local_map(456, "developer.world_lab", LocalMapConfig::default());
    let mut state = SpatialExplorationState::default();
    set_sandbox_map(&mut state, original.clone());
    let mut navigation = NavigationState::default();
    assert!(start_sandbox_playtest(&mut state, &mut navigation));

    set_sandbox_map(&mut state, replacement);

    let active_map = state
        .maps
        .values()
        .next()
        .expect("playtest map should remain mounted");
    assert_eq!(active_map.location_id, original.location_id);
    assert_eq!(active_map.seed, original.seed);
    assert_eq!(active_map.terrain, original.terrain);
    assert_eq!(active_map.props, original.props);
}

#[test]
fn contextual_prompt_names_each_target_instead_of_showing_an_action() {
    let mut game = crate::model::create_new_state(
        "Prompt Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let npc_id = game.world.allocate_id();
    game.npcs.push(crate::model::Npc::new(
        npc_id,
        "Mara",
        "Keeper",
        game.character.location_id,
        None,
    ));
    let corpse_id = game.world.allocate_id();
    game.corpses.push(crate::model::Corpse {
        id: corpse_id,
        former_name: "The Fallen Guard".into(),
        location_id: game.character.location_id,
        ..Default::default()
    });
    let destination_id = game
        .world
        .locations
        .iter()
        .find(|location| location.id != game.character.location_id)
        .expect("generated world should contain another location")
        .id;
    let mut map = generate_local_map(42, "location.prompt_test", LocalMapConfig::default());
    map.points_of_interest.clear();
    map.points_of_interest
        .push(crate::spatial::PointOfInterest {
            id: "test.black_altar".into(),
            position: map.entry,
            title: "Black Altar".into(),
            description: "A dark stone altar.".into(),
            discovered: true,
            interacted: false,
        });

    let interaction = |target, label: &str| AvailableInteraction {
        action: InteractionActionKind::Examine,
        target,
        label: label.into(),
    };
    assert_eq!(
        interaction_prompt_label(
            &[interaction(InteractionTarget::Npc(npc_id), "Talk — Mara")],
            &game,
            &map,
        ),
        "Mara"
    );
    assert_eq!(
        interaction_prompt_label(
            &[interaction(
                InteractionTarget::Corpse(corpse_id),
                "Search — The Fallen Guard"
            )],
            &game,
            &map,
        ),
        "The Fallen Guard"
    );
    assert_eq!(
        interaction_prompt_label(
            &[interaction(
                InteractionTarget::PointOfInterest("test.black_altar".into()),
                "Examine — Black Altar"
            )],
            &game,
            &map,
        ),
        "Black Altar"
    );
    let destination_name = game
        .world
        .location_by_id(destination_id)
        .unwrap()
        .name
        .clone();
    assert_eq!(
        interaction_prompt_label(
            &[interaction(
                InteractionTarget::Passage {
                    id: "test.passage".into(),
                    destination_id
                },
                "Enter — elsewhere"
            )],
            &game,
            &map,
        ),
        destination_name
    );
}

#[test]
fn contextual_prompt_counts_distinct_targets_not_action_entries() {
    let mut game = crate::model::create_new_state(
        "Prompt Count Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let npc_id = game.world.allocate_id();
    game.npcs.push(crate::model::Npc::new(
        npc_id,
        "Mara",
        "Keeper",
        game.character.location_id,
        None,
    ));
    let map = generate_local_map(43, "location.prompt_count_test", LocalMapConfig::default());
    let interactions = vec![
        AvailableInteraction {
            action: InteractionActionKind::Talk,
            target: InteractionTarget::Npc(npc_id),
            label: "Talk — Mara".into(),
        },
        AvailableInteraction {
            action: InteractionActionKind::Examine,
            target: InteractionTarget::Npc(npc_id),
            label: "Examine — Mara".into(),
        },
    ];
    assert_eq!(interaction_prompt_label(&interactions, &game, &map), "Mara");
}

#[test]
fn returning_to_spatial_exploration_requests_a_scene_rebuild() {
    assert!(should_refresh_spatial_scene(
        Some(ScreenId::Character),
        Some(ScreenId::SpatialExploration)
    ));
    assert!(should_refresh_spatial_scene(
        Some(ScreenId::Options),
        Some(ScreenId::SpatialExploration)
    ));
    assert!(!should_refresh_spatial_scene(
        Some(ScreenId::SpatialExploration),
        Some(ScreenId::SpatialExploration)
    ));
    assert!(!should_refresh_spatial_scene(
        Some(ScreenId::SpatialExploration),
        Some(ScreenId::Gameplay)
    ));
}

#[test]
fn spatial_investigation_is_available_only_for_a_threat_at_the_current_location() {
    let mut game = crate::model::create_new_state(
        "Investigation Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    assert!(!spatial_investigation_available(&game));

    let current_location = game.character.location_id;
    game.threat.activate(
        current_location,
        "A nearby threat".to_string(),
        "Something stirs nearby.".to_string(),
    );
    assert!(spatial_investigation_available(&game));

    let other_location = game
        .world
        .locations
        .iter()
        .find(|location| location.id != current_location)
        .expect("generated world should contain another location")
        .id;
    game.threat.activate(
        other_location,
        "A distant threat".to_string(),
        "Something stirs elsewhere.".to_string(),
    );
    assert!(!spatial_investigation_available(&game));
}

#[test]
fn spatial_clock_formats_the_existing_two_hour_world_time_steps() {
    for (slot, expected) in [
        (0, "02:00"),
        (1, "04:00"),
        (2, "06:00"),
        (3, "08:00"),
        (4, "10:00"),
        (5, "12:00"),
        (6, "14:00"),
        (7, "16:00"),
        (8, "18:00"),
        (9, "20:00"),
        (10, "22:00"),
        (11, "00:00"),
        (12, "02:00"),
    ] {
        assert_eq!(crate::game::time::clock_display(slot), expected);
    }
}

#[test]
fn spatial_environment_tint_tracks_dawn_day_dusk_and_night() {
    assert!(spatial_environment_tint(1).is_some());
    assert!(spatial_environment_tint(2).is_some());
    assert!(spatial_environment_tint(3).is_none());
    assert!(spatial_environment_tint(6).is_none());
    assert!(spatial_environment_tint(7).is_some());
    assert!(spatial_environment_tint(8).is_some());
    assert!(spatial_environment_tint(9).is_some());
    assert!(spatial_environment_tint(0).is_some());
    assert!(spatial_environment_tint(21).is_some());
}

#[test]
fn spatial_pause_button_long_press_opens_console_with_spatial_return_context() {
    let mut state = SpatialExplorationState::default();
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::SpatialExploration),
        ..default()
    };

    open_spatial_developer_console(&mut state, &mut navigation);

    assert_eq!(navigation.current_screen, Some(ScreenId::Console));
    assert_eq!(navigation.return_screen, Some(ScreenId::SpatialExploration));
}
