//! Reusable Bevy presentation primitives for the graphical frontend.
//!
//! This module depends only on Bevy and the frontend-neutral semantic input
//! model. Gameplay systems remain responsible for translating authoritative game
//! state into presentation view models and interpreting semantic input events.

use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use bevy::input::mouse::MouseWheel;
use bevy::input::ButtonState;
use bevy::input_focus::{
    tab_navigation::{TabGroup, TabIndex},
    FocusCause, InputFocus,
};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, Justify, LineBreak, TextCursorStyle, TextLayout};
use bevy::ui::{Pressed, UiTransform};
use bevy::ui_widgets::{Activate, Button};
use bevy::window::PrimaryWindow;
use std::collections::{HashMap, HashSet};
#[cfg(target_os = "android")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

use crate::input::InputEvent;

pub const THEME_BACKGROUND: Color = Color::srgb(7.0 / 255.0, 7.0 / 255.0, 7.0 / 255.0);
pub const THEME_PANEL: Color = Color::srgb(16.0 / 255.0, 16.0 / 255.0, 16.0 / 255.0);
pub const THEME_PANEL_ALT: Color = Color::srgb(25.0 / 255.0, 25.0 / 255.0, 25.0 / 255.0);
pub const THEME_TEXT: Color = Color::srgb(0.9, 0.88, 0.82);
pub const THEME_MUTED: Color = Color::srgb(0.62, 0.6, 0.58);
pub const THEME_ACCENT: Color = Color::srgb(0.72, 0.62, 0.46);
pub const THEME_SURFACE: Color = Color::srgba(14.0 / 255.0, 14.0 / 255.0, 14.0 / 255.0, 0.82);
pub const THEME_SURFACE_STRONG: Color =
    Color::srgba(22.0 / 255.0, 22.0 / 255.0, 22.0 / 255.0, 0.96);
pub const THEME_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.82);
pub const THEME_OVERLAY_SOFT: Color = Color::srgba(0.0, 0.0, 0.0, 0.58);
pub const THEME_BLOOD: Color = Color::srgb(0.42, 0.035, 0.025);
pub const THEME_BLOOD_DARK: Color = Color::srgb(0.11, 0.012, 0.012);
pub const THEME_BORDER: Color = Color::srgba(0.35, 0.30, 0.24, 0.72);
pub const THEME_HOVER: Color = Color::srgba(0.20, 0.16, 0.12, 0.92);
pub const THEME_PRESSED: Color = Color::srgba(0.30, 0.24, 0.17, 0.96);
pub const THEME_SELECTED: Color = Color::srgba(0.16, 0.13, 0.10, 0.90);
pub const THEME_SKY_DAY: Color = Color::srgb(0.19, 0.25, 0.32);
pub const THEME_SKY_DAWN: Color = Color::srgb(0.38, 0.19, 0.17);
pub const THEME_SKY_DUSK: Color = Color::srgb(0.32, 0.13, 0.14);
pub const THEME_SKY_NIGHT: Color = Color::srgb(0.08, 0.045, 0.085);
pub const THEME_SKY_HORIZON_WARM: Color = Color::srgba(0.48, 0.15, 0.075, 0.56);

pub const UI_TOUCH_TARGET_PX: f32 = 48.0;
pub const UI_COMPACT_GAP_VMIN: f32 = 1.111;
pub const UI_SURFACE_PADDING_VMIN: f32 = 1.667;
pub const UI_ACTION_PADDING_HORIZONTAL_VMIN: f32 = 1.944;
pub const UI_ACTION_PADDING_VERTICAL_VMIN: f32 = 0.833;
pub const UI_ACTION_MAX_WIDTH_PX: f32 = 520.0;

#[derive(Component)]
pub struct UiSurface;

#[derive(Component)]
pub struct UiOverlayRoot;

#[derive(Component)]
pub struct UiActionButton;

#[derive(Component)]
struct UiActionButtonGroup;

#[derive(Component)]
struct UiStackedActionButton;

#[derive(Component)]
pub struct UiMenuButton;

#[derive(Component)]
pub struct UiConditionIndicator;

#[derive(Component)]
pub struct UiContextMessage;

#[derive(Component)]
pub struct UiStyledButton;

#[derive(Component)]
pub struct UiSelected;

/// Marks interactive nodes whose minimum hit area must remain 48 logical pixels
/// on Android, independent of the user's visual UI scale.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiTouchTargetShape {
    MinimumHeight,
    MinimumBounds { width: u16, height: u16 },
    Square,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiTabButton {
    pub(crate) index: usize,
}

#[derive(Component)]
pub(crate) struct UiTabContent;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiHealthGauge {
    pub current: i32,
    pub maximum: i32,
}

#[derive(Component)]
struct UiHealthGaugeFill;

#[derive(Component)]
struct UiHealthGaugeShadow;

#[derive(Component)]
struct UiGaugeValueText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceTone {
    #[allow(dead_code)] // TODO: this varient is never constructed; remove thid after it is used
    Quiet,
    Strong,
}

#[derive(Component)]
pub struct BevyScreenRoot;

#[derive(Component)]
pub struct ChoiceButton {
    pub index: usize,
}

#[derive(Component)]
pub struct TextContent;

#[derive(Component)]
pub struct GaugeFill;

#[derive(Component)]
pub(crate) struct TouchScrollablePanel;

#[derive(Component)]
pub(crate) struct LifecycleTextField {
    pub(crate) index: usize,
}

#[derive(Component)]
struct ContextualConsoleInput;

#[derive(Component)]
struct ContextualConsoleTab;

#[derive(Resource, Default, Debug)]
pub(crate) struct ConsoleTextInputFocus {
    pub(crate) active: bool,
    suppress_next_back: bool,
}

#[cfg(target_os = "android")]
static ANDROID_BACK_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "android")]
static ANDROID_TEXT_INPUT_EVENTS: Mutex<Vec<(u8, AndroidTextInputEvent)>> = Mutex::new(Vec::new());

#[cfg(target_os = "android")]
#[derive(Debug)]
pub(crate) enum AndroidTextInputEvent {
    Changed(String),
    Submitted,
    Dismissed,
}

#[cfg(target_os = "android")]
fn queue_android_text_input_event(target: i32, event: AndroidTextInputEvent) {
    let Ok(target) = u8::try_from(target) else {
        return;
    };
    if target > 1 {
        return;
    }

    if let Ok(mut events) = ANDROID_TEXT_INPUT_EVENTS.lock() {
        events.push((target, event));
    }
}

#[cfg(target_os = "android")]
pub(crate) fn android_take_text_input_events(target: u8) -> Vec<AndroidTextInputEvent> {
    let Ok(mut events) = ANDROID_TEXT_INPUT_EVENTS.lock() else {
        return Vec::new();
    };

    let mut selected = Vec::new();
    let mut retained = Vec::with_capacity(events.len());
    for (event_target, event) in events.drain(..) {
        if event_target == target {
            selected.push(event);
        } else {
            retained.push((event_target, event));
        }
    }
    *events = retained;
    selected
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rightward_everunfoldingsaga_MainActivity_nativeBackNavigation(
    _env: jni::EnvUnowned<'_>,
    _class: jni::objects::JClass<'_>,
) {
    ANDROID_BACK_REQUESTED.store(true, Ordering::Release);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rightward_everunfoldingsaga_MainActivity_nativeAndroidTextInputChanged(
    mut env: jni::EnvUnowned<'_>,
    _class: jni::objects::JClass<'_>,
    target: i32,
    text: jni::objects::JString<'_>,
) {
    match env
        .with_env(|env| -> jni::errors::Result<()> {
            let text = text.try_to_string(env)?;
            queue_android_text_input_event(target, AndroidTextInputEvent::Changed(text));
            Ok(())
        })
        .into_outcome()
    {
        jni::Outcome::Ok(()) => {}
        jni::Outcome::Err(error) => {
            bevy::log::warn!("Could not decode Android text input: {error}");
        }
        jni::Outcome::Panic(_) => {
            bevy::log::warn!("Android text input JNI callback panicked");
        }
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rightward_everunfoldingsaga_MainActivity_nativeAndroidTextInputSubmitted(
    _env: jni::EnvUnowned<'_>,
    _class: jni::objects::JClass<'_>,
    target: i32,
) {
    queue_android_text_input_event(target, AndroidTextInputEvent::Submitted);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rightward_everunfoldingsaga_MainActivity_nativeAndroidTextInputDismissed(
    _env: jni::EnvUnowned<'_>,
    _class: jni::objects::JClass<'_>,
    target: i32,
) {
    queue_android_text_input_event(target, AndroidTextInputEvent::Dismissed);
}

const TOUCH_SCROLL_THRESHOLD_PX: f32 = 12.0;

fn touch_scroll_dragged(start: Vec2, current: Vec2) -> bool {
    current.distance(start) > TOUCH_SCROLL_THRESHOLD_PX
}

#[derive(Clone, Copy, Debug)]
struct ActiveTouchScroll {
    id: u64,
    panel: Entity,
    start: Vec2,
    last: Vec2,
    button: Option<Entity>,
    dragging: bool,
}

#[derive(Resource, Default, Debug)]
struct TouchScrollState {
    active: Option<ActiveTouchScroll>,
}

impl TouchScrollState {
    fn should_suppress_activation(&self, button: Entity, touches: &Touches) -> bool {
        let Some(active) = self.active else {
            return false;
        };
        if active.button != Some(button) {
            return false;
        }
        if active.dragging {
            return true;
        }

        let current_position = touches
            .iter()
            .find(|touch| touch.id() == active.id)
            .map(|touch| touch.position())
            .or_else(|| {
                touches
                    .iter_just_released()
                    .find(|touch| touch.id() == active.id)
                    .map(|touch| touch.position())
            });
        current_position.is_some_and(|position| touch_scroll_dragged(active.start, position))
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct UiButtonActivation(pub(crate) Entity);

#[derive(Resource, Default)]
pub struct SemanticInputQueue(pub Vec<InputEvent>);

#[derive(Resource, Default)]
pub struct GameplayInputQueue(pub Vec<InputEvent>);

pub(crate) fn physical_touch_position(position: Vec2, scale_factor: f32) -> Vec2 {
    position * scale_factor
}

#[cfg(target_os = "android")]
pub(crate) fn android_set_text_input_target(target: Option<u8>, text: &str) {
    if let Ok(mut events) = ANDROID_TEXT_INPUT_EVENTS.lock() {
        events.clear();
    }

    let Some(app) = bevy::android::ANDROID_APP.get().cloned() else {
        bevy::log::warn!(
            "Could not set Android text input target: Android app handle is unavailable"
        );
        return;
    };

    let java_target = target.map(i32::from).unwrap_or(-1);
    let text = text.to_owned();
    let java_app = app.clone();
    app.run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(java_app.vm_as_ptr().cast()) };
        if let Err(error) = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = java_app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let value = env.new_string(&text)?;
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("setAndroidTextInput"),
                jni::jni_sig!("(ILjava/lang/String;)V"),
                &[jni::JValue::from(java_target), jni::JValue::from(&value)],
            )?;
            Ok(())
        }) {
            bevy::log::warn!("Could not update Android native text input: {error}");
        }
    }));
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationState {
    pub current_screen: Option<ScreenId>,
    pub return_screen: Option<ScreenId>,
    pub selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Lifecycle,
    Gameplay,
    Character,
    Meditation,
    Talk,
    Conversation,
    RemainsResult,
    SpatialExploration,
    Combat,
    Console,
    Options,
    Feedback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresentationTransitionPhase {
    Entering,
    Active,
    Exiting,
    Finished,
}

/// A reusable scale transition for scene roots and contextual views.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct PresentationTransition {
    phase: PresentationTransitionPhase,
    elapsed: f32,
    duration: f32,
    start_scale: f32,
    end_scale: f32,
}

impl PresentationTransition {
    pub(crate) fn entering() -> Self {
        Self { phase: PresentationTransitionPhase::Entering, elapsed: 0.0, duration: 0.18, start_scale: 0.96, end_scale: 1.0 }
    }

    pub(crate) fn begin_exit(&mut self, current_scale: f32) {
        if matches!(self.phase, PresentationTransitionPhase::Exiting | PresentationTransitionPhase::Finished) {
            return;
        }
        self.phase = PresentationTransitionPhase::Exiting;
        self.elapsed = 0.0;
        self.duration = 0.16;
        self.start_scale = current_scale;
        self.end_scale = 0.96;
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.phase == PresentationTransitionPhase::Finished
    }
}

fn presentation_transition_scale(start: f32, end: f32, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let eased = progress * progress * (3.0 - 2.0 * progress);
    start + (end - start) * eased
}

pub(crate) fn update_presentation_transitions(
    time: Res<Time>,
    mut transitions: Query<(&mut PresentationTransition, &mut UiTransform)>,
) {
    for (mut transition, mut transform) in &mut transitions {
        if matches!(transition.phase, PresentationTransitionPhase::Active | PresentationTransitionPhase::Finished) {
            continue;
        }
        transition.elapsed += time.delta_secs();
        let progress = (transition.elapsed / transition.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        transform.scale = Vec2::splat(presentation_transition_scale(transition.start_scale, transition.end_scale, progress));
        if progress >= 1.0 {
            match transition.phase {
                PresentationTransitionPhase::Entering => {
                    transition.phase = PresentationTransitionPhase::Active;
                    transform.scale = Vec2::ONE;
                }
                PresentationTransitionPhase::Exiting => transition.phase = PresentationTransitionPhase::Finished,
                PresentationTransitionPhase::Active | PresentationTransitionPhase::Finished => {}
            }
        }
    }
}

fn screen_padding() -> Val {
    responsive_vmin(3.333)
}

fn screen_gap() -> Val {
    responsive_vmin(2.222)
}

fn panel_padding() -> Val {
    responsive_vmin(2.222)
}

fn panel_gap() -> Val {
    responsive_vmin(1.389)
}

/// Converts a viewport-minimum percentage into an Em value. Non-text UI nodes
/// receive a shared EmSize from ui_settings, allowing the user UI scale to
/// scale responsive layout spacing without coupling it to the font-size setting.
pub(crate) fn responsive_vmin(percent: f32) -> Val {
    em(percent / 2.778)
}

fn title_font_size() -> FontSize {
    FontSize::Rem(1.8)
}

fn screen_detail_font_size() -> FontSize {
    FontSize::Rem(0.72)
}

fn label_font_size() -> FontSize {
    FontSize::Rem(1.0)
}

pub(crate) fn muted_font_size() -> FontSize {
    FontSize::Rem(0.9)
}

pub fn install(app: &mut App) {
    app.add_systems(Update, update_presentation_transitions);
    app.add_message::<UiButtonActivation>()
        .add_observer(queue_button_activation)
        .init_resource::<SemanticInputQueue>()
        .init_resource::<GameplayInputQueue>()
        .init_resource::<NavigationState>()
        .init_resource::<ConsoleTextInputFocus>()
        .init_resource::<TouchScrollState>()
        .init_resource::<UiIconAssets>()
        .add_systems(
            PreUpdate,
            process_ime_events
                .after(bevy::input::InputSystems)
                .after(bevy::ui_widgets::ImeSystems::HandleEvents)
                .after(bevy::ui_widgets::ImeSystems::ToggleWindowIMEInput),
        )
        .add_systems(Update, keyboard_to_semantic_input)
        .add_systems(Update, mouse_to_semantic_input)
        .add_systems(Update, choice_button_input)
        .add_systems(Update, contextual_touch_input)
        .add_systems(PostUpdate, sync_ui_health_gauges)
        .add_systems(PostUpdate, sync_ui_button_visuals)
        .add_systems(Update, touch_scroll)
        .add_systems(PostUpdate, resolve_ui_icons)
        .add_systems(PostUpdate, organize_action_button_groups)
        .add_systems(PostUpdate, sync_ime_window)
        .add_systems(PostUpdate, sync_lifecycle_field_visuals);

    #[cfg(target_os = "android")]
    app.add_systems(Update, android_back_to_semantic_input);
}

pub fn spawn_world_root(commands: &mut Commands, background: Color) -> Entity {
    commands
        .spawn((
            BevyScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(screen_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: responsive_compact_gap(),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(background),
        ))
        .id()
}

pub fn spawn_screen(commands: &mut Commands, title: impl Into<String>) -> Entity {
    commands
        .spawn((
            BevyScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(screen_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: screen_gap(),
                ..default()
            },
            BackgroundColor(THEME_BACKGROUND),
            children![(
                Text::new(title.into()),
                TextContent,
                TextFont::from_font_size(title_font_size()),
                TextColor(THEME_TEXT),
            )],
        ))
        .id()
}

fn screen_header_row_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        column_gap: responsive_compact_gap(),
        row_gap: px(2),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }
}

/// Creates a screen root with a secondary detail aligned to the right of its title.
pub fn spawn_screen_with_right_detail(
    commands: &mut Commands,
    title: impl Into<String>,
    detail: impl Into<String>,
) -> Entity {
    commands
        .spawn((
            BevyScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(screen_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: screen_gap(),
                ..default()
            },
            BackgroundColor(THEME_BACKGROUND),
            children![(
                screen_header_row_node(),
                children![
                    (
                        Text::new(title.into()),
                        Node {
                            min_width: px(0),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                        TextContent,
                        TextFont::from_font_size(title_font_size()),
                        TextColor(THEME_TEXT),
                    ),
                    (
                        Text::new(detail.into()),
                        Node {
                            min_width: px(0),
                            max_width: percent(40),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        // Keep the version on one line even when the title row gets tight.
                        TextLayout::no_wrap(),
                        TextContent,
                        TextFont::from_font_size(screen_detail_font_size()),
                        TextColor(THEME_MUTED),
                    ),
                ],
            )],
        ))
        .id()
}

pub fn spawn_panel(commands: &mut Commands, parent: Entity) -> Entity {
    let panel = commands
        .spawn((
            TouchScrollablePanel,
            TabGroup::modal(),
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                padding: UiRect::all(panel_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: panel_gap(),
                overflow: Overflow::scroll(),
                ..default()
            },
            BackgroundColor(THEME_PANEL),
        ))
        .id();
    commands.entity(parent).add_child(panel);
    panel
}

fn fixed_panel_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        min_height: px(0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        padding: UiRect::all(panel_padding()),
        flex_direction: FlexDirection::Column,
        row_gap: panel_gap(),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Creates a fixed screen panel; only its dedicated list child should scroll.
pub fn spawn_fixed_panel(commands: &mut Commands, parent: Entity) -> Entity {
    let panel = commands
        .spawn((
            TabGroup::modal(),
            fixed_panel_node(),
            BackgroundColor(THEME_PANEL),
        ))
        .id();
    commands.entity(parent).add_child(panel);
    panel
}

fn scrollable_list_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        min_height: px(0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        flex_direction: FlexDirection::Column,
        row_gap: panel_gap(),
        overflow: Overflow::scroll(),
        ..default()
    }
}

/// Creates a stable tab-content host. Individual tabs can replace its children
/// without rebuilding the screen frame, tab strip, or surrounding controls.
fn tab_content_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        min_height: px(0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        flex_direction: FlexDirection::Column,
        row_gap: panel_gap(),
        overflow: Overflow::clip(),
        ..default()
    }
}

pub(crate) fn spawn_tab_content(commands: &mut Commands, parent: Entity) -> Entity {
    let content = commands.spawn((UiTabContent, tab_content_node())).id();
    commands.entity(parent).add_child(content);
    content
}

/// Removes only the current children of a stable tab-content host.
pub(crate) fn clear_tab_content(
    commands: &mut Commands,
    content: Entity,
    children: &Query<&Children>,
) {
    if let Ok(current_children) = children.get(content) {
        for child in current_children.iter() {
            commands.entity(child).despawn();
        }
    }
}

/// Creates the only scrolling region inside a fixed Options-style panel.
pub fn spawn_scrollable_list(commands: &mut Commands, parent: Entity) -> Entity {
    let list = commands
        .spawn((TouchScrollablePanel, scrollable_list_node()))
        .id();
    commands.entity(parent).add_child(list);
    list
}

fn option_footer_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        min_height: touch_target_size(),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        column_gap: responsive_compact_gap(),
        row_gap: responsive_compact_gap(),
        justify_content: JustifyContent::End,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        padding: UiRect::top(responsive_compact_gap()),
        ..default()
    }
}

/// Creates a non-scrolling footer below the option list.
pub fn spawn_option_footer(commands: &mut Commands, parent: Entity) -> Entity {
    let footer = commands.spawn(option_footer_node()).id();
    commands.entity(parent).add_child(footer);
    footer
}

/// Creates the shared, themed tab strip used by multi-section screens.
pub fn spawn_tab_strip(
    commands: &mut Commands,
    parent: Entity,
    labels: &[&str],
    selected: usize,
) -> Entity {
    let tab_strip = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: responsive_compact_gap(),
            row_gap: px(4),
            flex_shrink: 0.0,
            ..default()
        })
        .id();
    commands.entity(parent).add_child(tab_strip);
    commands.entity(tab_strip).with_children(|tabs| {
        for (index, label) in labels.iter().enumerate() {
            let mut tab = tabs.spawn((
                Button,
                Hovered::default(),
                UiTouchTargetShape::MinimumHeight,
                ChoiceButton { index },
                UiTabButton { index },
                UiStyledButton,
                Node {
                    width: Val::Auto,
                    min_width: px(100),
                    min_height: touch_target_size(),
                    padding: UiRect::axes(responsive_surface_padding(), px(6)),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                BorderColor::all(THEME_BORDER),
                BackgroundColor(THEME_SURFACE),
                children![(
                    Text::new(*label),
                    TextContent,
                    TextFont::from_font_size(muted_font_size()),
                    TextColor(THEME_TEXT),
                )],
            ));
            if selected == index {
                tab.insert(UiSelected);
            }
        }
    });
    tab_strip
}

pub(crate) fn update_tab_strip_selection(
    commands: &mut Commands,
    tabs: &Query<(Entity, &UiTabButton)>,
    selected: usize,
) {
    for (entity, tab) in tabs {
        if tab.index == selected {
            commands.entity(entity).insert(UiSelected);
        } else {
            commands.entity(entity).remove::<UiSelected>();
        }
    }
}

fn standalone_label_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        max_width: percent(100),
        flex_shrink: 0.0,
        ..default()
    }
}

pub fn spawn_label(commands: &mut Commands, parent: Entity, text: impl Into<String>) -> Entity {
    let label = commands
        .spawn((
            Text::new(text.into()),
            standalone_label_node(),
            TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
            TextContent,
            TextFont::from_font_size(label_font_size()),
            TextColor(THEME_TEXT),
        ))
        .id();
    commands.entity(parent).add_child(label);
    label
}

pub fn spawn_muted_label(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
) -> Entity {
    let label = commands
        .spawn((
            Text::new(text.into()),
            standalone_label_node(),
            TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
            TextContent,
            TextFont::from_font_size(muted_font_size()),
            TextColor(THEME_MUTED),
        ))
        .id();
    commands.entity(parent).add_child(label);
    label
}

pub fn spawn_text_input_field(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    label: impl Into<String>,
    value: impl Into<String>,
    selected: bool,
) -> Entity {
    let label = label.into();
    let value = value.into();
    let row = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            min_height: px(48),
            flex_direction: FlexDirection::Row,
            column_gap: responsive_vmin(1.389),
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    let label_entity = commands
        .spawn((
            Text::new(format!("{label}:")),
            Node {
                width: percent(25),
                min_width: px(0),
                ..default()
            },
            TextContent,
            TextFont::from_font_size(label_font_size()),
            TextColor(THEME_TEXT),
        ))
        .id();

    let field = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::MinimumHeight,
            LifecycleTextField { index },
            EditableText::new(&value),
            TextCursorStyle::default(),
            TextLayout::no_wrap(),
            TabIndex(index as i32),
            Node {
                width: percent(75),
                min_width: px(0),
                min_height: px(48),
                padding: UiRect::axes(responsive_vmin(1.389), responsive_vmin(0.833)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(if selected { THEME_ACCENT } else { THEME_MUTED }),
            BackgroundColor(THEME_PANEL_ALT),
            TextFont::from_font_size(label_font_size()),
            TextColor(THEME_TEXT),
        ))
        .id();

    commands.entity(row).add_child(label_entity);
    commands.entity(row).add_child(field);
    commands.entity(parent).add_child(row);
    field
}

pub(crate) fn spawn_console_input_row(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
) -> Entity {
    let row = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Row,
            column_gap: responsive_vmin(1.111),
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    let input = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::MinimumHeight,
            ContextualConsoleInput,
            Text::new(text.into()),
            Node {
                width: percent(84),
                min_width: px(0),
                max_width: percent(100),
                min_height: px(48),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                padding: UiRect::axes(responsive_vmin(1.389), responsive_vmin(0.833)),
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                ..default()
            },
            TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
            TextContent,
            TextFont::from_font_size(label_font_size()),
            TextColor(THEME_TEXT),
            BorderColor::all(THEME_ACCENT),
            BackgroundColor(THEME_PANEL_ALT),
        ))
        .id();

    let tab = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::MinimumHeight,
            ContextualConsoleTab,
            Node {
                width: px(64),
                min_width: px(64),
                min_height: px(48),
                padding: UiRect::axes(px(10), px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(THEME_ACCENT),
            BackgroundColor(THEME_PANEL_ALT),
            children![(
                Text::new("Tab"),
                TextContent,
                TextFont::from_font_size(muted_font_size()),
                TextColor(THEME_TEXT),
            )],
        ))
        .id();

    commands.entity(row).add_child(input);
    commands.entity(row).add_child(tab);
    commands.entity(parent).add_child(row);
    input
}

pub fn responsive_compact_gap() -> Val {
    responsive_vmin(UI_COMPACT_GAP_VMIN)
}

pub fn responsive_surface_padding() -> Val {
    responsive_vmin(UI_SURFACE_PADDING_VMIN)
}

pub fn touch_target_size() -> Val {
    px(UI_TOUCH_TARGET_PX)
}

pub fn spawn_surface(commands: &mut Commands, parent: Entity, tone: SurfaceTone) -> Entity {
    let background = match tone {
        SurfaceTone::Quiet => THEME_SURFACE,
        SurfaceTone::Strong => THEME_SURFACE_STRONG,
    };
    let surface = commands
        .spawn((
            UiSurface,
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(responsive_surface_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: responsive_compact_gap(),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(THEME_BORDER),
            BackgroundColor(background),
        ))
        .id();
    commands.entity(parent).add_child(surface);
    surface
}

pub fn spawn_overlay(commands: &mut Commands, parent: Entity) -> Entity {
    let overlay = commands
        .spawn((
            UiOverlayRoot,
            TabGroup::modal(),
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                padding: UiRect::all(screen_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: responsive_compact_gap(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(THEME_OVERLAY_SOFT),
        ))
        .id();
    commands.entity(parent).add_child(overlay);
    overlay
}

fn action_button_group_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        max_width: px(UI_ACTION_MAX_WIDTH_PX),
        flex_direction: FlexDirection::Column,
        row_gap: responsive_compact_gap(),
        flex_shrink: 0.0,
        align_self: AlignSelf::FlexStart,
        margin: UiRect {
            top: Val::Auto,
            ..default()
        },
        ..default()
    }
}

fn action_button_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        max_width: percent(100),
        min_height: touch_target_size(),
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: UiRect::axes(
            responsive_vmin(UI_ACTION_PADDING_HORIZONTAL_VMIN),
            responsive_vmin(UI_ACTION_PADDING_VERTICAL_VMIN),
        ),
        border: UiRect::all(px(1)),
        justify_content: JustifyContent::Start,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// The two content slots created by a shared Options/settings row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionRowSlots {
    pub row: Entity,
    pub label: Entity,
    pub controls: Entity,
}

#[derive(Component)]
struct OptionRowLabelSlot;

#[derive(Component)]
struct OptionRowControlsSlot;

fn option_row_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        min_height: touch_target_size(),
        padding: UiRect::axes(responsive_surface_padding(), responsive_vmin(0.833)),
        border: UiRect::all(px(1)),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        column_gap: responsive_compact_gap(),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }
}

fn option_row_label_node() -> Node {
    Node {
        min_width: px(0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        flex_direction: FlexDirection::Column,
        row_gap: px(2),
        align_items: AlignItems::Start,
        ..default()
    }
}

fn option_row_controls_node() -> Node {
    Node {
        min_width: px(0),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Row,
        column_gap: responsive_compact_gap(),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::End,
        ..default()
    }
}

fn compact_action_button_node() -> Node {
    Node {
        width: Val::Auto,
        min_width: px(76),
        max_width: px(180),
        min_height: touch_target_size(),
        flex_grow: 0.0,
        flex_shrink: 0.0,
        padding: UiRect::axes(
            responsive_vmin(1.111),
            responsive_vmin(UI_ACTION_PADDING_VERTICAL_VMIN),
        ),
        border: UiRect::all(px(1)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn option_value_node() -> Node {
    Node {
        min_width: px(58),
        min_height: touch_target_size(),
        flex_shrink: 0.0,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates the shared landscape option row: the option name stays on the left
/// and its controls stay together on the right.
pub fn spawn_option_row(commands: &mut Commands, parent: Entity) -> OptionRowSlots {
    let row = commands
        .spawn((
            UiSurface,
            option_row_node(),
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE),
        ))
        .id();
    let label = commands
        .spawn((OptionRowLabelSlot, option_row_label_node()))
        .id();
    let controls = commands
        .spawn((OptionRowControlsSlot, option_row_controls_node()))
        .id();

    commands.entity(row).add_child(label);
    commands.entity(row).add_child(controls);
    commands.entity(parent).add_child(row);

    OptionRowSlots {
        row,
        label,
        controls,
    }
}

/// Adds a current setting value to the right-hand control group.
pub fn spawn_option_value(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
) -> Entity {
    let value = commands
        .spawn((
            Text::new(text.into()),
            option_value_node(),
            TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
            TextContent,
            TextFont::from_font_size(muted_font_size()),
            TextColor(THEME_TEXT),
        ))
        .id();
    commands.entity(parent).add_child(value);
    value
}

/// Spawns a standard horizontal button for option rows and persistent footers.
pub fn spawn_compact_action_button(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    label: impl Into<String>,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::MinimumBounds {
                width: UI_TOUCH_TARGET_PX as u16,
                height: UI_TOUCH_TARGET_PX as u16,
            },
            ChoiceButton { index },
            UiActionButton,
            UiStyledButton,
            compact_action_button_node(),
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE),
            children![(
                Text::new(label.into()),
                Node {
                    min_width: px(52),
                    max_width: percent(100),
                    ..default()
                },
                TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
                TextContent,
                TextFont::from_font_size(muted_font_size()),
                TextColor(THEME_TEXT),
            )],
        ))
        .id();
    commands.entity(parent).add_child(button);
    button
}

#[derive(Component)]
struct UiIconName(String);

#[derive(Resource, Default)]
struct UiIconAssets(HashMap<String, Handle<Image>>);

fn icon_asset_reference(icon: &str) -> String {
    if icon.starts_with("mod:") || icon.contains('/') {
        icon.to_string()
    } else {
        format!("icons/ui/{icon}.png")
    }
}

/// Resolve semantic icon names into actual PNG sprites from the selected game
/// data folder. Missing images are logged and omitted, never replaced by text.
fn resolve_ui_icons(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<UiIconAssets>,
    mut logged_missing: Local<HashSet<String>>,
    icons: Query<(Entity, &UiIconName)>,
) {
    for (entity, icon) in &icons {
        let reference = icon_asset_reference(&icon.0);
        if let Some(image) = cache.0.get(&reference) {
            commands
                .entity(entity)
                .insert(ImageNode::new(image.clone()))
                .remove::<UiIconName>();
            continue;
        }

        match crate::asset_loading::load_image_from_data(&mut images, &reference) {
            Ok(image) => {
                cache.0.insert(reference.clone(), image.clone());
                commands
                    .entity(entity)
                    .insert(ImageNode::new(image))
                    .remove::<UiIconName>();
            }
            Err(error) => {
                if logged_missing.insert(reference.clone()) {
                    warn!("Could not load UI icon {reference}: {error}");
                }
                commands.entity(entity).remove::<UiIconName>();
            }
        }
    }
}

pub fn spawn_action_button(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    icon: impl Into<String>,
    label: impl Into<String>,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::MinimumHeight,
            ChoiceButton { index },
            UiActionButton,
            UiStackedActionButton,
            UiStyledButton,
            action_button_node(),
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE),
            children![(
                Node {
                    width: percent(100),
                    min_width: px(0),
                    max_width: percent(100),
                    flex_direction: FlexDirection::Row,
                    column_gap: responsive_compact_gap(),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Node {
                            width: px(22),
                            height: px(22),
                            min_width: px(22),
                            min_height: px(22),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        UiIconName(icon.into()),
                    ),
                    (
                        Text::new(label.into()),
                        Node {
                            min_width: px(0),
                            max_width: percent(100),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                        TextContent,
                        TextFont::from_font_size(label_font_size()),
                        TextColor(THEME_TEXT),
                    ),
                ],
            )],
        ))
        .id();
    commands.entity(parent).add_child(button);
    button
}

pub fn spawn_action_button_with_detail(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    icon: impl Into<String>,
    label: impl Into<String>,
    detail: impl Into<String>,
) -> Entity {
    let label = label.into();
    let detail = detail.into();
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::MinimumHeight,
            ChoiceButton { index },
            UiActionButton,
            UiStackedActionButton,
            UiStyledButton,
            action_button_node(),
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE),
            children![(
                Node {
                    width: percent(100),
                    min_width: px(0),
                    max_width: percent(100),
                    flex_direction: FlexDirection::Row,
                    column_gap: responsive_compact_gap(),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Node {
                            width: px(22),
                            height: px(22),
                            min_width: px(22),
                            min_height: px(22),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        UiIconName(icon.into()),
                    ),
                    (
                        Node {
                            width: percent(100),
                            min_width: px(0),
                            max_width: percent(100),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(2),
                            ..default()
                        },
                        children![
                            (
                                Text::new(label),
                                Node {
                                    width: percent(100),
                                    min_width: px(0),
                                    max_width: percent(100),
                                    ..default()
                                },
                                TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                                TextContent,
                                TextFont::from_font_size(label_font_size()),
                                TextColor(THEME_TEXT),
                            ),
                            (
                                Text::new(detail),
                                Node {
                                    width: percent(100),
                                    min_width: px(0),
                                    max_width: percent(100),
                                    ..default()
                                },
                                TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                                TextContent,
                                TextFont::from_font_size(muted_font_size()),
                                TextColor(THEME_MUTED),
                            ),
                        ],
                    ),
                ],
            )],
        ))
        .id();
    commands.entity(parent).add_child(button);
    button
}

pub fn spawn_primary_action_button(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    icon: impl Into<String>,
    label: impl Into<String>,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::MinimumHeight,
            ChoiceButton { index },
            UiActionButton,
            UiStyledButton,
            Node {
                width: Val::Auto,
                min_width: px(110),
                min_height: touch_target_size(),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                padding: UiRect::axes(
                    responsive_vmin(UI_ACTION_PADDING_HORIZONTAL_VMIN),
                    responsive_vmin(UI_ACTION_PADDING_VERTICAL_VMIN),
                ),
                border: UiRect::all(px(1)),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE),
            children![(
                Node {
                    width: percent(100),
                    min_width: px(0),
                    max_width: percent(100),
                    flex_direction: FlexDirection::Row,
                    column_gap: responsive_compact_gap(),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Node {
                            width: px(22),
                            height: px(22),
                            min_width: px(22),
                            min_height: px(22),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        UiIconName(icon.into()),
                    ),
                    (
                        Text::new(label.into()),
                        Node {
                            min_width: px(0),
                            max_width: percent(100),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                        TextContent,
                        TextFont::from_font_size(label_font_size()),
                        TextColor(THEME_TEXT),
                    ),
                ],
            )],
        ))
        .id();
    commands.entity(parent).add_child(button);
    button
}

pub fn spawn_menu_button(
    commands: &mut Commands,
    parent: Entity,
    icon: impl Into<String>,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            UiTouchTargetShape::Square,
            UiMenuButton,
            UiStyledButton,
            Node {
                width: touch_target_size(),
                min_width: touch_target_size(),
                height: touch_target_size(),
                min_height: touch_target_size(),
                border: UiRect::all(px(1)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE),
            children![(
                Node {
                    width: px(26),
                    height: px(26),
                    ..default()
                },
                UiIconName(icon.into())
            )],
        ))
        .id();
    commands.entity(parent).add_child(button);
    button
}

pub fn spawn_health_gauge(
    commands: &mut Commands,
    parent: Entity,
    current: i32,
    maximum: i32,
) -> Entity {
    let gauge_ratio = gauge_ratio(current, maximum);
    let gauge = commands
        .spawn((
            UiHealthGauge { current, maximum },
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(28),
                position_type: PositionType::Relative,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE_STRONG),
            children![
                (
                    UiHealthGaugeShadow,
                    Node {
                        width: percent(gauge_ratio * 100.0),
                        height: percent(100),
                        position_type: PositionType::Absolute,
                        left: px(2),
                        top: px(1),
                        ..default()
                    },
                    BackgroundColor(THEME_BLOOD_DARK),
                ),
                (
                    UiHealthGaugeFill,
                    Node {
                        width: percent(gauge_ratio * 100.0),
                        height: percent(100),
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        ..default()
                    },
                    BackgroundColor(THEME_BLOOD),
                ),
                (
                    UiGaugeValueText,
                    Text::new(format!("{current} / {maximum}")),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        min_width: px(0),
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    TextContent,
                    TextFont::from_font_size(muted_font_size()),
                    TextColor(THEME_TEXT),
                ),
            ],
        ))
        .id();
    commands.entity(parent).add_child(gauge);
    gauge
}

pub fn spawn_condition_indicator(
    commands: &mut Commands,
    parent: Entity,
    icon: impl Into<String>,
) -> Entity {
    let indicator = commands
        .spawn((
            UiConditionIndicator,
            Node {
                width: touch_target_size(),
                min_width: touch_target_size(),
                height: touch_target_size(),
                min_height: touch_target_size(),
                padding: UiRect::all(responsive_vmin(0.556)),
                border: UiRect::all(px(1)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(THEME_BORDER),
            BackgroundColor(THEME_SURFACE),
            children![(
                Node {
                    width: px(22),
                    height: px(22),
                    ..default()
                },
                UiIconName(icon.into())
            )],
        ))
        .id();
    commands.entity(parent).add_child(indicator);
    indicator
}

pub fn spawn_context_message(
    commands: &mut Commands,
    parent: Entity,
    message: impl Into<String>,
) -> Entity {
    let message = commands
        .spawn((
            UiContextMessage,
            Node {
                width: percent(100),
                min_width: px(0),
                max_width: percent(100),
                padding: UiRect::axes(responsive_surface_padding(), responsive_vmin(1.111)),
                border: UiRect::all(px(1)),
                flex_shrink: 1.0,
                ..default()
            },
            BorderColor::all(THEME_ACCENT),
            BackgroundColor(THEME_SURFACE_STRONG),
            children![(
                Text::new(message.into()),
                Node {
                    width: percent(100),
                    min_width: px(0),
                    max_width: percent(100),
                    ..default()
                },
                TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                TextContent,
                TextFont::from_font_size(label_font_size()),
                TextColor(THEME_TEXT),
            )],
        ))
        .id();
    commands.entity(parent).add_child(message);
    message
}

fn sync_ui_health_gauges(
    gauges: Query<(&UiHealthGauge, &Children), Changed<UiHealthGauge>>,
    mut fills: Query<&mut Node, (With<UiHealthGaugeFill>, Without<UiHealthGaugeShadow>)>,
    mut shadows: Query<&mut Node, (With<UiHealthGaugeShadow>, Without<UiHealthGaugeFill>)>,
    mut values: Query<&mut Text, With<UiGaugeValueText>>,
) {
    for (gauge, children) in &gauges {
        for child in children.iter() {
            let ratio = gauge_ratio(gauge.current, gauge.maximum);
            if let Ok(mut fill) = fills.get_mut(child) {
                fill.width = percent(ratio * 100.0);
            }
            if let Ok(mut shadow) = shadows.get_mut(child) {
                shadow.width = percent(ratio * 100.0);
            }
            if let Ok(mut value) = values.get_mut(child) {
                *value = Text::new(format!("{} / {}", gauge.current, gauge.maximum));
            }
        }
    }
}

#[allow(clippy::type_complexity)] // ECS query combines widget state and visual components.
fn sync_ui_button_visuals(
    mut buttons: Query<
        (
            &Hovered,
            Has<Pressed>,
            Option<&UiSelected>,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        With<UiStyledButton>,
    >,
) {
    for (hovered, pressed, selected, mut border, mut background) in &mut buttons {
        let (border_color, background_color) = if pressed {
            (THEME_ACCENT, THEME_PRESSED)
        } else if hovered.get() {
            (THEME_ACCENT, THEME_HOVER)
        } else if selected.is_some() {
            (THEME_ACCENT, THEME_SELECTED)
        } else {
            (THEME_BORDER, THEME_SURFACE)
        };
        border.set_all(border_color);
        *background = BackgroundColor(background_color);
    }
}

pub fn spawn_gauge(commands: &mut Commands, parent: Entity, current: i32, maximum: i32) -> Entity {
    let ratio = gauge_ratio(current, maximum);
    let gauge = commands
        .spawn((
            Node {
                width: percent(100),
                min_width: px(0),
                height: px(18),
                ..default()
            },
            BackgroundColor(THEME_PANEL_ALT),
            children![(
                GaugeFill,
                Node {
                    width: percent(ratio * 100.0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(THEME_ACCENT),
            )],
        ))
        .id();
    commands.entity(parent).add_child(gauge);
    gauge
}

pub(crate) fn gauge_ratio(current: i32, maximum: i32) -> f32 {
    if maximum <= 0 {
        0.0
    } else {
        (current.max(0) as f32 / maximum as f32).clamp(0.0, 1.0)
    }
}

fn process_ime_events(
    mut ime: MessageReader<Ime>,
    mut keyboard_input: MessageWriter<KeyboardInput>,
    mut input_focus: ResMut<InputFocus>,
    editable_texts: Query<(), With<EditableText>>,
    mut console_focus: ResMut<ConsoleTextInputFocus>,
) {
    for event in ime.read() {
        match event {
            Ime::Commit { window, value }
                if !cfg!(target_os = "android") && console_focus.active && !value.is_empty() =>
            {
                keyboard_input.write(KeyboardInput {
                    key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
                    logical_key: Key::Character(value.clone().into()),
                    state: ButtonState::Pressed,
                    window: *window,
                    repeat: false,
                    text: Some(value.clone().into()),
                });
            }
            Ime::Disabled { .. } if !cfg!(target_os = "android") => {
                if console_focus.active {
                    console_focus.active = false;
                    console_focus.suppress_next_back = true;
                } else if input_focus
                    .get()
                    .is_some_and(|entity| editable_texts.contains(entity))
                {
                    input_focus.clear();
                    console_focus.suppress_next_back = true;
                }
            }
            _ => {}
        }
    }
}

#[cfg(target_os = "android")]
fn android_back_to_semantic_input(
    navigation: Res<NavigationState>,
    input_focus: Res<InputFocus>,
    editable_fields: Query<(), With<LifecycleTextField>>,
    console_focus: Res<ConsoleTextInputFocus>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    if !ANDROID_BACK_REQUESTED.swap(false, Ordering::AcqRel) {
        return;
    }

    let lifecycle_text_focused = navigation.current_screen == Some(ScreenId::Lifecycle)
        && input_focus
            .get()
            .is_some_and(|entity| editable_fields.contains(entity));

    if lifecycle_text_focused || console_focus.active {
        return;
    }

    if navigation.current_screen == Some(ScreenId::Lifecycle) {
        lifecycle_queue.0.push(InputEvent::Cancel);
    } else {
        gameplay_queue.0.push(InputEvent::Cancel);
    }
}

fn mouse_to_semantic_input(
    mouse: Res<ButtonInput<MouseButton>>,
    navigation: Res<NavigationState>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    if !mouse.just_pressed(MouseButton::Back) {
        return;
    }

    if navigation.current_screen == Some(ScreenId::Lifecycle) {
        lifecycle_queue.0.push(InputEvent::Cancel);
    } else {
        gameplay_queue.0.push(InputEvent::Cancel);
    }
}

fn keyboard_to_semantic_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    navigation: Res<NavigationState>,
    input_focus: Res<InputFocus>,
    editable_fields: Query<(), With<LifecycleTextField>>,
    mut console_focus: ResMut<ConsoleTextInputFocus>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    if navigation.current_screen != Some(ScreenId::Console) {
        console_focus.active = false;
    }

    let lifecycle_text_focused = navigation.current_screen == Some(ScreenId::Lifecycle)
        && input_focus
            .get()
            .is_some_and(|entity| editable_fields.contains(entity));

    let mappings = [
        (KeyCode::ArrowUp, InputEvent::Up),
        (KeyCode::ArrowDown, InputEvent::Down),
        (KeyCode::Home, InputEvent::Home),
        (KeyCode::End, InputEvent::End),
        (KeyCode::PageUp, InputEvent::PageUp),
        (KeyCode::PageDown, InputEvent::PageDown),
        (KeyCode::Enter, InputEvent::Confirm),
        (KeyCode::Escape, InputEvent::Cancel),
        (KeyCode::Tab, InputEvent::Tab),
        (KeyCode::Backspace, InputEvent::Backspace),
        (KeyCode::Delete, InputEvent::Delete),
        (KeyCode::BrowserBack, InputEvent::Cancel),
        (KeyCode::KeyJ, InputEvent::Character('j')),
        (KeyCode::KeyK, InputEvent::Character('k')),
        (KeyCode::Digit1, InputEvent::Character('1')),
        (KeyCode::Digit2, InputEvent::Character('2')),
        (KeyCode::Digit3, InputEvent::Character('3')),
    ];
    for (key, event) in mappings {
        if !keyboard.just_pressed(key) {
            continue;
        }
        if lifecycle_text_focused
            && !matches!(key, KeyCode::Enter | KeyCode::Escape | KeyCode::BrowserBack)
        {
            continue;
        }
        if key == KeyCode::BrowserBack {
            if console_focus.suppress_next_back {
                console_focus.suppress_next_back = false;
                continue;
            }
            if console_focus.active || lifecycle_text_focused {
                continue;
            }
        }
        if navigation.current_screen == Some(ScreenId::Lifecycle) {
            lifecycle_queue.0.push(event);
        } else {
            gameplay_queue.0.push(event);
        }
    }
}

// Reproduce keyboard navigation for pointer/touch activation so every choice
// uses the same selection and confirmation path.
pub(crate) fn queue_choice_activation(queue: &mut Vec<InputEvent>, index: usize) {
    queue.push(InputEvent::Home);
    for _ in 0..index {
        queue.push(InputEvent::Down);
    }
    queue.push(InputEvent::Confirm);
}

fn queue_button_activation(
    activation: On<Activate>,
    touches: Res<Touches>,
    scroll_state: Res<TouchScrollState>,
    mut activations: MessageWriter<UiButtonActivation>,
) {
    let button = activation.event_target();
    if scroll_state.should_suppress_activation(button, &touches) {
        return;
    }
    activations.write(UiButtonActivation(button));
}

fn choice_button_input(
    mut activations: MessageReader<UiButtonActivation>,
    choices: Query<(&ChoiceButton, Option<&UiTabButton>)>,
    mut navigation: ResMut<NavigationState>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    for activation in activations.read() {
        let Ok((choice, tab)) = choices.get(activation.0) else {
            continue;
        };
        if let Some(tab) = tab {
            match navigation.current_screen {
                Some(ScreenId::Options | ScreenId::Character) => {
                    gameplay_queue.0.push(InputEvent::SelectTab(tab.index));
                }
                Some(ScreenId::Lifecycle) => {
                    lifecycle_queue.0.push(InputEvent::SelectTab(tab.index));
                }
                _ => {}
            }
            continue;
        }

        navigation.selected = choice.index;
        let queue = if navigation.current_screen == Some(ScreenId::Lifecycle) {
            &mut lifecycle_queue.0
        } else {
            &mut gameplay_queue.0
        };
        queue_choice_activation(queue, choice.index);
    }
}

#[allow(clippy::type_complexity)] // For interaction_query
fn contextual_touch_input(
    mut activations: MessageReader<UiButtonActivation>,
    interaction_query: Query<(
        Entity,
        Option<&LifecycleTextField>,
        Option<&ContextualConsoleInput>,
        Option<&ContextualConsoleTab>,
        Option<&UiMenuButton>,
    )>,
    mut navigation: ResMut<NavigationState>,
    mut input_focus: ResMut<InputFocus>,
    mut console_focus: ResMut<ConsoleTextInputFocus>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    for activation in activations.read() {
        let Ok((entity, field, console, tab, menu_button)) = interaction_query.get(activation.0)
        else {
            continue;
        };
        if let Some(field) = field {
            if navigation.current_screen != Some(ScreenId::Lifecycle) {
                continue;
            }
            input_focus.set(entity, FocusCause::Pressed);
            navigation.selected = field.index;
            console_focus.active = false;
            console_focus.suppress_next_back = false;
        } else if console.is_some() {
            if navigation.current_screen == Some(ScreenId::Console) {
                console_focus.active = true;
                console_focus.suppress_next_back = false;
            }
        } else if menu_button.is_some() && navigation.current_screen == Some(ScreenId::Gameplay) {
            gameplay_queue.0.push(InputEvent::OpenSecondaryNavigation);
        } else if tab.is_some() && navigation.current_screen == Some(ScreenId::Console) {
            gameplay_queue.0.push(InputEvent::Tab);
        }
    }
}

fn organize_action_button_groups(
    mut commands: Commands,
    parents: Query<(Entity, &Children, Option<&UiActionButtonGroup>)>,
    action_buttons: Query<(), With<UiStackedActionButton>>,
    action_groups: Query<(), With<UiActionButtonGroup>>,
) {
    for (parent, children, is_action_group) in &parents {
        if is_action_group.is_some() {
            continue;
        }

        let buttons = children
            .iter()
            .filter(|entity| action_buttons.contains(*entity))
            .collect::<Vec<_>>();
        if buttons.is_empty() {
            continue;
        }

        let group = children
            .iter()
            .find(|entity| action_groups.contains(*entity))
            .unwrap_or_else(|| {
                let group = commands
                    .spawn((UiActionButtonGroup, action_button_group_node()))
                    .id();
                commands.entity(parent).add_child(group);
                group
            });

        for button in buttons {
            commands.entity(group).add_child(button);
        }
    }
}

fn touch_scroll(
    touches: Res<Touches>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchScrollState>,
    panels: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<TouchScrollablePanel>>,
    mut scroll_positions: Query<&mut ScrollPosition, With<TouchScrollablePanel>>,
    buttons: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    if let Some(cursor_position) = window.cursor_position() {
        let physical_cursor = physical_touch_position(cursor_position, window.scale_factor());
        let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();
        if wheel_delta != 0.0 {
            for (entity, computed, transform) in &panels {
                if computed.contains_point(*transform, physical_cursor) {
                    if let Ok(mut scroll) = scroll_positions.get_mut(entity) {
                        scroll.0.y = (scroll.0.y - wheel_delta * 48.0).max(0.0);
                    }
                    break;
                }
            }
        }
    } else {
        mouse_wheel.clear();
    }

    for touch in touches.iter_just_pressed() {
        let physical_position = physical_touch_position(touch.position(), window.scale_factor());
        let panel = panels
            .iter()
            .find(|(_, computed, transform)| {
                computed.contains_point(**transform, physical_position)
            })
            .map(|(entity, _, _)| entity);
        let Some(panel) = panel else {
            continue;
        };
        let button = buttons
            .iter()
            .find(|(_, computed, transform)| {
                computed.contains_point(**transform, physical_position)
            })
            .map(|(entity, _, _)| entity);
        state.active = Some(ActiveTouchScroll {
            id: touch.id(),
            panel,
            start: touch.position(),
            last: touch.position(),
            button,
            dragging: false,
        });
    }

    let Some(mut active) = state.active else {
        return;
    };
    if let Some(position) = touches
        .iter()
        .find(|touch| touch.id() == active.id)
        .map(|touch| touch.position())
    {
        let crossed_drag_threshold =
            !active.dragging && touch_scroll_dragged(active.start, position);
        if crossed_drag_threshold {
            active.dragging = true;
        }
        if active.dragging {
            // Apply the full movement once dragging starts so the threshold
            // distance does not feel like lost input.
            let delta = if crossed_drag_threshold {
                position - active.start
            } else {
                position - active.last
            };
            if let Ok(mut scroll) = scroll_positions.get_mut(active.panel) {
                scroll.0.y = (scroll.0.y - delta.y).max(0.0);
            }
        }
        active.last = position;
        state.active = Some(active);
    } else if touches.just_released(active.id) || touches.just_canceled(active.id) {
        state.active = None;
    }
}

#[cfg(not(target_os = "android"))]
fn sync_ime_window(
    input_focus: Res<InputFocus>,
    editable_texts: Query<(), With<EditableText>>,
    console_focus: Res<ConsoleTextInputFocus>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(mut window) = windows.iter_mut().next() else {
        return;
    };
    let native_editable_focused = input_focus
        .get()
        .is_some_and(|entity| editable_texts.contains(entity));
    window.ime_enabled = console_focus.active || native_editable_focused;
}

#[cfg(target_os = "android")]
fn sync_ime_window() {}

fn sync_lifecycle_field_visuals(
    input_focus: Res<InputFocus>,
    navigation: Res<NavigationState>,
    mut fields: Query<(
        Entity,
        &LifecycleTextField,
        &mut BorderColor,
        &mut BackgroundColor,
    )>,
) {
    let focused = input_focus
        .get()
        .filter(|entity| fields.get(*entity).is_ok());
    let lifecycle_active = navigation.current_screen == Some(ScreenId::Lifecycle);

    for (entity, field, mut border, mut background) in &mut fields {
        let selected = lifecycle_active && navigation.selected == field.index;
        border.set_all(if focused == Some(entity) || selected {
            THEME_ACCENT
        } else {
            THEME_MUTED
        });
        *background = BackgroundColor(THEME_PANEL_ALT);
    }
}

#[cfg(test)]
#[path = "../tests/unit/bevy_presentation.rs"]
mod tests;
