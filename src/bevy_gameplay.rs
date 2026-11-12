//! Bevy presentation and interaction for the spatial pause menu and World Map.
//!
//! The former Gameplay dashboard has been retired. This adapter now owns only
//! navigation overlays hosted by the active spatial exploration scene.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Pressed, UiTransform};
use bevy::ui_widgets::Button;
use bevy::window::PrimaryWindow;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{self, GameplayInputQueue, NavigationState, ScreenId};
use crate::game::{actions, world_map};
use crate::input::InputEvent;
use crate::presentation::WorldMapView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameplayMenuScreen {
    Closed,
    WorldMap,
    Pause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PauseAction {
    Resume,
    Map,
    Meditate,
    Character,
    NewGame,
    LoadGame,
    Options,
    Quit,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GameplayRuntimeState {
    paused: bool,
}

impl GameplayRuntimeState {
    pub(crate) fn is_paused(self) -> bool {
        self.paused
    }

    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct GameplayMenuChoice {
    screen: GameplayMenuScreen,
    index: usize,
}

#[derive(Resource)]
pub(crate) struct GameplayMenuState {
    pub(crate) screen: GameplayMenuScreen,
    pub(crate) selected: usize,
    pub(crate) pause_selected: usize,
    pub(crate) message: Option<String>,
}

impl Default for GameplayMenuState {
    fn default() -> Self {
        Self {
            screen: GameplayMenuScreen::Closed,
            selected: 0,
            pause_selected: 0,
            message: None,
        }
    }
}

#[derive(Component)]
struct WorldMapViewport;

#[derive(Component)]
struct WorldMapCanvas {
    size: Vec2,
    current_location_center: Vec2,
}

#[derive(Component)]
struct WorldMapMapNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldMapViewAction {
    ZoomIn,
    ZoomOut,
    Fit,
    Reset,
}

#[derive(Component)]
struct WorldMapViewControl(WorldMapViewAction);

#[derive(Clone, Copy)]
struct ActiveWorldMapTouch {
    start: Vec2,
    last: Vec2,
    dragging: bool,
    button: Option<Entity>,
}

#[derive(Clone, Copy)]
struct ActiveWorldMapMouse {
    start: Vec2,
    last: Vec2,
    dragging: bool,
    button: Option<Entity>,
}

#[derive(Clone, Copy)]
struct ActiveWorldMapPinch {
    ids: [u64; 2],
    last_distance: f32,
    last_midpoint: Vec2,
}

#[derive(Resource, Default)]
struct WorldMapViewportState {
    viewport: Option<Entity>,
    zoom: f32,
    pan: Vec2,
    default_zoom: f32,
    default_pan: Vec2,
    requested_action: Option<WorldMapViewAction>,
    touches: HashMap<u64, ActiveWorldMapTouch>,
    pinch: Option<ActiveWorldMapPinch>,
    mouse: Option<ActiveWorldMapMouse>,
}

const WORLD_MAP_MIN_ZOOM: f32 = 0.05;
const WORLD_MAP_MAX_ZOOM: f32 = 3.0;
const WORLD_MAP_ZOOM_STEP: f32 = 1.2;
const WORLD_MAP_GESTURE_THRESHOLD: f32 = 12.0;
const WORLD_MAP_FIT_MARGIN: f32 = 0.92;

fn world_map_fit_zoom(canvas_size: Vec2, viewport_size: Vec2) -> f32 {
    if canvas_size.min_element() <= 0.0 || viewport_size.min_element() <= 0.0 {
        return 1.0;
    }
    ((viewport_size.x / canvas_size.x).min(viewport_size.y / canvas_size.y)
        * WORLD_MAP_FIT_MARGIN)
        .clamp(WORLD_MAP_MIN_ZOOM, WORLD_MAP_MAX_ZOOM)
}

fn world_map_clamp_pan(pan: Vec2, zoom: f32, canvas_size: Vec2, viewport_size: Vec2) -> Vec2 {
    let clamp_axis = |value: f32, canvas: f32, viewport: f32| {
        let overflow = canvas * zoom - viewport;
        if overflow <= 0.0 {
            0.0
        } else {
            value.clamp(-overflow * 0.5, overflow * 0.5)
        }
    };
    Vec2::new(
        clamp_axis(pan.x, canvas_size.x, viewport_size.x),
        clamp_axis(pan.y, canvas_size.y, viewport_size.y),
    )
}

fn world_map_initial_pan(
    canvas_size: Vec2,
    viewport_size: Vec2,
    current_location_center: Vec2,
    zoom: f32,
) -> Vec2 {
    world_map_clamp_pan(
        (canvas_size * 0.5 - current_location_center) * zoom,
        zoom,
        canvas_size,
        viewport_size,
    )
}

fn world_map_zoom_pan(
    pan: Vec2,
    old_zoom: f32,
    new_zoom: f32,
    previous_anchor: Vec2,
    current_anchor: Vec2,
) -> Vec2 {
    if old_zoom <= 0.0 {
        return pan;
    }
    let map_point_from_center = (previous_anchor - pan) / old_zoom;
    current_anchor - map_point_from_center * new_zoom
}

fn world_map_viewport_anchor(
    viewport: &ComputedNode,
    transform: &UiGlobalTransform,
    logical_position: Vec2,
    scale_factor: f32,
) -> Option<Vec2> {
    let physical_position =
        bevy_presentation::physical_touch_position(logical_position, scale_factor);
    let normalized = viewport.normalize_point(*transform, physical_position)?;
    let logical_size = viewport.content_size * viewport.inverse_scale_factor();
    Some(normalized * logical_size)
}

fn world_map_control_input(
    mut activations: MessageReader<bevy_presentation::UiButtonActivation>,
    controls: Query<&WorldMapViewControl>,
    navigation: Res<NavigationState>,
    gameplay: Res<GameplayMenuState>,
    mut viewport: ResMut<WorldMapViewportState>,
) {
    if navigation.current_screen != Some(ScreenId::SpatialExploration)
        || gameplay.screen != GameplayMenuScreen::WorldMap
    {
        return;
    }

    for activation in activations.read() {
        if let Ok(control) = controls.get(activation.0) {
            viewport.requested_action = Some(control.0);
        }
    }
}

fn world_map_viewport_input(
    mut commands: Commands,
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    window: Single<&Window, With<PrimaryWindow>>,
    navigation: Res<NavigationState>,
    gameplay: Res<GameplayMenuState>,
    mut viewport_state: ResMut<WorldMapViewportState>,
    mut touch_scroll: ResMut<bevy_presentation::TouchScrollState>,
    viewports: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<WorldMapViewport>>,
    mut canvases: Query<(&mut UiTransform, &WorldMapCanvas), With<WorldMapCanvas>>,
    map_buttons: Query<
        (Entity, &ComputedNode, &UiGlobalTransform),
        (With<Button>, With<WorldMapMapNode>),
    >,
) {
    touch_scroll.clear_external_suppression();

    if navigation.current_screen != Some(ScreenId::SpatialExploration)
        || gameplay.screen != GameplayMenuScreen::WorldMap
    {
        viewport_state.viewport = None;
        viewport_state.touches.clear();
        viewport_state.pinch = None;
        viewport_state.mouse = None;
        viewport_state.requested_action = None;
        return;
    }

    let Some((viewport_entity, viewport_node, viewport_transform)) =
        viewports.iter().next()
    else {
        return;
    };
    let Some((mut canvas_transform, canvas)) = canvases.iter_mut().next() else {
        return;
    };

    let viewport_size = viewport_node.content_size * viewport_node.inverse_scale_factor();
    let canvas_size = canvas.size;
    if viewport_size.min_element() <= 0.0 || canvas_size.min_element() <= 0.0 {
        return;
    }

    if viewport_state.viewport != Some(viewport_entity) {
        let default_zoom = 1.0;
        let default_pan = world_map_initial_pan(
            canvas_size,
            viewport_size,
            canvas.current_location_center,
            default_zoom,
        );
        viewport_state.viewport = Some(viewport_entity);
        viewport_state.zoom = default_zoom;
        viewport_state.pan = default_pan;
        viewport_state.default_zoom = default_zoom;
        viewport_state.default_pan = default_pan;
        viewport_state.touches.clear();
        viewport_state.pinch = None;
        viewport_state.mouse = None;
    }

    if let Some(action) = viewport_state.requested_action.take() {
        match action {
            WorldMapViewAction::ZoomIn => {
                let next = (viewport_state.zoom * WORLD_MAP_ZOOM_STEP)
                    .clamp(WORLD_MAP_MIN_ZOOM, WORLD_MAP_MAX_ZOOM);
                viewport_state.pan = world_map_zoom_pan(
                    viewport_state.pan,
                    viewport_state.zoom,
                    next,
                    Vec2::ZERO,
                    Vec2::ZERO,
                );
                viewport_state.zoom = next;
            }
            WorldMapViewAction::ZoomOut => {
                let next = (viewport_state.zoom / WORLD_MAP_ZOOM_STEP)
                    .clamp(WORLD_MAP_MIN_ZOOM, WORLD_MAP_MAX_ZOOM);
                viewport_state.pan = world_map_zoom_pan(
                    viewport_state.pan,
                    viewport_state.zoom,
                    next,
                    Vec2::ZERO,
                    Vec2::ZERO,
                );
                viewport_state.zoom = next;
            }
            WorldMapViewAction::Fit => {
                viewport_state.zoom = world_map_fit_zoom(canvas_size, viewport_size);
                viewport_state.pan = Vec2::ZERO;
            }
            WorldMapViewAction::Reset => {
                viewport_state.zoom = viewport_state.default_zoom;
                viewport_state.pan = viewport_state.default_pan;
            }
        }
    }

    if let Some(cursor) = window.cursor_position() {
        let physical =
            bevy_presentation::physical_touch_position(cursor, window.scale_factor());
        if viewport_node.contains_point(*viewport_transform, physical) {
            if let Some(anchor) = world_map_viewport_anchor(
                viewport_node,
                viewport_transform,
                cursor,
                window.scale_factor(),
            ) {
                for event in mouse_wheel.read() {
                    let delta = match event.unit {
                        MouseScrollUnit::Line => event.y,
                        MouseScrollUnit::Pixel => event.y / 120.0,
                    };
                    if delta == 0.0 {
                        continue;
                    }
                    let next = (viewport_state.zoom * WORLD_MAP_ZOOM_STEP.powf(delta))
                        .clamp(WORLD_MAP_MIN_ZOOM, WORLD_MAP_MAX_ZOOM);
                    viewport_state.pan = world_map_zoom_pan(
                        viewport_state.pan,
                        viewport_state.zoom,
                        next,
                        anchor,
                        anchor,
                    );
                    viewport_state.zoom = next;
                }
            } else {
                mouse_wheel.clear();
            }
        } else {
            mouse_wheel.clear();
        }
    } else {
        mouse_wheel.clear();
    }

    if let Some(cursor) = window.cursor_position() {
        if mouse.just_pressed(MouseButton::Left) {
            let physical =
                bevy_presentation::physical_touch_position(cursor, window.scale_factor());
            if viewport_node.contains_point(*viewport_transform, physical) {
                let button = map_buttons
                    .iter()
                    .find(|(_, computed, transform)| {
                        computed.contains_point(**transform, physical)
                    })
                    .map(|(entity, _, _)| entity);
                viewport_state.mouse = Some(ActiveWorldMapMouse {
                    start: cursor,
                    last: cursor,
                    dragging: false,
                    button,
                });
            }
        }

        if let Some(mut active) = viewport_state.mouse {
            if mouse.pressed(MouseButton::Left) {
                if !active.dragging
                    && cursor.distance(active.start) > WORLD_MAP_GESTURE_THRESHOLD
                {
                    active.dragging = true;
                }
                if active.dragging {
                    viewport_state.pan = world_map_clamp_pan(
                        viewport_state.pan + cursor - active.last,
                        viewport_state.zoom,
                        canvas_size,
                        viewport_size,
                    );
                    if let Some(button) = active.button {
                        commands.entity(button).remove::<Pressed>();
                        touch_scroll.suppress_external_activation(button);
                    }
                }
                active.last = cursor;
                viewport_state.mouse = Some(active);
            } else {
                viewport_state.mouse = None;
            }
        }
    } else if !mouse.pressed(MouseButton::Left) {
        viewport_state.mouse = None;
    }

    for touch in touches.iter_just_pressed() {
        let physical =
            bevy_presentation::physical_touch_position(touch.position(), window.scale_factor());
        if !viewport_node.contains_point(*viewport_transform, physical) {
            continue;
        }
        let button = map_buttons
            .iter()
            .find(|(_, computed, transform)| {
                computed.contains_point(**transform, physical)
            })
            .map(|(entity, _, _)| entity);
        viewport_state.touches.insert(
            touch.id(),
            ActiveWorldMapTouch {
                start: touch.position(),
                last: touch.position(),
                dragging: false,
                button,
            },
        );
    }

    let mut active_ids = viewport_state
        .touches
        .keys()
        .copied()
        .filter(|id| touches.iter().any(|touch| touch.id() == *id))
        .collect::<Vec<_>>();
    active_ids.sort_unstable();

    if active_ids.len() >= 2 {
        let ids = [active_ids[0], active_ids[1]];
        let first = touches
            .iter()
            .find(|touch| touch.id() == ids[0])
            .map(|touch| touch.position());
        let second = touches
            .iter()
            .find(|touch| touch.id() == ids[1])
            .map(|touch| touch.position());
        if let (Some(first), Some(second)) = (first, second) {
            let distance = first.distance(second).max(1.0);
            let midpoint = (first + second) * 0.5;
            let anchor = world_map_viewport_anchor(
                viewport_node,
                viewport_transform,
                midpoint,
                window.scale_factor(),
            )
            .unwrap_or(Vec2::ZERO);

            if let Some(previous) = viewport_state.pinch.filter(|pinch| pinch.ids == ids) {
                let next = (viewport_state.zoom * distance / previous.last_distance)
                    .clamp(WORLD_MAP_MIN_ZOOM, WORLD_MAP_MAX_ZOOM);
                viewport_state.pan = world_map_zoom_pan(
                    viewport_state.pan,
                    viewport_state.zoom,
                    next,
                    previous.last_midpoint,
                    anchor,
                );
                viewport_state.zoom = next;
            } else {
                for id in ids {
                    if let Some(button) = viewport_state
                        .touches
                        .get(&id)
                        .and_then(|touch| touch.button)
                    {
                        touch_scroll.suppress_external_activation(button);
                    }
                }
            }

            viewport_state.pinch = Some(ActiveWorldMapPinch {
                ids,
                last_distance: distance,
                last_midpoint: anchor,
            });
            for id in ids {
                if let Some(touch) = viewport_state.touches.get_mut(&id) {
                    touch.dragging = true;
                    touch.last = if id == ids[0] { first } else { second };
                    if let Some(button) = touch.button {
                        touch_scroll.suppress_external_activation(button);
                    }
                }
            }
        }
    } else if let Some(id) = active_ids.first().copied() {
        if viewport_state.pinch.take().is_some() {
            if let Some(position) = touches
                .iter()
                .find(|touch| touch.id() == id)
                .map(|touch| touch.position())
            {
                if let Some(touch) = viewport_state.touches.get_mut(&id) {
                    touch.start = position;
                    touch.last = position;
                    touch.dragging = false;
                }
            }
        } else if let Some(position) = touches
            .iter()
            .find(|touch| touch.id() == id)
            .map(|touch| touch.position())
        {
            if let Some(active) = viewport_state.touches.get(&id).copied() {
                let crossed = !active.dragging
                    && position.distance(active.start) > WORLD_MAP_GESTURE_THRESHOLD;
                if crossed {
                    if let Some(button) = active.button {
                        touch_scroll.suppress_external_activation(button);
                    }
                    viewport_state.pan = world_map_clamp_pan(
                        viewport_state.pan + position - active.start,
                        viewport_state.zoom,
                        canvas_size,
                        viewport_size,
                    );
                } else if active.dragging {
                    viewport_state.pan = world_map_clamp_pan(
                        viewport_state.pan + position - active.last,
                        viewport_state.zoom,
                        canvas_size,
                        viewport_size,
                    );
                }
                if let Some(touch) = viewport_state.touches.get_mut(&id) {
                    touch.dragging |= crossed;
                    touch.last = position;
                    if touch.dragging {
                        if let Some(button) = touch.button {
                            touch_scroll.suppress_external_activation(button);
                        }
                    }
                }
            }
        }
    } else {
        viewport_state.pinch = None;
    }

    for touch in touches.iter_just_released() {
        if let Some(active) = viewport_state.touches.remove(&touch.id()) {
            if active.dragging {
                if let Some(button) = active.button {
                    touch_scroll.suppress_external_activation(button);
                }
            }
        }
    }
    if viewport_state.touches.len() < 2 {
        viewport_state.pinch = None;
    }

    viewport_state.pan = world_map_clamp_pan(
        viewport_state.pan,
        viewport_state.zoom,
        canvas_size,
        viewport_size,
    );
    canvas_transform.translation = Val2::px(
        viewport_state.pan.x - canvas_size.x * 0.5,
        viewport_state.pan.y - canvas_size.y * 0.5,
    );
    canvas_transform.scale = Vec2::splat(viewport_state.zoom);
}

fn layout_size_center(width: f32, height: f32) -> Vec2 {
    Vec2::new(width * 0.5, height * 0.5)
}

fn spawn_world_map_view_controls(commands: &mut Commands, parent: Entity) {
    let row = commands
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
            column_gap: px(6),
            ..default()
        })
        .id();
    commands.entity(parent).add_child(row);

    for (label, action) in [
        ("-", WorldMapViewAction::ZoomOut),
        ("+", WorldMapViewAction::ZoomIn),
        ("Fit", WorldMapViewAction::Fit),
        ("Reset", WorldMapViewAction::Reset),
    ] {
        let button = commands
            .spawn((
                Button,
                Hovered::default(),
                bevy_presentation::UiTouchTargetShape::MinimumHeight,
                WorldMapViewControl(action),
                bevy_presentation::UiStyledButton,
                Node {
                    min_width: px(44),
                    height: px(40),
                    padding: UiRect::axes(px(10), px(6)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BorderColor::all(bevy_presentation::THEME_BORDER),
                BackgroundColor(bevy_presentation::THEME_SURFACE),
                children![(
                    Text::new(label),
                    bevy_presentation::TextContent,
                    TextFont::from_font_size(bevy_presentation::muted_font_size()),
                    TextColor(bevy_presentation::THEME_TEXT),
                )],
            ))
            .id();
        commands.entity(row).add_child(button);
    }
}


pub(crate) fn install(app: &mut App) {
    app.init_resource::<GameplayMenuState>()
        .init_resource::<GameplayRuntimeState>()
        .init_resource::<WorldMapViewportState>()
        .add_systems(
            PreUpdate,
            world_map_viewport_input.after(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            Update,
            (
                gameplay_input,
                world_map_control_input,
                sync_gameplay_menu_selection,
            )
                .chain(),
        );
}

fn accepts_gameplay_input(
    current_screen: Option<ScreenId>,
    menu_screen: GameplayMenuScreen,
) -> bool {
    current_screen == Some(ScreenId::SpatialExploration)
        && matches!(
            menu_screen,
            GameplayMenuScreen::Pause | GameplayMenuScreen::WorldMap
        )
}

fn gameplay_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut gameplay: ResMut<GameplayMenuState>,
    mut runtime_state: ResMut<GameplayRuntimeState>,
    mut navigation_state: ResMut<NavigationState>,
    mut spatial_state: ResMut<crate::bevy_spatial::SpatialExplorationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || lifecycle.session.is_none()
        || !accepts_gameplay_input(navigation_state.current_screen, gameplay.screen)
    {
        return;
    }
    if input_queue.0.is_empty() {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => move_selection(&mut gameplay, &mut navigation_state, -1, &lifecycle),
            InputEvent::Down => move_selection(&mut gameplay, &mut navigation_state, 1, &lifecycle),
            InputEvent::Home => {
                if gameplay.screen == GameplayMenuScreen::Pause {
                    gameplay.pause_selected = 0;
                } else {
                    gameplay.selected = 0;
                }
                navigation_state.selected = 0;
            }
            InputEvent::End => {
                move_selection_to_end(&mut gameplay, &mut navigation_state, &lifecycle)
            }
            InputEvent::Cancel | InputEvent::Backspace => match gameplay.screen {
                GameplayMenuScreen::WorldMap => {
                    gameplay.screen = GameplayMenuScreen::Pause;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    navigation_state.selected = gameplay.pause_selected;
                    spatial_state.request_presentation_refresh();
                }
                GameplayMenuScreen::Pause => {
                    close_pause(&mut gameplay, &mut runtime_state, &mut navigation_state);
                    spatial_state.request_presentation_refresh();
                }
                GameplayMenuScreen::Closed => {}
            },
            InputEvent::Confirm => {
                activate_selection(
                    &mut lifecycle,
                    &mut gameplay,
                    &mut runtime_state,
                    &mut navigation_state,
                    &mut spatial_state,
                );
            }
            _ => {}
        }
    }
}

fn move_selection(
    gameplay: &mut GameplayMenuState,
    navigation_state: &mut NavigationState,
    direction: isize,
    lifecycle: &LifecycleState,
) {
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let count = match gameplay.screen {
        GameplayMenuScreen::Closed => 0,
        GameplayMenuScreen::WorldMap => {
            world_map::build_view(&session.state)
                .fast_travel_destinations
                .len()
                + 1
        }
        GameplayMenuScreen::Pause => pause_action_count(meditation_is_available(lifecycle)),
    };
    if count == 0 {
        return;
    }
    if gameplay.screen == GameplayMenuScreen::Pause {
        gameplay.pause_selected =
            (gameplay.pause_selected as isize + direction).rem_euclid(count as isize) as usize;
        navigation_state.selected = gameplay.pause_selected;
    } else {
        gameplay.selected =
            (gameplay.selected as isize + direction).rem_euclid(count as isize) as usize;
        navigation_state.selected = gameplay.selected;
    }
}

fn move_selection_to_end(
    gameplay: &mut GameplayMenuState,
    navigation_state: &mut NavigationState,
    lifecycle: &LifecycleState,
) {
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let count = match gameplay.screen {
        GameplayMenuScreen::Closed => 0,
        GameplayMenuScreen::WorldMap => {
            world_map::build_view(&session.state)
                .fast_travel_destinations
                .len()
                + 1
        }
        GameplayMenuScreen::Pause => pause_action_count(meditation_is_available(lifecycle)),
    };
    if count > 0 {
        if gameplay.screen == GameplayMenuScreen::Pause {
            gameplay.pause_selected = count - 1;
            navigation_state.selected = gameplay.pause_selected;
        } else {
            gameplay.selected = count - 1;
            navigation_state.selected = gameplay.selected;
        }
    }
}

pub(crate) fn open_pause(
    gameplay: &mut GameplayMenuState,
    runtime_state: &mut GameplayRuntimeState,
    navigation_state: &mut NavigationState,
) {
    gameplay.screen = GameplayMenuScreen::Pause;
    gameplay.pause_selected = 0;
    runtime_state.set_paused(true);
    navigation_state.selected = 0;
}

fn close_pause(
    gameplay: &mut GameplayMenuState,
    runtime_state: &mut GameplayRuntimeState,
    navigation_state: &mut NavigationState,
) {
    gameplay.screen = GameplayMenuScreen::Closed;
    runtime_state.set_paused(false);
    navigation_state.current_screen = Some(ScreenId::SpatialExploration);
    navigation_state.selected = 0;
}

fn sync_gameplay_menu_selection(
    navigation_state: Res<NavigationState>,
    gameplay: Res<GameplayMenuState>,
    mut commands: Commands,
    buttons: Query<(
        Entity,
        &GameplayMenuChoice,
        Option<&bevy_presentation::UiSelected>,
    )>,
) {
    if !matches!(
        navigation_state.current_screen,
        Some(ScreenId::SpatialExploration)
    ) {
        return;
    }
    let selected_index = match gameplay.screen {
        GameplayMenuScreen::Pause => gameplay.pause_selected,
        GameplayMenuScreen::WorldMap => gameplay.selected,
        GameplayMenuScreen::Closed => return,
    };
    for (entity, choice, selected) in &buttons {
        let should_select = choice.screen == gameplay.screen && choice.index == selected_index;
        if should_select != selected.is_some() {
            if should_select {
                commands
                    .entity(entity)
                    .insert(bevy_presentation::UiSelected);
            } else {
                commands
                    .entity(entity)
                    .remove::<bevy_presentation::UiSelected>();
            }
        }
    }
}

fn meditation_is_available(lifecycle: &LifecycleState) -> bool {
    lifecycle
        .session
        .as_ref()
        .is_some_and(|session| actions::build_meditation_view(&session.state).safe_to_meditate)
}

fn pause_action_count(meditation_available: bool) -> usize {
    pause_actions_from(
        crate::bevy_lifecycle::has_available_saves(),
        meditation_available,
    )
    .len()
}

fn pause_actions(meditation_available: bool) -> Vec<PauseAction> {
    pause_actions_from(
        crate::bevy_lifecycle::has_available_saves(),
        meditation_available,
    )
}

fn pause_actions_from(has_load: bool, meditation_available: bool) -> Vec<PauseAction> {
    let mut actions = vec![PauseAction::Resume, PauseAction::Map];
    if meditation_available {
        actions.push(PauseAction::Meditate);
    }
    actions.extend([PauseAction::Character, PauseAction::NewGame]);
    if has_load {
        actions.push(PauseAction::LoadGame);
    }
    actions.push(PauseAction::Options);
    actions.push(PauseAction::Quit);
    actions
}

fn open_dedicated_screen(navigation_state: &mut NavigationState, screen: ScreenId) {
    navigation_state.open_screen(screen);
}

fn activate_selection(
    lifecycle: &mut LifecycleState,
    gameplay: &mut GameplayMenuState,
    runtime_state: &mut GameplayRuntimeState,
    navigation_state: &mut NavigationState,
    spatial_state: &mut crate::bevy_spatial::SpatialExplorationState,
) {
    match gameplay.screen {
        GameplayMenuScreen::Pause => {
            let actions = pause_actions(meditation_is_available(lifecycle));
            let Some(action) = actions.get(gameplay.pause_selected).copied() else {
                gameplay.pause_selected = 0;
                navigation_state.selected = 0;
                return;
            };

            match action {
                PauseAction::Meditate => {
                    open_dedicated_screen(navigation_state, ScreenId::Meditation);
                }
                PauseAction::Map => {
                    gameplay.screen = GameplayMenuScreen::WorldMap;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    navigation_state.selected = 0;
                    spatial_state.request_presentation_refresh();
                }
                PauseAction::Resume => {
                    close_pause(gameplay, runtime_state, navigation_state);
                    spatial_state.request_presentation_refresh();
                }
                PauseAction::Character => {
                    open_dedicated_screen(navigation_state, ScreenId::Character);
                }
                PauseAction::NewGame => {
                    gameplay.screen = GameplayMenuScreen::Closed;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.pause_selected = 0;
                    runtime_state.set_paused(false);
                    lifecycle.start_new_game(navigation_state);
                }
                PauseAction::LoadGame => {
                    if lifecycle.start_load_game(navigation_state) {
                        gameplay.screen = GameplayMenuScreen::Closed;
                        gameplay.selected = 0;
                        gameplay.message = None;
                        gameplay.pause_selected = 0;
                        runtime_state.set_paused(false);
                    }
                }
                PauseAction::Options => {
                    open_dedicated_screen(navigation_state, ScreenId::Options);
                }
                PauseAction::Quit => {
                    lifecycle.start_quit_confirmation(
                        navigation_state,
                        Some(ScreenId::SpatialExploration),
                    );
                }
            }
        }
        GameplayMenuScreen::WorldMap => {
            let map_view = lifecycle
                .session
                .as_ref()
                .map(|session| world_map::build_view(&session.state));
            let Some(destination) = map_view
                .as_ref()
                .and_then(|view| view.fast_travel_destinations.get(gameplay.selected))
            else {
                gameplay.screen = GameplayMenuScreen::Pause;
                gameplay.selected = 0;
                gameplay.message = None;
                navigation_state.selected = gameplay.pause_selected;
                spatial_state.request_presentation_refresh();
                return;
            };
            let target_id = destination.location.id;

            match crate::bevy_spatial::fast_travel_to_known_location(
                spatial_state,
                lifecycle,
                navigation_state,
                target_id,
            ) {
                Ok(()) => {
                    gameplay.screen = GameplayMenuScreen::Closed;
                    gameplay.selected = 0;
                    gameplay.pause_selected = 0;
                    gameplay.message = None;
                    runtime_state.set_paused(false);
                    if lifecycle.phase == LifecyclePhase::Death {
                        navigation_state.current_screen = Some(ScreenId::Lifecycle);
                        navigation_state.return_screen = None;
                        navigation_state.selected = lifecycle.selected;
                    }
                }
                Err(message) => {
                    gameplay.message = Some(message.to_string());
                    spatial_state.request_presentation_refresh();
                }
            }
        }
        GameplayMenuScreen::Closed => {}
    }
}

pub(crate) fn render_spatial_pause_overlay(
    commands: &mut Commands,
    parent: Entity,
    selected: usize,
    meditation_available: bool,
) {
    let actions = pause_actions(meditation_available);
    render_pause(commands, parent, &actions, selected);
}

fn render_pause(commands: &mut Commands, parent: Entity, actions: &[PauseAction], selected: usize) {
    let overlay = bevy_presentation::spawn_overlay(commands, parent);
    commands
        .entity(overlay)
        .insert(BackgroundColor(bevy_presentation::THEME_OVERLAY));

    let surface = commands
        .spawn((
            bevy_presentation::UiSurface,
            bevy_presentation::TouchScrollablePanel,
            Node {
                width: percent(60),
                min_width: px(280),
                max_width: px(620),
                max_height: percent(90),
                min_height: px(0),
                padding: UiRect::all(bevy_presentation::responsive_surface_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: bevy_presentation::responsive_compact_gap(),
                overflow: Overflow::scroll(),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    commands.entity(overlay).add_child(surface);

    bevy_presentation::spawn_label(commands, surface, "PAUSED");
    bevy_presentation::spawn_muted_label(
        commands,
        surface,
        "The chronicle waits. Resume your journey or choose another action.",
    );

    for (index, action) in actions.iter().enumerate() {
        let (icon, label) = pause_action_visuals(*action);
        let button = bevy_presentation::spawn_action_button(commands, surface, index, icon, label);
        commands.entity(button).insert(GameplayMenuChoice {
            screen: GameplayMenuScreen::Pause,
            index,
        });
        if index == selected {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }
}

fn pause_action_visuals(action: PauseAction) -> (&'static str, &'static str) {
    match action {
        PauseAction::Resume => ("resume", "Resume"),
        PauseAction::Map => ("travel", "Map"),
        PauseAction::Meditate => ("meditate", "Meditate"),
        PauseAction::Character => ("character", "Character"),
        PauseAction::NewGame => ("new-game", "New Game"),
        PauseAction::LoadGame => ("load-game", "Load Game"),
        PauseAction::Options => ("options", "Options"),
        PauseAction::Quit => ("quit", "Quit"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct WorldMapNodePlacement {
    location_id: u64,
    column: usize,
    left: f32,
    top: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct WorldMapLayout {
    nodes: Vec<WorldMapNodePlacement>,
    width: f32,
    height: f32,
}

const WORLD_MAP_NODE_WIDTH: f32 = 204.0;
const WORLD_MAP_NODE_HEIGHT: f32 = 84.0;
const WORLD_MAP_COLUMN_GAP: f32 = 104.0;
const WORLD_MAP_ROW_GAP: f32 = 30.0;
const WORLD_MAP_PADDING: f32 = 28.0;

fn build_world_map_layout(view: &WorldMapView) -> WorldMapLayout {
    let node_ids = view
        .nodes
        .iter()
        .map(|node| node.location.id)
        .collect::<HashSet<_>>();
    let mut adjacency = HashMap::<u64, Vec<u64>>::new();
    for id in &node_ids {
        adjacency.entry(*id).or_default();
    }
    for route in &view.routes {
        if node_ids.contains(&route.from_id) && node_ids.contains(&route.to_id) {
            adjacency
                .entry(route.from_id)
                .or_default()
                .push(route.to_id);
            adjacency
                .entry(route.to_id)
                .or_default()
                .push(route.from_id);
        }
    }
    for neighbours in adjacency.values_mut() {
        neighbours.sort_unstable();
        neighbours.dedup();
    }

    let mut distances = HashMap::<u64, usize>::new();
    if let Some(current_id) = view.current_location.as_ref().map(|location| location.id) {
        if node_ids.contains(&current_id) {
            assign_world_map_component(current_id, 0, &adjacency, &mut distances);
        }
    }

    let mut next_component_column = distances.values().copied().max().unwrap_or(0) + 1;
    let mut remaining = view
        .nodes
        .iter()
        .map(|node| node.location.id)
        .collect::<Vec<_>>();
    remaining.sort_unstable();
    for id in remaining {
        if !distances.contains_key(&id) {
            assign_world_map_component(id, next_component_column, &adjacency, &mut distances);
            next_component_column = distances.values().copied().max().unwrap_or(0) + 1;
        }
    }

    let nodes_by_id = view
        .nodes
        .iter()
        .map(|node| (node.location.id, node))
        .collect::<HashMap<_, _>>();
    let mut columns = BTreeMap::<usize, Vec<u64>>::new();
    for (id, column) in &distances {
        columns.entry(*column).or_default().push(*id);
    }
    for ids in columns.values_mut() {
        ids.sort_by(|left, right| {
            let left_node = nodes_by_id[left];
            let right_node = nodes_by_id[right];
            left_node
                .region_id
                .cmp(&right_node.region_id)
                .then_with(|| {
                    left_node
                        .location
                        .region_name
                        .cmp(&right_node.location.region_name)
                })
                .then_with(|| left.cmp(right))
        });
    }

    let max_column = columns.keys().copied().max().unwrap_or(0);
    let max_rows = columns.values().map(Vec::len).max().unwrap_or(1);
    let width = WORLD_MAP_PADDING * 2.0
        + (max_column + 1) as f32 * WORLD_MAP_NODE_WIDTH
        + max_column as f32 * WORLD_MAP_COLUMN_GAP;
    let height = (WORLD_MAP_PADDING * 2.0
        + max_rows as f32 * WORLD_MAP_NODE_HEIGHT
        + max_rows.saturating_sub(1) as f32 * WORLD_MAP_ROW_GAP)
        .max(300.0);

    let mut placements = Vec::with_capacity(view.nodes.len());
    for (column, ids) in columns {
        for (row, location_id) in ids.into_iter().enumerate() {
            placements.push(WorldMapNodePlacement {
                location_id,
                column,
                left: WORLD_MAP_PADDING
                    + column as f32 * (WORLD_MAP_NODE_WIDTH + WORLD_MAP_COLUMN_GAP),
                top: WORLD_MAP_PADDING + row as f32 * (WORLD_MAP_NODE_HEIGHT + WORLD_MAP_ROW_GAP),
            });
        }
    }
    placements.sort_by_key(|node| node.location_id);

    WorldMapLayout {
        nodes: placements,
        width,
        height,
    }
}

fn assign_world_map_component(
    start: u64,
    start_column: usize,
    adjacency: &HashMap<u64, Vec<u64>>,
    distances: &mut HashMap<u64, usize>,
) {
    if distances.contains_key(&start) {
        return;
    }
    let mut queue = VecDeque::from([start]);
    distances.insert(start, start_column);
    while let Some(current) = queue.pop_front() {
        let next_column = distances[&current] + 1;
        if let Some(neighbours) = adjacency.get(&current) {
            for neighbour in neighbours {
                if !distances.contains_key(neighbour) {
                    distances.insert(*neighbour, next_column);
                    queue.push_back(*neighbour);
                }
            }
        }
    }
}

pub(crate) fn render_spatial_world_map_overlay(
    commands: &mut Commands,
    parent: Entity,
    view: &WorldMapView,
    selected: usize,
    message: Option<&str>,
) {
    let overlay = bevy_presentation::spawn_overlay(commands, parent);
    commands
        .entity(overlay)
        .insert(BackgroundColor(bevy_presentation::THEME_OVERLAY));

    let surface = commands
        .spawn((
            bevy_presentation::UiSurface,
            Node {
                width: percent(92),
                min_width: px(280),
                max_width: px(1120),
                height: percent(92),
                max_height: percent(94),
                min_height: px(0),
                padding: UiRect::all(bevy_presentation::responsive_surface_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: bevy_presentation::responsive_compact_gap(),
                overflow: Overflow::clip(),
                flex_grow: 1.0,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    commands.entity(overlay).add_child(surface);

    render_world_map_content(commands, surface, view, selected, message);
}

fn render_world_map_content(
    commands: &mut Commands,
    panel: Entity,
    view: &WorldMapView,
    selected: usize,
    message: Option<&str>,
) {
    bevy_presentation::spawn_label(commands, panel, "WORLD MAP");
    if let Some(current) = &view.current_location {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            format!("CURRENT REGION  ·  {}", current.region_name),
        );
        bevy_presentation::spawn_label(commands, panel, current.name.clone());
    } else {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "Your current location can no longer be resolved.",
        );
    }
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "Select a previously visited place to fast-travel. Unexplored places must be reached in the world.",
    );
    if let Some(message) = message {
        bevy_presentation::spawn_muted_label(commands, panel, message);
    }

    render_world_map_graph(commands, panel, view, selected);

    if view.fast_travel_destinations.is_empty() {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "No previously visited locations are reachable through known routes.",
        );
    }
    let back_index = view.fast_travel_destinations.len();
    let button =
        bevy_presentation::spawn_action_button(commands, panel, back_index, "back", "Back");
    commands.entity(button).insert(GameplayMenuChoice {
        screen: GameplayMenuScreen::WorldMap,
        index: back_index,
    });
    if selected == back_index {
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
    }
}

fn render_world_map_graph(
    commands: &mut Commands,
    parent: Entity,
    view: &WorldMapView,
    selected: usize,
) {
    let layout = build_world_map_layout(view);
    spawn_world_map_view_controls(commands, parent);

    let current_location_center = layout
        .nodes
        .iter()
        .find(|placement| {
            view.current_location
                .as_ref()
                .is_some_and(|current| current.id == placement.location_id)
        })
        .map(|placement| {
            Vec2::new(
                placement.left + WORLD_MAP_NODE_WIDTH * 0.5,
                placement.top + WORLD_MAP_NODE_HEIGHT * 0.5,
            )
        })
        .unwrap_or_else(|| layout_size_center(layout.width, layout.height));

    let viewport = commands
        .spawn((
            WorldMapViewport,
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(180),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                padding: UiRect::all(px(6)),
                overflow: Overflow::clip(),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_PANEL),
        ))
        .id();
    commands.entity(parent).add_child(viewport);

    let canvas = commands
        .spawn((
            WorldMapCanvas {
                size: Vec2::new(layout.width, layout.height),
                current_location_center,
            },
            UiTransform {
                translation: Val2::px(-layout.width * 0.5, -layout.height * 0.5),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                width: px(layout.width),
                height: px(layout.height),
                min_width: px(layout.width),
                min_height: px(layout.height),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_PANEL),
        ))
        .id();
    commands.entity(viewport).add_child(canvas);

    let placement_by_id = layout
        .nodes
        .iter()
        .map(|placement| (placement.location_id, placement))
        .collect::<HashMap<_, _>>();
    let node_by_id = view
        .nodes
        .iter()
        .map(|node| (node.location.id, node))
        .collect::<HashMap<_, _>>();

    for route in &view.routes {
        let (Some(from), Some(to)) = (
            placement_by_id.get(&route.from_id),
            placement_by_id.get(&route.to_id),
        ) else {
            continue;
        };
        let line_color = if route.bidirectional {
            Color::srgba(0.52, 0.42, 0.29, 0.76)
        } else {
            Color::srgba(0.72, 0.62, 0.46, 0.88)
        };
        let from_y = from.top + WORLD_MAP_NODE_HEIGHT / 2.0;
        let to_y = to.top + WORLD_MAP_NODE_HEIGHT / 2.0;
        if from.column == to.column {
            let rail_x = from.left + WORLD_MAP_NODE_WIDTH + 18.0;
            spawn_world_map_segment(
                commands,
                canvas,
                from.left + WORLD_MAP_NODE_WIDTH,
                from_y - 1.5,
                18.0,
                3.0,
                line_color,
            );
            spawn_world_map_segment(
                commands,
                canvas,
                rail_x - 1.5,
                from_y.min(to_y),
                3.0,
                (to_y - from_y).abs().max(3.0),
                line_color,
            );
            spawn_world_map_segment(
                commands,
                canvas,
                to.left + WORLD_MAP_NODE_WIDTH,
                to_y - 1.5,
                18.0,
                3.0,
                line_color,
            );
        } else {
            let (left, right, left_y, right_y) = if from.left <= to.left {
                (from.left + WORLD_MAP_NODE_WIDTH, to.left, from_y, to_y)
            } else {
                (to.left + WORLD_MAP_NODE_WIDTH, from.left, to_y, from_y)
            };
            let midpoint = (left + right) / 2.0;
            spawn_world_map_segment(
                commands,
                canvas,
                left,
                left_y - 1.5,
                (midpoint - left).max(2.0),
                3.0,
                line_color,
            );
            spawn_world_map_segment(
                commands,
                canvas,
                midpoint - 1.5,
                left_y.min(right_y),
                3.0,
                (right_y - left_y).abs().max(3.0),
                line_color,
            );
            spawn_world_map_segment(
                commands,
                canvas,
                midpoint,
                right_y - 1.5,
                (right - midpoint).max(2.0),
                3.0,
                line_color,
            );
        }
        if !route.bidirectional {
            let (marker, left, top) = if from.column < to.column {
                (">", to.left - 10.0, to_y - 9.0)
            } else if from.column > to.column {
                ("<", to.left + WORLD_MAP_NODE_WIDTH + 2.0, to_y - 9.0)
            } else if from.top < to.top {
                ("v", from.left + WORLD_MAP_NODE_WIDTH + 12.0, to_y - 9.0)
            } else {
                ("^", from.left + WORLD_MAP_NODE_WIDTH + 12.0, to_y - 9.0)
            };
            spawn_world_map_direction_marker(commands, canvas, marker, left, top, line_color);
        }
    }

    for placement in &layout.nodes {
        let Some(node) = node_by_id.get(&placement.location_id) else {
            continue;
        };
        if let (Some(index), Some(steps)) = (node.fast_travel_index, node.fast_travel_steps) {
            let route_label = if steps == 1 {
                "route step"
            } else {
                "route steps"
            };
            let detail = format!(
                "{}  ·  FAST TRAVEL  ·  {} {}",
                node.location.region_name, steps, route_label
            );
            let button = bevy_presentation::spawn_action_button_with_detail(
                commands,
                canvas,
                index,
                "travel",
                node.location.name.clone(),
                detail,
            );
            commands.entity(button).insert((
                WorldMapMapNode,
                Node {
                    width: px(WORLD_MAP_NODE_WIDTH),
                    height: px(WORLD_MAP_NODE_HEIGHT),
                    position_type: PositionType::Absolute,
                    left: px(placement.left),
                    top: px(placement.top),
                    padding: UiRect::all(px(8)),
                    border: UiRect::all(px(1)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                GameplayMenuChoice {
                    screen: GameplayMenuScreen::WorldMap,
                    index,
                },
            ));
            if selected == index {
                commands
                    .entity(button)
                    .insert(bevy_presentation::UiSelected);
            }
            continue;
        }

        let (border, background, status) = if node.current {
            (
                bevy_presentation::THEME_ACCENT,
                bevy_presentation::THEME_SELECTED,
                "CURRENT LOCATION",
            )
        } else if node.explored {
            (
                bevy_presentation::THEME_BORDER,
                bevy_presentation::THEME_SURFACE,
                "KNOWN · NO FAST-TRAVEL ROUTE",
            )
        } else {
            (
                bevy_presentation::THEME_BORDER,
                bevy_presentation::THEME_SURFACE_STRONG,
                "UNEXPLORED · REACH IN THE WORLD",
            )
        };
        let card = commands
            .spawn((
                Node {
                    width: px(WORLD_MAP_NODE_WIDTH),
                    height: px(WORLD_MAP_NODE_HEIGHT),
                    position_type: PositionType::Absolute,
                    left: px(placement.left),
                    top: px(placement.top),
                    padding: UiRect::all(px(8)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    border: UiRect::all(px(1)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderColor::all(border),
                BackgroundColor(background),
            ))
            .id();
        commands.entity(canvas).add_child(card);
        bevy_presentation::spawn_muted_label(commands, card, node.location.region_name.clone());
        bevy_presentation::spawn_label(commands, card, node.location.name.clone());
        let detail = if node.location.dangerous {
            format!("{status}  ·  DANGEROUS")
        } else {
            status.to_string()
        };
        bevy_presentation::spawn_muted_label(commands, card, detail);
    }
}

fn spawn_world_map_direction_marker(
    commands: &mut Commands,
    parent: Entity,
    marker: &str,
    left: f32,
    top: f32,
    color: Color,
) {
    let marker = commands
        .spawn((
            Text::new(marker),
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(12.0),
                height: px(18.0),
                ..default()
            },
            bevy_presentation::TextContent,
            TextFont::from_font_size(14.0),
            TextColor(color),
        ))
        .id();
    commands.entity(parent).add_child(marker);
}

fn spawn_world_map_segment(
    commands: &mut Commands,
    parent: Entity,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    color: Color,
) {
    let segment = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(width.max(1.0)),
                height: px(height.max(1.0)),
                ..default()
            },
            BackgroundColor(color),
        ))
        .id();
    commands.entity(parent).add_child(segment);
}

#[cfg(test)]
#[path = "../tests/unit/bevy_gameplay.rs"]
mod tests;
