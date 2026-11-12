//! Presentation-only pan, zoom, and gesture handling for the interim world map.
//!
//! The map graph and travel model remain owned by the gameplay/presentation
//! layers. This module only controls the clipped viewport and its canvas transform.

use bevy::input::mouse::{MouseScrollPixelsPerLine, MouseScrollUnit, MouseWheel};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Pressed, UiTransform};
use bevy::ui_widgets::Button;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

use crate::bevy_presentation::{self, NavigationState, ScreenId};

pub(crate) fn install(app: &mut App) {
    app.init_resource::<WorldMapViewportState>()
        .add_systems(
            PreUpdate,
            world_map_viewport_input.after(bevy::ui::UiSystems::Focus),
        )
        .add_systems(Update, world_map_control_input);
}

#[derive(Component)]
struct WorldMapViewport;

#[derive(Component)]
struct WorldMapCanvas {
    size: Vec2,
    current_location_center: Vec2,
}

#[derive(Component)]
pub(crate) struct WorldMapMapNode;

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
struct ActiveWorldMapGesture {
    start: Vec2,
    last: Vec2,
    dragging: bool,
    button: Option<Entity>,
}

impl ActiveWorldMapGesture {
    fn begin(position: Vec2, button: Option<Entity>) -> Self {
        Self {
            start: position,
            last: position,
            dragging: false,
            button,
        }
    }

    /// Returns the viewport-space pan delta for this input update.
    fn update(&mut self, position: Vec2) -> Vec2 {
        let crossed_drag_threshold =
            !self.dragging && position.distance(self.start) > WORLD_MAP_GESTURE_THRESHOLD;
        self.dragging |= crossed_drag_threshold;

        let delta = if crossed_drag_threshold {
            // Apply the complete movement that crossed the threshold. Discarding
            // this initial displacement makes desktop drags feel unresponsive.
            position - self.start
        } else if self.dragging {
            position - self.last
        } else {
            Vec2::ZERO
        };
        self.last = position;
        delta
    }

    /// A second pointer turns this gesture into navigation, never a node tap.
    fn begin_drag(&mut self, position: Vec2) {
        self.start = position;
        self.last = position;
        self.dragging = true;
    }

    fn release_activation(self, released_over_button: bool) -> Option<Entity> {
        if self.dragging || !released_over_button {
            None
        } else {
            self.button
        }
    }
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
    viewport_size: Vec2,
    requested_action: Option<WorldMapViewAction>,
    touches: HashMap<u64, ActiveWorldMapGesture>,
    pinch: Option<ActiveWorldMapPinch>,
    mouse: Option<ActiveWorldMapGesture>,
}

impl WorldMapViewportState {
    fn reset(&mut self) {
        self.viewport = None;
        self.viewport_size = Vec2::ZERO;
        self.touches.clear();
        self.pinch = None;
        self.mouse = None;
        self.requested_action = None;
    }
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
    ((viewport_size.x / canvas_size.x).min(viewport_size.y / canvas_size.y) * WORLD_MAP_FIT_MARGIN)
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
    let logical_size = viewport.size * viewport.inverse_scale_factor();
    Some(normalized * logical_size)
}

fn world_map_control_input(
    mut activations: MessageReader<bevy_presentation::UiButtonActivation>,
    controls: Query<&WorldMapViewControl>,
    navigation: Res<NavigationState>,
    mut viewport: ResMut<WorldMapViewportState>,
) {
    if navigation.current_screen != Some(ScreenId::SpatialExploration) {
        return;
    }

    for activation in activations.read() {
        if let Ok(control) = controls.get(activation.0) {
            viewport.requested_action = Some(control.0);
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Bevy systems require independent resources and queries.
fn world_map_viewport_input(
    mut commands: Commands,
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    scroll_pixels_per_line: Res<MouseScrollPixelsPerLine>,
    window: Single<&Window, With<PrimaryWindow>>,
    navigation: Res<NavigationState>,
    mut viewport_state: ResMut<WorldMapViewportState>,
    mut activations: MessageWriter<bevy_presentation::UiButtonActivation>,
    viewports: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<WorldMapViewport>>,
    mut canvases: Query<(&mut UiTransform, &WorldMapCanvas), With<WorldMapCanvas>>,
    map_buttons: Query<
        (Entity, &ComputedNode, &UiGlobalTransform),
        (With<Button>, With<WorldMapMapNode>),
    >,
) {

    if navigation.current_screen != Some(ScreenId::SpatialExploration) {
        viewport_state.reset();
        return;
    }

    let Some((viewport_entity, viewport_node, viewport_transform)) = viewports.iter().next() else {
        viewport_state.reset();
        return;
    };
    let Some((mut canvas_transform, canvas)) = canvases.iter_mut().next() else {
        viewport_state.reset();
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
        viewport_state.viewport_size = viewport_size;
        viewport_state.touches.clear();
        viewport_state.pinch = None;
        viewport_state.mouse = None;
    } else if viewport_state.viewport_size != viewport_size {
        viewport_state.viewport_size = viewport_size;
        viewport_state.default_pan = world_map_initial_pan(
            canvas_size,
            viewport_size,
            canvas.current_location_center,
            viewport_state.default_zoom,
        );
        viewport_state.pan = world_map_clamp_pan(
            viewport_state.pan,
            viewport_state.zoom,
            canvas_size,
            viewport_size,
        );
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
        let physical = bevy_presentation::physical_touch_position(cursor, window.scale_factor());
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
                        MouseScrollUnit::Pixel => event.y / *scroll_pixels_per_line,
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
                    .find(|(_, computed, transform)| computed.contains_point(**transform, physical))
                    .map(|(entity, _, _)| entity);
                viewport_state.mouse = Some(ActiveWorldMapGesture::begin(cursor, button));
            }
        }

        if let Some(mut active) = viewport_state.mouse {
            if mouse.pressed(MouseButton::Left) {
                let delta = active.update(cursor);
                if active.dragging {
                    viewport_state.pan = world_map_clamp_pan(
                        viewport_state.pan + delta,
                        viewport_state.zoom,
                        canvas_size,
                        viewport_size,
                    );
                    if let Some(button) = active.button {
                        commands.entity(button).remove::<Pressed>();
                    }
                }
                viewport_state.mouse = Some(active);
            } else {
                if active.dragging {
                    if let Some(button) = active.button {
                        commands.entity(button).remove::<Pressed>();
                    }
                } else if mouse.just_released(MouseButton::Left) {
                    let released_over_button = active.button.is_some_and(|button| {
                        let physical = bevy_presentation::physical_touch_position(
                            cursor,
                            window.scale_factor(),
                        );
                        map_buttons.iter().any(|(entity, computed, transform)| {
                            entity == button && computed.contains_point(*transform, physical)
                        })
                    });
                    if let Some(button) = active.release_activation(released_over_button) {
                        activations.write(bevy_presentation::UiButtonActivation(button));
                    }
                }
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
            .find(|(_, computed, transform)| computed.contains_point(**transform, physical))
            .map(|(entity, _, _)| entity);
        viewport_state.touches.insert(
            touch.id(),
            ActiveWorldMapGesture::begin(touch.position(), button),
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

            let previous_pinch_is_none = viewport_state
                .pinch
                .filter(|pinch| pinch.ids == ids)
                .is_none();
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
                        commands.entity(button).remove::<Pressed>();
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
                    let position = if id == ids[0] { first } else { second };
                    if previous_pinch_is_none {
                        touch.begin_drag(position);
                    } else {
                        touch.last = position;
                    }
                    if let Some(button) = touch.button {
                        commands.entity(button).remove::<Pressed>();
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
                    // The remaining finger may keep panning, but it must not
                    // turn the completed pinch into a destination tap.
                    touch.begin_drag(position);
                }
            }
        } else if let Some(position) = touches
            .iter()
            .find(|touch| touch.id() == id)
            .map(|touch| touch.position())
        {
            if let Some(touch) = viewport_state.touches.get_mut(&id) {
                let delta = touch.update(position);
                if touch.dragging {
                    if let Some(button) = touch.button {
                        commands.entity(button).remove::<Pressed>();
                    }
                }
                viewport_state.pan = world_map_clamp_pan(
                    viewport_state.pan + delta,
                    viewport_state.zoom,
                    canvas_size,
                    viewport_size,
                );
            }
        }
    } else {
        viewport_state.pinch = None;
    }

    for touch in touches.iter_just_released() {
        if let Some(active) = viewport_state.touches.remove(&touch.id()) {
            let physical =
                bevy_presentation::physical_touch_position(touch.position(), window.scale_factor());
            let released_over_button = active.button.is_some_and(|button| {
                map_buttons.iter().any(|(entity, computed, transform)| {
                    entity == button && computed.contains_point(*transform, physical)
                })
            });

            if active.dragging {
                if let Some(button) = active.button {
                    commands.entity(button).remove::<Pressed>();
                }
            } else if let Some(button) = active.release_activation(released_over_button) {
                activations.write(bevy_presentation::UiButtonActivation(button));
            }
        }
    }
    for touch in touches.iter_just_canceled() {
        viewport_state.touches.remove(&touch.id());
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

pub(crate) fn spawn_world_map_view_controls(commands: &mut Commands, parent: Entity) {
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
        ("−", WorldMapViewAction::ZoomOut),
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

pub(crate) fn spawn_world_map_viewport(
    commands: &mut Commands,
    parent: Entity,
    canvas_size: Vec2,
    current_location_center: Vec2,
) -> Entity {
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
                size: canvas_size,
                current_location_center,
            },
            UiTransform {
                translation: Val2::px(-canvas_size.x * 0.5, -canvas_size.y * 0.5),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                width: px(canvas_size.x),
                height: px(canvas_size.y),
                min_width: px(canvas_size.x),
                min_height: px(canvas_size.y),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_PANEL),
        ))
        .id();
    commands.entity(viewport).add_child(canvas);
    canvas
}

#[cfg(test)]
#[path = "../tests/unit/bevy_world_map_viewport.rs"]
mod tests;
