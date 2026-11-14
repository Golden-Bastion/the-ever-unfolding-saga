use super::*;

#[test]
fn character_tabs_have_a_stable_five_tab_order() {
    assert_eq!(CharacterTab::ALL.map(CharacterTab::index), [0, 1, 2, 3, 4]);
    assert_eq!(CharacterTab::from_index(0), Some(CharacterTab::General));
    assert_eq!(CharacterTab::from_index(4), Some(CharacterTab::Journal));
    assert_eq!(CharacterTab::from_index(5), None);
    assert_eq!(CharacterTab::Journal.next(), CharacterTab::General);
}

#[test]
fn closing_records_restores_spatial_context() {
    let mut navigation = NavigationState {
        current_screen: Some(ScreenId::Character),
        return_screen: Some(ScreenId::SpatialExploration),
        selected: 2,
    };

    close_to_return_screen(&mut navigation);

    assert_eq!(
        navigation.current_screen,
        Some(ScreenId::SpatialExploration)
    );
    assert_eq!(navigation.return_screen, None);
    assert_eq!(navigation.selected, 0);
}

#[test]
fn journal_does_not_repeat_the_note_as_a_chronicle_entry() {
    let notes = vec!["I should remember this.".to_string()];

    assert!(is_personal_note_history_entry(
        "Ada noted: I should remember this.",
        "Ada",
        &notes
    ));
    assert!(!is_personal_note_history_entry(
        "Ada noted: a different note.",
        "Ada",
        &notes
    ));
    assert!(!is_personal_note_history_entry(
        "Ada found a sealed letter.",
        "Ada",
        &notes
    ));
}

#[test]
fn character_tab_back_action_follows_tab_content() {
    assert_eq!(character_tab_back_index(CharacterTab::General, 3, 2, 4), 0);
    assert_eq!(
        character_tab_selection_count(CharacterTab::Reputation, 3, 2, 4),
        1
    );
    assert_eq!(
        character_tab_back_index(CharacterTab::Inventory, 3, 2, 4),
        3
    );
    assert_eq!(
        character_tab_selection_count(CharacterTab::Quests, 3, 2, 4),
        3
    );
    assert_eq!(character_tab_back_index(CharacterTab::Journal, 3, 2, 4), 5);
    assert_eq!(
        character_tab_selection_count(CharacterTab::Journal, 3, 2, 4),
        6
    );
}

#[test]
fn all_character_tabs_return_to_the_spatial_invoking_screen() {
    for tab in CharacterTab::ALL {
        let mut navigation = NavigationState {
            current_screen: Some(ScreenId::SpatialExploration),
            return_screen: None,
            selected: 3,
        };
        navigation.open_screen(ScreenId::Character);

        let mut records_state = RecordScreenState {
            active_navigation: Some(ScreenId::Character),
            screen: RecordScreen::Character,
            character_tab: tab,
            ..Default::default()
        };

        assert_eq!(navigation.return_screen, Some(ScreenId::SpatialExploration));
        cancel(&mut navigation, &mut records_state);

        assert_eq!(
            navigation.current_screen,
            Some(ScreenId::SpatialExploration)
        );
        assert_eq!(navigation.return_screen, None);
    }
}

#[test]
fn character_detail_and_journal_editor_back_paths_keep_spatial_parent() {
    for screen in [
        RecordScreen::InventoryDetail,
        RecordScreen::QuestDetail,
        RecordScreen::ChronicleDetail,
        RecordScreen::JournalEntry,
    ] {
        let mut navigation = NavigationState {
            current_screen: Some(ScreenId::Character),
            return_screen: Some(ScreenId::SpatialExploration),
            selected: 0,
        };
        let mut records_state = RecordScreenState {
            active_navigation: Some(ScreenId::Character),
            screen,
            ..Default::default()
        };

        cancel(&mut navigation, &mut records_state);

        assert_eq!(navigation.current_screen, Some(ScreenId::Character));
        assert_eq!(navigation.return_screen, Some(ScreenId::SpatialExploration));

        records_state.screen = RecordScreen::Character;
        cancel(&mut navigation, &mut records_state);
        assert_eq!(
            navigation.current_screen,
            Some(ScreenId::SpatialExploration)
        );
        assert_eq!(navigation.return_screen, None);
    }
}

#[derive(Resource, Clone, Copy)]
struct RecordDetailShellFixture {
    root: Entity,
    content: Entity,
}

fn spawn_record_detail_shell_fixture(mut commands: Commands) {
    let (root, content) = spawn_record_detail_shell(&mut commands, RecordScreen::InventoryDetail);
    bevy_presentation::spawn_label(&mut commands, content, "Initial detail content");
    commands.insert_resource(RecordDetailShellFixture { root, content });
}

fn clear_record_detail_content(
    mut commands: Commands,
    hosts: Query<Entity, With<RecordDetailContentHost>>,
    children: Query<&Children>,
) {
    for host in &hosts {
        bevy_presentation::clear_content_host(&mut commands, host, &children);
    }
}

#[test]
fn record_detail_content_refresh_preserves_root_and_scroll_panel_state() {
    let mut world = World::new();
    let mut setup = Schedule::default();
    setup.add_systems(spawn_record_detail_shell_fixture);
    setup.run(&mut world);
    world.flush();

    let fixture = *world.resource::<RecordDetailShellFixture>();
    assert!(world.get::<BevyScreenRoot>(fixture.root).is_some());
    assert_eq!(
        world
            .get::<RecordDetailScreen>(fixture.root)
            .map(|detail| detail.screen),
        Some(RecordScreen::InventoryDetail)
    );
    assert!(world
        .get::<RecordDetailContentHost>(fixture.content)
        .is_some());
    assert!(world
        .get::<bevy_presentation::TouchScrollablePanel>(fixture.content)
        .is_some());
    world
        .entity_mut(fixture.content)
        .insert(bevy::ui::ScrollPosition(Vec2::new(0.0, 116.0)));

    let mut clear = Schedule::default();
    clear.add_systems(clear_record_detail_content);
    clear.run(&mut world);
    world.flush();

    assert!(world.get::<RecordDetailScreen>(fixture.root).is_some());
    assert!(world
        .get::<RecordDetailContentHost>(fixture.content)
        .is_some());
    assert_eq!(
        world
            .get::<bevy::ui::ScrollPosition>(fixture.content)
            .unwrap()
            .0,
        Vec2::new(0.0, 116.0)
    );
    assert!(world
        .get::<Children>(fixture.content)
        .is_none_or(|children| children.is_empty()));
}

#[test]
fn record_detail_screens_keep_distinct_screen_titles() {
    assert_eq!(
        record_detail_title(RecordScreen::InventoryDetail),
        "INVENTORY | ITEM"
    );
    assert_eq!(
        record_detail_title(RecordScreen::QuestDetail),
        "QUEST | DETAILS"
    );
    assert_eq!(record_detail_title(RecordScreen::Meditation), "MEDITATION");
    assert_eq!(
        record_detail_title(RecordScreen::MeditationResult),
        "MEDITATION | COMPLETE"
    );
    assert_eq!(
        record_detail_title(RecordScreen::ChronicleDetail),
        "JOURNAL | CHRONICLE"
    );
    assert_eq!(
        record_detail_title(RecordScreen::JournalEntry),
        "WRITE IN JOURNAL"
    );
}

#[derive(Resource)]
struct InventoryRowsFixture {
    host: Entity,
    view: InventoryView,
    selected: usize,
}

fn spawn_inventory_rows_fixture(mut commands: Commands) {
    let parent = commands.spawn_empty().id();
    let host = bevy_presentation::spawn_action_button_group(&mut commands, parent);
    commands.insert_resource(InventoryRowsFixture {
        host,
        view: InventoryView {
            items: vec![
                crate::presentation::ItemView {
                    id: 101,
                    name: "Iron sword".to_string(),
                    description: String::new(),
                },
                crate::presentation::ItemView {
                    id: 102,
                    name: "Wooden shield".to_string(),
                    description: String::new(),
                },
            ],
            ..Default::default()
        },
        selected: 0,
    });
}

fn render_inventory_rows_fixture(
    mut commands: Commands,
    fixture: Res<InventoryRowsFixture>,
    children: Query<&Children>,
    rows: Query<(Entity, &CharacterListRow)>,
    empty_states: Query<(), With<CharacterEmptyState>>,
) {
    render_inventory_tab(
        &mut commands,
        fixture.host,
        &fixture.view,
        fixture.selected,
        &children,
        &rows,
        &empty_states,
    );
}

#[derive(Resource)]
struct QuestRowsFixture {
    host: Entity,
    view: QuestLogView,
    selected: usize,
}

fn spawn_quest_rows_fixture(mut commands: Commands) {
    let parent = commands.spawn_empty().id();
    let host = bevy_presentation::spawn_action_button_group(&mut commands, parent);
    commands.insert_resource(QuestRowsFixture {
        host,
        view: QuestLogView {
            quests: vec![QuestView {
                id: 701,
                title: "Find the sealed letter".to_string(),
                status: "ACTIVE".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        selected: 0,
    });
}

fn render_quest_rows_fixture(
    mut commands: Commands,
    fixture: Res<QuestRowsFixture>,
    children: Query<&Children>,
    rows: Query<(Entity, &CharacterListRow)>,
    empty_states: Query<(), With<CharacterEmptyState>>,
) {
    render_quests_tab(
        &mut commands,
        fixture.host,
        &fixture.view,
        fixture.selected,
        &children,
        &rows,
        &empty_states,
    );
}

fn find_character_row(world: &mut World, key: &str) -> Entity {
    let mut rows = world.query::<(Entity, &CharacterListRow)>();
    rows.iter(world)
        .find_map(|(entity, row)| (row.key == key).then_some(entity))
        .expect("character list row should exist")
}

#[test]
fn inventory_rows_reconcile_by_item_identity_and_reorder_in_place() {
    let mut world = World::new();
    let mut setup = Schedule::default();
    setup.add_systems(spawn_inventory_rows_fixture);
    setup.run(&mut world);
    world.flush();

    let mut render = Schedule::default();
    render.add_systems(render_inventory_rows_fixture);
    render.run(&mut world);
    world.flush();

    let first = find_character_row(&mut world, "inventory:101");
    let second = find_character_row(&mut world, "inventory:102");
    let host = world.resource::<InventoryRowsFixture>().host;

    {
        let mut fixture = world.resource_mut::<InventoryRowsFixture>();
        fixture.view.items = vec![
            crate::presentation::ItemView {
                id: 102,
                name: "Repaired wooden shield".to_string(),
                description: String::new(),
            },
            crate::presentation::ItemView {
                id: 103,
                name: "Torch".to_string(),
                description: String::new(),
            },
        ];
        fixture.selected = 1;
    }

    render.run(&mut world);
    world.flush();

    let second_after = find_character_row(&mut world, "inventory:102");
    let third = find_character_row(&mut world, "inventory:103");
    assert_eq!(second_after, second);
    assert_ne!(third, first);
    assert!(world.get::<CharacterListRow>(first).is_none());
    assert_eq!(
        world
            .get::<bevy_presentation::ChoiceButton>(third)
            .unwrap()
            .index,
        1
    );
    assert!(world.get::<bevy_presentation::UiSelected>(third).is_some());
    let ordered = world
        .get::<Children>(host)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(ordered, vec![second, third]);
}

#[test]
fn quest_rows_use_persistent_quest_ids_for_retention() {
    let mut world = World::new();
    let mut setup = Schedule::default();
    setup.add_systems(spawn_quest_rows_fixture);
    setup.run(&mut world);
    world.flush();

    let mut render = Schedule::default();
    render.add_systems(render_quest_rows_fixture);
    render.run(&mut world);
    world.flush();

    let original = find_character_row(&mut world, "quest:701");
    {
        let mut fixture = world.resource_mut::<QuestRowsFixture>();
        fixture.view.quests[0].title = "Find the sealed letter and return it".to_string();
        fixture.view.quests[0].status = "READY".to_string();
        fixture.selected = 0;
    }

    render.run(&mut world);
    world.flush();

    assert_eq!(find_character_row(&mut world, "quest:701"), original);
    let row = world.get::<CharacterListRow>(original).unwrap();
    assert!(world.get::<Text>(row.label).is_some());
    assert!(world.get::<bevy_presentation::UiSelected>(original).is_some());
}
