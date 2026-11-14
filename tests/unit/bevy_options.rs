use super::*;

#[test]
fn options_tabs_have_two_categories() {
    let mut options = OptionsState::default();
    assert_eq!(options_selection_end(&options), 1); // Install and Back.
    options.tab = 1;
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
    retained_row: Entity,
}

fn spawn_options_content_hosts_fixture(mut commands: Commands) {
    let content = commands.spawn_empty().id();
    let hosts = spawn_options_content_hosts(&mut commands, content, 0);
    let retained_row = commands.spawn_empty().id();
    commands.entity(hosts.rows).add_child(retained_row);
    commands.insert_resource(OptionsContentHostsFixture {
        content,
        hosts,
        retained_row,
    });
}

fn clear_options_content_hosts_fixture(
    mut commands: Commands,
    fixture: Res<OptionsContentHostsFixture>,
    children: Query<&Children>,
) {
    clear_options_content_hosts(&mut commands, fixture.hosts, &children);
}

#[test]
fn options_auxiliary_refresh_preserves_scroll_and_dynamic_rows_host() {
    let mut world = World::new();
    let mut setup = Schedule::default();
    setup.add_systems(spawn_options_content_hosts_fixture);
    setup.run(&mut world);
    world.flush();

    let fixture = *world.resource::<OptionsContentHostsFixture>();
    world
        .entity_mut(fixture.hosts.list)
        .insert(bevy::ui::ScrollPosition(Vec2::new(0.0, 144.0)));

    let mut refresh = Schedule::default();
    refresh.add_systems(clear_options_content_hosts_fixture);
    refresh.run(&mut world);
    world.flush();

    assert!(world.get::<OptionsContent>(fixture.content).is_some());
    assert!(world
        .get::<bevy_presentation::TouchScrollablePanel>(fixture.hosts.list)
        .is_some());
    assert!(world.get::<Node>(fixture.hosts.messages).is_some());
    assert!(world.get::<Node>(fixture.hosts.footer).is_some());
    assert_eq!(
        world
            .get::<bevy::ui::ScrollPosition>(fixture.hosts.list)
            .unwrap()
            .0,
        Vec2::new(0.0, 144.0)
    );
    assert!(world
        .get::<Children>(fixture.hosts.list)
        .unwrap()
        .iter()
        .any(|child| child == fixture.hosts.rows));
    assert!(world
        .get::<Children>(fixture.hosts.rows)
        .unwrap()
        .iter()
        .any(|child| child == fixture.retained_row));
}

#[derive(Resource)]
struct ListRowsFixture(Entity);

fn spawn_list_rows_host(mut commands: Commands) {
    let host = commands.spawn(Node::default()).id();
    commands.insert_resource(ListRowsFixture(host));
}

#[derive(Resource)]
struct ModRowsFixture(Entity);

fn reconcile_test_mod_rows(
    mut commands: Commands,
    options: Res<OptionsState>,
    fixture: Res<ModRowsFixture>,
    children: Query<&Children>,
    rows: Query<(Entity, &ModListRow)>,
    empty_states: Query<(), With<OptionsEmptyState>>,
) {
    reconcile_mod_rows(
        &mut commands,
        fixture.0,
        &options,
        &children,
        &rows,
        &empty_states,
    );
}

fn test_installed_mod(id: &str, name: &str, priority: i32) -> InstalledMod {
    InstalledMod {
        manifest: crate::content::ModManifest {
            id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            priority,
            enabled: true,
            content_file: None,
        },
        directory: PathBuf::from(format!("mods/{id}")),
        enabled: true,
        priority,
        asset_count: 3,
    }
}

fn find_mod_row(world: &mut World, key: &str) -> Entity {
    let mut rows = world.query::<(Entity, &ModListRow)>();
    rows.iter(world)
        .find_map(|(entity, row)| (row.key == key).then_some(entity))
        .expect("mod row should exist")
}

#[test]
fn mod_rows_reconcile_by_stable_id_and_update_action_indices() {
    let options = OptionsState {
        mods: vec![
            test_installed_mod("mod.one", "First Mod", 1),
            test_installed_mod("mod.two", "Second Mod", 2),
        ],
        ..Default::default()
    };

    let mut world = World::new();
    world.insert_resource(options);
    let mut setup = Schedule::default();
    setup.add_systems(spawn_list_rows_host);
    setup.run(&mut world);
    world.flush();
    let host = world.resource::<ListRowsFixture>().0;
    world.insert_resource(ModRowsFixture(host));

    let mut render = Schedule::default();
    render.add_systems(reconcile_test_mod_rows);
    render.run(&mut world);
    world.flush();

    let first = find_mod_row(&mut world, "mod.one");
    let second = find_mod_row(&mut world, "mod.two");
    {
        let mut options = world.resource_mut::<OptionsState>();
        options.mods = vec![
            test_installed_mod("mod.two", "Renamed Second Mod", 1),
            test_installed_mod("mod.three", "Third Mod", 3),
        ];
        options.selected = MOD_ENABLE_ACTION;
        options.pending_remove_id = Some("mod.two".to_string());
    }

    render.run(&mut world);
    world.flush();

    let second_after = find_mod_row(&mut world, "mod.two");
    let third = find_mod_row(&mut world, "mod.three");
    assert_eq!(second_after, second);
    assert_ne!(third, first);
    assert!(world.get::<ModListRow>(first).is_none());

    let second_row = world.get::<ModListRow>(second).unwrap();
    assert_eq!(
        world
            .get::<bevy_presentation::ChoiceButton>(second_row.buttons[MOD_ENABLE_ACTION])
            .unwrap()
            .index,
        MOD_ENABLE_ACTION
    );
    assert_eq!(
        world
            .get::<bevy_presentation::ChoiceButton>(second_row.buttons[MOD_DELETE_ACTION])
            .unwrap()
            .index,
        MOD_DELETE_ACTION
    );
    let ordered = world
        .get::<Children>(host)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(ordered, vec![second, third]);
}
