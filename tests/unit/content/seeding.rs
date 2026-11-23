use super::*;
use crate::model::WorldMode;

#[test]
fn seed_world_adds_missing_content_locations_and_repairs_metadata() {
    let mut world = World::new("Test", WorldMode::New);
    let content = CampaignContent {
        version: 1,
        world: crate::content::definitions::WorldContent {
            region: crate::content::definitions::RegionContent {
                id: "region.test".into(),
                name: "Test Region".into(),
                description: "Updated".into(),
            },
            locations: vec![crate::content::definitions::LocationContent {
                id: "location.test".into(),
                name: "Test Location".into(),
                description: "Location".into(),
                dangerous: true,
                archetype: Some(crate::geography::locations::LocationArchetype::Landmark),
                exits: vec![],
                scene_art: None,
                scene_image: None,
                spatial_layout: Some("shrine_chamber".into()),
                geography: Some(crate::content::definitions::LocationGeographyContent {
                    anchor: Some(crate::geography::MacroCoordinate::new(12, 34)),
                    footprint_radius_samples: Some(2),
                }),
                spatial_discoveries: Vec::new(),
                exit_sides: Default::default(),
                exit_placements: Default::default(),
            }],
        },
        factions: vec![],
        npcs: vec![],
        character_parts: vec![],
        quests: vec![],
        encounters: vec![],
        atmospheres: vec![],
        item_visuals: vec![],
        events: vec![],
        loaded_mods: vec![],
    };

    assert_eq!(content.seed_world(&mut world), 1);
    assert_eq!(world.locations.len(), 1);
    assert_eq!(world.locations[0].name, "Test Location");
    assert!(world.locations[0].dangerous);
    assert_eq!(
        world.locations[0].spatial_layout.as_deref(),
        Some("shrine_chamber")
    );
    assert_eq!(world.regions[0].name, "Test Region");
    assert_eq!(
        world.location_geography[0].anchor,
        crate::geography::MacroCoordinate::new(12, 34)
    );
    assert_eq!(world.location_geography[0].footprint_radius_samples, 2);
    assert_eq!(
        world.location_geography[0].stable_key,
        "location.content.location.test"
    );
}
