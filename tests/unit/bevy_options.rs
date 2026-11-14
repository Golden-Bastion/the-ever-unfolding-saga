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
struct SaveRowsFixture(Entity);

fn spawn_save_rows_host(mut commands: Commands) {
    let host = commands.spawn(Node::default()).id();
    commands.insert_resource(SaveRowsFixture(host));
}

fn reconcile_test_save_rows(
    mut commands: Commands,
    options: Res<OptionsState>,
    fixture: Res<SaveRowsFixture>,
    children: Query<&Children>,
    rows: Query<(Entity, &SaveListRow)>,
    empty_states: Query<(), With<OptionsEmptyState>>,
) {
    reconcile_save_rows(
        &mut commands,
        fixture.0,
        &options,
        &children,
        &rows,
        &empty_states,
    );
}

fn find_save_row(world: &mut World, key: &PathBuf) -> Entity {
    let mut rows = world.query::<(Entity, &SaveListRow)>();
    rows.iter(world)
        .find_map(|(entity, row)| (&row.key == key).then_some(entity))
        .expect("save row should exist")
}

#[test]
fn save_rows_reconcile_by_path_and_preserve_entities_when_reordered() {
    let first = PathBuf::from("saves/first.json.gz");
    let second = PathBuf::from("saves/second.json.gz");
    let third = PathBuf::from("saves/third.json.gz");
    let mut options = OptionsState::default();
    options.saves = vec![
        SaveEntry {
            path: first.clone(),
            metadata: None,
        },
        SaveEntry {
            path: second.clone(),
            metadata: None,
        },
    ];

    let mut world = World::new();
    world.insert_resource(options);
    let mut setup = Schedule::default();
    setup.add_systems(spawn_save_rows_host);
    setup.run(&mut world);
    world.flush();
    let host = world.resource::<SaveRowsFixture>().0;

    let mut render = Schedule::default();
    render.add_systems(reconcile_test_save_rows);
    render.run(&mut world);
    world.flush();

    let first_entity = find_save_row(&mut world, &first);
    let second_entity = find_save_row(&mut world, &second);

    {
        let mut options = world.resource_mut::<OptionsState>();
        options.saves = vec![
            SaveEntry {
                path: second.clone(),
                metadata: None,
            },
            SaveEntry {
                path: third.clone(),
                metadata: None,
            },
        ];
    }

    render.run(&mut world);
    world.flush();

    let second_after = find_save_row(&mut world, &second);
    let third_entity = find_save_row(&mut world, &third);
    assert_eq!(second_after, second_entity);
    assert_ne!(third_entity, first_entity);
    assert!(world.get::<SaveListRow>(first_entity).is_none());

    let ordered = world
        .get::<Children>(host)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(ordered, vec![second_entity, third_entity]);
    let second_row = world.get::<SaveListRow>(second_entity).unwrap();
    assert_eq!(
        world
            .get::<bevy_presentation::ChoiceButton>(second_row.button)
            .unwrap()
            .index,
        0
    );
}
