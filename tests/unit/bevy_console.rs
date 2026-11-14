use super::*;

#[test]
fn developer_console_shortcut_is_available_from_spatial_exploration() {
    assert!(console_shortcut_screen_active(Some(
        ScreenId::SpatialExploration
    )));
    assert!(!console_shortcut_screen_active(Some(ScreenId::Lifecycle)));
    assert!(!console_shortcut_screen_active(Some(ScreenId::Character)));
    assert!(!console_shortcut_screen_active(None));
}

#[derive(Resource, Clone, Copy)]
struct ConsoleShellFixture {
    output: Entity,
    completion: Entity,
    input: Entity,
    scroll: Entity,
}

fn spawn_console_shell_fixture(mut commands: Commands) {
    let view = ConsoleView::default();
    let (output, completion, input) = spawn_console_screen(&mut commands, &view);
    commands.entity(input).insert(ConsoleInputText);
    bevy_presentation::spawn_label(&mut commands, output, "Initial output");
    let scroll = commands
        .spawn((ConsoleScrollableOutput, bevy_presentation::TouchScrollablePanel))
        .id();
    commands.insert_resource(ConsoleShellFixture {
        output,
        completion,
        input,
        scroll,
    });
    let _ = scroll;
}

fn clear_console_output_for_test(
    mut commands: Commands,
    output_hosts: Query<Entity, With<ConsoleOutputLines>>,
    children: Query<&Children>,
) {
    for host in &output_hosts {
        bevy_presentation::clear_tab_content(&mut commands, host, &children);
    }
}

#[test]
fn console_shell_separates_scrollable_output_from_the_fixed_input_control() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(spawn_console_shell_fixture);
    schedule.run(&mut world);
    world.flush();

    let fixture = *world.resource::<ConsoleShellFixture>();
    assert!(world.get::<ConsoleOutputLines>(fixture.output).is_some());
    assert!(world.get::<ConsoleCompletionLines>(fixture.completion).is_some());
    assert!(world.get::<ConsoleInputText>(fixture.input).is_some());
    assert!(world.get::<ConsoleScreenRoot>(
        world
            .query_filtered::<Entity, With<ConsoleScreenRoot>>()
            .single(&world)
            .unwrap()
    ).is_some());
    assert_eq!(
        world
            .query_filtered::<Entity, With<ConsoleScrollableOutput>>()
            .iter(&world)
            .count(),
        0
    );
    assert!(world.get::<bevy_presentation::TouchScrollablePanel>(fixture.scroll).is_some());
}

#[test]
fn console_input_edits_do_not_dirty_output_or_completion_regions() {
    let previous = ConsoleView {
        output: vec!["welcome".into()],
        ..ConsoleView::default()
    };
    let mut current = previous.clone();
    current.input = "help".into();

    assert!(!console_output_region_changed(Some(&previous), &current));
    assert!(!console_completion_region_changed(Some(&previous), &current));
    assert!(console_input_region_changed(Some(&previous), &current));
}

#[test]
fn console_completion_selection_refreshes_only_the_completion_region() {
    let previous = ConsoleView {
        output: vec!["welcome".into()],
        input: "sta".into(),
        autocomplete: true,
        candidates: vec![
            crate::presentation::ConsoleCandidateView {
                value: "status".into(),
                hint: "character/world state".into(),
            },
            crate::presentation::ConsoleCandidateView {
                value: "save".into(),
                hint: "save current game".into(),
            },
        ],
        selected: 0,
        ..ConsoleView::default()
    };
    let mut current = previous.clone();
    current.selected = 1;

    assert!(!console_output_region_changed(Some(&previous), &current));
    assert!(console_completion_region_changed(Some(&previous), &current));
    assert!(!console_input_region_changed(Some(&previous), &current));
}
