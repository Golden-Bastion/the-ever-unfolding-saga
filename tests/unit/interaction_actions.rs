use super::*;

#[test]
fn interaction_targets_have_stable_distinct_keys() {
    assert_eq!(InteractionTarget::Npc(7).stable_key(), "npc:7");
    assert_eq!(InteractionTarget::Corpse(7).stable_key(), "corpse:7");
    assert_eq!(
        InteractionTarget::PointOfInterest("poi.shrine".into()).stable_key(),
        "poi:poi.shrine"
    );
    assert_eq!(
        InteractionTarget::Passage {
            id: "exit.north".into(),
            destination_id: 12
        }
        .stable_key(),
        "passage:exit.north:12"
    );
}

#[test]
fn discovery_filters_stale_npcs_and_unsearchable_remains() {
    let mut map = crate::spatial::generate_local_map(
        17,
        "location.interaction_filter",
        crate::spatial::LocalMapConfig::default(),
    );
    map.props.clear();
    map.points_of_interest.clear();
    map.exits.clear();

    let mut game = crate::model::create_new_state(
        "Interaction Test",
        crate::model::WorldMode::New,
        "Ash".into(),
        "Wanderer".into(),
    );
    let location_id = game.character.location_id;
    game.corpses.push(crate::model::Corpse {
        id: 701,
        former_name: "Empty Wanderer".into(),
        former_title: "Pilgrim".into(),
        location_id,
        turn_of_death: 0,
        inventory: Vec::new(),
        epitaph: String::new(),
        scavenged: false,
    });
    map.interactables = vec![
        crate::spatial::SpatialEntityPlacement {
            entity_id: 700,
            kind: crate::spatial::SpatialEntityKind::Npc,
            position: map.entry,
        },
        crate::spatial::SpatialEntityPlacement {
            entity_id: 701,
            kind: crate::spatial::SpatialEntityKind::Corpse,
            position: map.entry,
        },
    ];

    let position = (f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
    let available = discover_available_interactions(&map, &game, position);

    assert!(
        available.is_empty(),
        "stale actors and corpses without searchable inventory must not be offered",
    );
}
