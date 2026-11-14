use super::*;

#[test]
fn interaction_screen_set_matches_navigation_targets() {
    assert!(is_interaction_screen(Some(ScreenId::Talk)));
    assert!(is_interaction_screen(Some(ScreenId::Conversation)));
    assert!(is_interaction_screen(Some(ScreenId::RemainsResult)));
    assert!(!is_interaction_screen(Some(ScreenId::Lifecycle)));
    assert!(!is_interaction_screen(Some(ScreenId::SpatialExploration)));
}

#[test]
fn spatial_interaction_overlay_keeps_the_exploration_screen_active() {
    let mut state = BevyInteractionState::default();
    assert!(!spatial_overlay_open(&state));
    state.spatial_overlay = true;

    assert!(spatial_overlay_open(&state));
    assert!(!is_interaction_screen(Some(ScreenId::SpatialExploration)));
}

#[test]
fn portrait_layers_are_composited_in_stable_draw_order() {
    let portrait = CharacterPortraitView {
        layers: vec![
            CharacterPortraitLayerView {
                id: "body".to_string(),
                slot: "body".to_string(),
                asset: "body.png".to_string(),
                layer: 3,
            },
            CharacterPortraitLayerView {
                id: "hair-z".to_string(),
                slot: "hair".to_string(),
                asset: "hair-z.png".to_string(),
                layer: 1,
            },
            CharacterPortraitLayerView {
                id: "face-b".to_string(),
                slot: "face".to_string(),
                asset: "face-b.png".to_string(),
                layer: 1,
            },
            CharacterPortraitLayerView {
                id: "face-a".to_string(),
                slot: "face".to_string(),
                asset: "face-a.png".to_string(),
                layer: 1,
            },
        ],
    };

    let ordered = ordered_portrait_layers(&portrait)
        .into_iter()
        .map(|layer| layer.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ordered, vec!["face-a", "face-b", "hair-z", "body"]);
}

#[test]
fn character_slot_controls_align_with_their_portrait_regions() {
    assert_eq!(portrait_slot_anchor("headgear", 0, 5), 7.0);
    assert_eq!(portrait_slot_anchor("hair", 1, 5), 20.0);
    assert_eq!(portrait_slot_anchor("face", 2, 5), 34.0);
    assert_eq!(portrait_slot_anchor("accessory", 3, 5), 53.0);
    assert_eq!(portrait_slot_anchor("body", 4, 5), 72.0);
    assert_eq!(portrait_slot_anchor("custom-layer", 0, 2), 27.5);
    assert_eq!(portrait_slot_anchor("custom-layer", 1, 2), 66.5);
    assert_eq!(portrait_slot_anchor("custom-layer", 0, 0), 50.0);
}

#[test]
fn closing_spatial_dialogue_keeps_movement_suspended_during_transition() {
    let mut state = BevyInteractionState::default();
    let mut navigation = NavigationState::default();
    state.spatial_overlay = true;

    close_spatial_overlay(&mut state, &mut navigation);

    assert!(state.spatial_overlay_closing);
    assert!(spatial_overlay_open(&state));
    assert!(!state.dirty);
}

#[derive(Resource, Clone, Copy)]
struct InteractionShellFixture(Entity);

fn spawn_interaction_shell_fixture(mut commands: Commands) {
    let host = spawn_interaction_screen(&mut commands, InteractionScreen::Talk);
    bevy_presentation::spawn_label(&mut commands, host, "Initial interaction content");
    commands.insert_resource(InteractionShellFixture(host));
}

fn clear_interaction_content_for_test(
    mut commands: Commands,
    hosts: Query<Entity, With<InteractionContentHost>>,
    children: Query<&Children>,
) {
    for host in &hosts {
        bevy_presentation::clear_content_host(&mut commands, host, &children);
    }
}

#[test]
fn interaction_content_refresh_preserves_the_screen_root_and_scroll_position() {
    let mut world = World::new();
    let mut setup = Schedule::default();
    setup.add_systems(spawn_interaction_shell_fixture);
    setup.run(&mut world);
    world.flush();

    let host = world.resource::<InteractionShellFixture>().0;
    let root = world
        .query_filtered::<Entity, With<InteractionRoot>>()
        .iter(&world)
        .next()
        .expect("interaction root should be mounted");
    let panel = world
        .query_filtered::<Entity, With<InteractionScrollPanel>>()
        .iter(&world)
        .next()
        .expect("interaction scroll panel should be mounted");
    world
        .entity_mut(panel)
        .insert(bevy::ui::ScrollPosition(Vec2::new(0.0, 142.0)));

    let mut clear = Schedule::default();
    clear.add_systems(clear_interaction_content_for_test);
    clear.run(&mut world);
    world.flush();

    assert!(world.get::<InteractionRoot>(root).is_some());
    assert!(world.get::<InteractionContentHost>(host).is_some());
    assert!(world.get::<InteractionScrollPanel>(panel).is_some());
    assert_eq!(
        world.get::<bevy::ui::ScrollPosition>(panel).unwrap().0,
        Vec2::new(0.0, 142.0)
    );
    assert!(world
        .get::<Children>(host)
        .is_none_or(|children| children.is_empty()));
}
