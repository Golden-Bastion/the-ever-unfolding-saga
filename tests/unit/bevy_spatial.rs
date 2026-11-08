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
        ("spatial/player_ash_warden.png", 16, 24),
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
    let map = generate_local_map(4, "location.test", LocalMapConfig::default());
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
                destination_id: 999,
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
