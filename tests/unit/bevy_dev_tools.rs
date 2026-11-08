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
        vec!["face", "hair"]
    );
    assert_eq!(
        slots[1]
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

    slots[0].visible = false;
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
fn world_lab_generation_is_reproducible_and_respects_selected_layout() {
    let options = WorldLabOptions {
        seed: 7_654_321,
        size_index: 2,
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
    let preview = world_lab_map_preview_with_cursor(&map, Some(cursor), Some(&held_id));
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
