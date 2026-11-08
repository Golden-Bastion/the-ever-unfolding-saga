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
