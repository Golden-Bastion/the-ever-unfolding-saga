use super::*;
use crate::content::CharacterPartContent;

fn part(
    id: &str,
    slot: &str,
    weight: u32,
    layer: i32,
    requires: &[&str],
    excludes: &[&str],
) -> CharacterPartContent {
    CharacterPartContent {
        id: id.into(),
        slot: slot.into(),
        asset: format!("characters/parts/{id}.png"),
        weight,
        layer,
        requires_tags: requires.iter().map(|tag| (*tag).into()).collect(),
        excludes_tags: excludes.iter().map(|tag| (*tag).into()).collect(),
    }
}

#[test]
fn actor_seed_and_portrait_are_reproducible() {
    let parts = vec![
        part("face.one", "face", 1, 10, &[], &[]),
        part("hair.one", "hair", 1, 20, &[], &[]),
    ];
    let seed = actor_appearance_seed(42, 123);
    assert_eq!(seed, actor_appearance_seed(42, 123));
    assert_eq!(
        generate_character_portrait(&parts, seed, &[]),
        generate_character_portrait(&parts, seed, &[])
    );
}

#[test]
fn portrait_layers_follow_explicit_draw_order() {
    let parts = vec![
        part("hair.one", "hair", 1, 30, &[], &[]),
        part("face.one", "face", 1, 10, &[], &[]),
    ];
    let chosen = generate_character_portrait(&parts, 7, &[]);
    assert_eq!(
        chosen.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
        vec!["face.one", "hair.one"]
    );
}

#[test]
fn compatibility_tags_filter_candidates() {
    let parts = vec![
        part("head.bearded", "head", 10, 10, &["bearded"], &[]),
        part("head.clean", "head", 10, 10, &[], &["bearded"]),
    ];
    let tags = vec!["bearded".to_string()];
    let chosen = generate_character_portrait(&parts, 12, &tags);
    assert_eq!(chosen.len(), 1);
    assert_eq!(chosen[0].id, "head.bearded");
}

#[test]
fn higher_weight_parts_are_selected_more_often() {
    let parts = vec![
        part("rare", "face", 1, 10, &[], &[]),
        part("common", "face", 9, 10, &[], &[]),
    ];
    let mut rare = 0;
    let mut common = 0;
    for seed in 0..500 {
        match generate_character_portrait(&parts, seed, &[])
            .first()
            .map(|p| p.id.as_str())
        {
            Some("rare") => rare += 1,
            Some("common") => common += 1,
            _ => panic!("one compatible part should be selected"),
        }
    }
    assert!(common > rare * 3, "common={common}, rare={rare}");
}

#[test]
fn incompatible_slots_are_omitted_gracefully() {
    let parts = vec![
        part("face.restricted", "face", 1, 10, &["royal"], &[]),
        part("hair.one", "hair", 1, 20, &[], &[]),
    ];
    let chosen = generate_character_portrait(&parts, 5, &[]);
    assert_eq!(chosen.len(), 1);
    assert_eq!(chosen[0].id, "hair.one");
}

#[test]
fn player_portrait_uses_explicit_part_ids_in_draw_order() {
    let parts = vec![
        part("hair.default", "hair", 1, 20, &[], &[]),
        part("face.default", "face", 1, 10, &[], &[]),
        part("face.player", "face", 1, 10, &[], &[]),
        part("hair.player", "hair", 1, 20, &[], &[]),
    ];
    let chosen = resolve_character_portrait(
        &parts,
        &["hair.player".to_string(), "face.player".to_string()],
    );
    assert_eq!(
        chosen.iter().map(|layer| layer.id.as_str()).collect::<Vec<_>>(),
        vec!["face.player", "hair.player"]
    );
}

#[test]
fn player_portrait_defaults_and_missing_parts_are_stable() {
    let parts = vec![
        part("hair.z", "hair", 1, 20, &[], &[]),
        part("face.z", "face", 1, 10, &[], &[]),
        part("hair.a", "hair", 0, 20, &[], &[]),
        part("face.a", "face", 1, 10, &[], &[]),
        part("face.tagged", "face", 1, 10, &["royal"], &[]),
    ];
    let defaults = default_character_appearance_ids(&parts);
    assert_eq!(defaults, vec!["face.a", "hair.a"]);
    assert_eq!(
        resolve_character_portrait(&parts, &[]),
        resolve_character_portrait(&parts, &["removed.mod.part".to_string()])
    );
    assert_eq!(
        resolve_character_portrait(&parts, &defaults)
            .iter()
            .map(|layer| layer.id.as_str())
            .collect::<Vec<_>>(),
        vec!["face.a", "hair.a"]
    );
}
