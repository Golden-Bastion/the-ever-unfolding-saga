use super::*;

#[test]
fn options_tabs_have_three_categories() {
    let mut options = OptionsState::default();
    assert_eq!(options_selection_end(&options), 1); // Import and Back.
    options.tab = 1;
    assert_eq!(options_selection_end(&options), 1); // Install and Back.
    options.tab = 2;
    assert_eq!(options_selection_end(&options), 5);
}

#[test]
fn mod_action_slots_match_the_row_control_order() {
    let second_mod_first_action = mod_action_start_index(1);
    assert_eq!(MOD_ENABLE_ACTION, 0);
    assert_eq!(MOD_DELETE_ACTION, 1);
    assert_eq!(MOD_PRIORITY_UP_ACTION, 2);
    assert_eq!(MOD_PRIORITY_DOWN_ACTION, 3);
    assert_eq!(second_mod_first_action, 4);
    assert_eq!(mod_install_action_index(2), 8);
    assert_eq!(mod_back_action_index(2), 9);
    assert_eq!(
        second_mod_first_action + MOD_PRIORITY_DOWN_ACTION,
        second_mod_first_action + MOD_ACTIONS_PER_MOD - 1
    );
}

#[derive(Resource, Clone, Copy)]
struct OptionsContentHostsFixture {
    content: Entity,
    hosts: OptionsContent,
}

fn spawn_options_content_hosts_fixture(mut commands: Commands) {
    let content = commands.spawn_empty().id();
    let hosts = spawn_options_content_hosts(&mut commands, content, 0);
    commands.insert_resource(OptionsContentHostsFixture { content, hosts });
}

fn clear_options_content_hosts_fixture(
    mut commands: Commands,
    fixture: Res<OptionsContentHostsFixture>,
    children: Query<&Children>,
) {
    clear_options_content_hosts(&mut commands, fixture.hosts, &children);
}

#[test]
fn options_refresh_preserves_scroll_host_and_all_content_regions() {
    let mut world = World::new();
    let mut setup = Schedule::default();
    setup.add_systems(spawn_options_content_hosts_fixture);
    setup.run(&mut world);
    world.flush();

    let fixture = *world.resource::<OptionsContentHostsFixture>();
    world.entity_mut(fixture.hosts.list)
        .insert(bevy::ui::ScrollPosition(Vec2::new(0.0, 144.0)));

    let mut refresh = Schedule::default();
    refresh.add_systems(clear_options_content_hosts_fixture);
    refresh.run(&mut world);
    world.flush();

    assert!(world.get::<OptionsContent>(fixture.content).is_some());
    assert!(world.get::<bevy_presentation::TouchScrollablePanel>(fixture.hosts.list).is_some());
    assert!(world.get::<Node>(fixture.hosts.messages).is_some());
    assert!(world.get::<Node>(fixture.hosts.footer).is_some());
    assert_eq!(
        world.get::<bevy::ui::ScrollPosition>(fixture.hosts.list).unwrap().0,
        Vec2::new(0.0, 144.0)
    );
    assert!(world.get::<Children>(fixture.hosts.list).is_none_or(|children| children.is_empty()));
}
