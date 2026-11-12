//! Bevy presentation and interaction for the spatial pause menu and World Map.
//!
//! The former Gameplay dashboard has been retired. This adapter now owns only
//! navigation overlays hosted by the active spatial exploration scene.

use bevy::prelude::*;
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
    bevy_world_map_viewport::spawn_world_map_view_controls(commands, parent);

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
        .unwrap_or_else(|| Vec2::new(layout.width * 0.5, layout.height * 0.5));

    let canvas = bevy_world_map_viewport::spawn_world_map_viewport(
        commands,
        parent,
        Vec2::new(layout.width, layout.height),
        current_location_center,
    );

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
                bevy_world_map_viewport::WorldMapMapNode,
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
