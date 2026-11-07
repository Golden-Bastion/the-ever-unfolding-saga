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
                exits: vec![],
                scene_art: None,
                scene_image: None,
                spatial_layout: Some("shrine_chamber".into()),
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
}
