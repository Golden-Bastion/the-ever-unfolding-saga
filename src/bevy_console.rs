use bevy::input::keyboard::KeyboardInput;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::bevy_lifecycle::LifecycleState;
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId, TextContent,
    UiButtonActivation,
};
use crate::game::console::ConsoleSession;
use crate::input::InputEvent;
use crate::presentation::{ConsoleScrollView, ConsoleView};

#[cfg(target_os = "android")]
#[derive(Resource, Default)]
struct AndroidConsoleTextSync {
    active: bool,
    last_sent_text: String,
}

#[derive(Component)]
struct ConsoleCloseButton;

#[derive(Resource)]
pub(crate) struct BevyConsoleState {
    pub(crate) console: ConsoleSession,
    dirty: bool,
}

impl Default for BevyConsoleState {
    fn default() -> Self {
        Self {
            console: ConsoleSession::default(),
            dirty: true,
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<BevyConsoleState>().add_systems(
        Update,
        (
            open_shortcut,
            text_input,
            close_button_input,
            console_input,
            render_if_active,
        )
            .chain(),
    );

    #[cfg(target_os = "android")]
    app.init_resource::<AndroidConsoleTextSync>()
        .add_systems(Update, sync_android_console_text_input);
}

fn console_shortcut_screen_active(screen: Option<ScreenId>) -> bool {
    screen == Some(ScreenId::SpatialExploration)
}

fn open_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    lifecycle: Res<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
) {
    if lifecycle.phase == crate::bevy_lifecycle::LifecyclePhase::Complete
        && lifecycle.session.is_some()
        && console_shortcut_screen_active(navigation.current_screen)
        && keyboard.just_pressed(KeyCode::Slash)
    {
        navigation.open_screen(ScreenId::Console);
    }
}

#[cfg(target_os = "android")]
fn sync_android_console_text_input(
    navigation: Res<NavigationState>,
    mut state: ResMut<BevyConsoleState>,
    mut sync: ResMut<AndroidConsoleTextSync>,
    mut console_focus: ResMut<bevy_presentation::ConsoleTextInputFocus>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if navigation.current_screen != Some(ScreenId::Console) {
        if sync.active {
            bevy_presentation::android_set_text_input_target(None, "");
        }
        sync.active = false;
        sync.last_sent_text.clear();
        console_focus.active = false;
        return;
    }

    if !console_focus.active {
        if sync.active {
            bevy_presentation::android_set_text_input_target(None, "");
            sync.active = false;
            sync.last_sent_text.clear();
        }
        return;
    }

    let current = state.console.view().input;

    if !sync.active {
        sync.active = true;
        sync.last_sent_text = current.clone();
        bevy_presentation::android_set_text_input_target(Some(1), &sync.last_sent_text);
        return;
    }

    for event in bevy_presentation::android_take_text_input_events(1) {
        match event {
            bevy_presentation::AndroidTextInputEvent::Changed(text) => {
                let text = text
                    .chars()
                    .filter(|character| !character.is_control())
                    .collect::<String>();
                if text != current {
                    state.console.set_text(&text);
                    state.dirty = true;
                }
                sync.last_sent_text = text;
            }
            bevy_presentation::AndroidTextInputEvent::Submitted => {
                input_queue.0.push(InputEvent::Confirm);
            }
            bevy_presentation::AndroidTextInputEvent::Dismissed => {
                console_focus.active = false;
                sync.active = false;
                sync.last_sent_text.clear();
            }
        }
    }
}

fn text_input(
    mut keyboard: MessageReader<KeyboardInput>,
    navigation: Res<NavigationState>,
    mut state: ResMut<BevyConsoleState>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || state.console.is_autocomplete() {
        return;
    }
    for event in keyboard.read() {
        if let Some(text) = &event.text {
            state.console.push_text(text);
            state.dirty = true;
        }
    }
}

fn close_button_input(
    mut activations: MessageReader<UiButtonActivation>,
    close_buttons: Query<(), With<ConsoleCloseButton>>,
    mut state: ResMut<BevyConsoleState>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
) {
    let mut should_close = false;
    for activation in activations.read() {
        should_close |= close_buttons.contains(activation.0);
    }
    if navigation.current_screen != Some(ScreenId::Console) || !should_close {
        return;
    }

    if let Some(session) = lifecycle.session.as_mut() {
        crate::game::console::bootstrap_after_console(&mut session.state);
    }
    leave_console(&mut state, &mut navigation);
}

fn console_input(
    mut state: ResMut<BevyConsoleState>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || input_queue.0.is_empty() {
        return;
    }
    let events = std::mem::take(&mut input_queue.0);
    let Some(session) = lifecycle.session.as_mut() else {
        leave_console(&mut state, &mut navigation);
        return;
    };

    for event in events {
        match event {
            InputEvent::Cancel => {
                if state.console.is_autocomplete() {
                    state.console.cancel_completion();
                    state.dirty = true;
                } else {
                    crate::game::console::bootstrap_after_console(&mut session.state);
                    leave_console(&mut state, &mut navigation);
                    break;
                }
            }
            InputEvent::Confirm => {
                if state.console.is_autocomplete() {
                    state.console.accept_completion();
                } else if let Err(error) = state
                    .console
                    .execute_line(&mut session.state, &session.save_path)
                {
                    state
                        .console
                        .output_error(&format!("Command failed: {error}"));
                }
                if state.console.should_exit() {
                    crate::game::console::bootstrap_after_console(&mut session.state);
                    leave_console(&mut state, &mut navigation);
                    break;
                }
                state.dirty = true;
            }
            InputEvent::Tab => {
                if state.console.is_autocomplete() {
                    state.console.accept_completion();
                } else {
                    state.console.start_completion(&session.state);
                }
                state.dirty = true;
            }
            InputEvent::Up => {
                if state.console.is_autocomplete() {
                    state.console.select_previous_completion();
                } else {
                    state.console.history_previous();
                }
                state.dirty = true;
            }
            InputEvent::Down => {
                if state.console.is_autocomplete() {
                    state.console.select_next_completion();
                } else {
                    state.console.history_next();
                }
                state.dirty = true;
            }
            InputEvent::Home => {
                state.console.jump_home();
                state.dirty = true;
            }
            InputEvent::End => {
                state.console.jump_end();
                state.dirty = true;
            }
            InputEvent::PageUp => {
                state.console.scroll_up(6);
                state.dirty = true;
            }
            InputEvent::PageDown => {
                state.console.scroll_down(6);
                state.dirty = true;
            }
            InputEvent::Backspace => {
                state.console.edit(InputEvent::Backspace);
                state.dirty = true;
            }
            InputEvent::Delete => {
                state.console.edit(InputEvent::Delete);
                state.dirty = true;
            }
            InputEvent::Character(_) => {}
            InputEvent::SelectTab(_) => {}
            InputEvent::OpenSecondaryNavigation | InputEvent::OpenDeveloperConsole => {}
        }
    }
}

fn leave_console(state: &mut BevyConsoleState, navigation: &mut NavigationState) {
    navigation.return_to_previous_screen(ScreenId::SpatialExploration);
    state.console = ConsoleSession::default();
    state.dirty = true;
}

fn render_if_active(
    mut commands: Commands,
    mut state: ResMut<BevyConsoleState>,
    navigation: Res<NavigationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || !state.dirty {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    let view = state.console.view();
    render(&mut commands, &view);
    state.dirty = false;
}

fn render(commands: &mut Commands, view: &ConsoleView) {
    let root = bevy_presentation::spawn_screen(commands, "DEVELOPER CONSOLE");
    spawn_console_close_button(commands, root);
    let panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "Enter commands directly. Esc closes the console.",
    );
    for line in visible_output(view) {
        bevy_presentation::spawn_label(commands, panel, line);
    }
    bevy_presentation::spawn_console_input_row(commands, panel, format!("> {}", view.input));
    if view.autocomplete && !view.candidates.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "Completions:");
        let visible = view.candidates.len().min(8);
        let start = view
            .completion_scroll
            .min(view.candidates.len().saturating_sub(visible));
        for (index, candidate) in view.candidates.iter().enumerate().skip(start).take(visible) {
            let marker = if index == view.selected { ">" } else { " " };
            bevy_presentation::spawn_label(
                commands,
                panel,
                format!("{marker} {} - {}", candidate.value, candidate.hint),
            );
        }
    }
}

fn spawn_console_close_button(commands: &mut Commands, root: Entity) {
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            bevy_presentation::UiTouchTargetShape::MinimumHeight,
            ConsoleCloseButton,
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                right: px(0),
                width: px(80),
                min_width: px(80),
                min_height: px(48),
                padding: UiRect::axes(px(10), px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_ACCENT),
            BackgroundColor(bevy_presentation::THEME_PANEL_ALT),
            ZIndex(1),
            children![(
                Text::new("Close"),
                TextContent,
                TextFont::from_font_size(bevy_presentation::muted_font_size()),
                TextColor(bevy_presentation::THEME_TEXT),
            )],
        ))
        .id();
    commands.entity(root).add_child(button);
}

fn visible_output(view: &ConsoleView) -> Vec<String> {
    const VISIBLE_LINES: usize = 24;
    if view.output.len() <= VISIBLE_LINES {
        return view.output.clone();
    }
    match view.scroll {
        ConsoleScrollView::Follow | ConsoleScrollView::Offset(0) => {
            view.output[view.output.len() - VISIBLE_LINES..].to_vec()
        }
        ConsoleScrollView::Offset(offset) => {
            let end = view.output.len().saturating_sub(offset);
            let start = end.saturating_sub(VISIBLE_LINES);
            view.output[start..end].to_vec()
        }
        ConsoleScrollView::Home => view.output[..VISIBLE_LINES].to_vec(),
    }
}

#[cfg(test)]
#[path = "../tests/unit/bevy_console.rs"]
mod tests;
