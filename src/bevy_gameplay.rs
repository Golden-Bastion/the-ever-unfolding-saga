//! Bevy presentation and interaction for the spatial pause menu and World Map.
//!
//! The former Gameplay dashboard has been retired. This adapter now owns only
//! navigation overlays hosted by the active spatial exploration scene.

use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{self, GameplayInputQueue, NavigationState, ScreenId};
use crate::bevy_world_map_viewport;
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

pub(crate) fn install(app: &mut App) {
    app.init_resource::<GameplayMenuState>()
        .init_resource::<GameplayRuntimeState>()
        .add_systems(
            Update,
            (gameplay_input, sync_gameplay_menu_selection).chain(),
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
                overflow: Overflow::scroll(),
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
                bevy_presentation::UiManualActivation,
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
