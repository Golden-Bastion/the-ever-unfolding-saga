//! Bevy presentation and interaction for the primary gameplay/world screen.
//!
//! This module adapts the existing gameplay state and world-navigation rules
//! into Bevy UI. It does not reimplement gameplay simulation.

use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::bevy_combat::CombatState;
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::game::{actions, menu, world_map};
use crate::input::InputEvent;
use crate::presentation::{ConditionView, WorldMapView, WorldView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameplayScreen {
    Dashboard,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameplayAction {
    Map,
    Meditate,
    Explore,
    Investigate,
}

const PRIMARY_GAMEPLAY_ACTIONS: [GameplayAction; 3] = [
    GameplayAction::Map,
    GameplayAction::Meditate,
    GameplayAction::Explore,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CelestialBody {
    Sun,
    Moon,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SkyVisual {
    pub(crate) background: Color,
    pub(crate) celestial: CelestialBody,
    pub(crate) celestial_x: f32,
    pub(crate) celestial_y: f32,
    pub(crate) warm_horizon: bool,
}

#[derive(Component, Debug, Clone, Copy)]
struct WorldCloud {
    base_x: f32,
    drift: f32,
    speed: f32,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GameplayRuntimeState {
    paused: bool,
}

#[derive(Resource, Debug, Default)]
struct LocationArtAssets {
    images: HashMap<String, Handle<Image>>,
}

#[derive(Resource, Default)]
pub(crate) struct CelestialArtAssets {
    pub(crate) image: Option<Handle<Image>>,
    pub(crate) sun_rect: Option<Rect>,
    pub(crate) moon_rect: Option<Rect>,
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
    screen: GameplayScreen,
    index: usize,
}

#[derive(Resource)]
pub(crate) struct GameplayState {
    pub(crate) screen: GameplayScreen,
    pub(crate) selected: usize,
    pub(crate) pause_selected: usize,
    pub(crate) message: Option<String>,
    dirty: bool,
}

impl Default for GameplayState {
    fn default() -> Self {
        Self {
            screen: GameplayScreen::Dashboard,
            selected: 0,
            pause_selected: 0,
            message: None,
            dirty: true,
        }
    }
}
impl GameplayState {
    pub(crate) fn navigation_selection(&self) -> usize {
        if self.screen == GameplayScreen::Pause {
            self.pause_selected
        } else {
            self.selected
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<GameplayState>()
        .init_resource::<GameplayRuntimeState>()
        .init_resource::<CelestialArtAssets>()
        .add_systems(Startup, (load_location_art, load_celestial_art))
        .add_systems(
            Update,
            (
                gameplay_input,
                render_if_active,
                sync_gameplay_menu_selection,
            )
                .chain(),
        )
        .add_systems(Update, animate_world_clouds);
}

fn load_location_art(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let campaign = crate::content::load_campaign_content();
    let mut loaded = HashMap::new();

    for location in &campaign.world.locations {
        let Some(reference) = location.scene_image.as_deref() else {
            continue;
        };
        if loaded.contains_key(reference) {
            continue;
        }

        match crate::asset_loading::load_image_from_data(&mut images, reference) {
            Ok(image) => {
                loaded.insert(reference.to_string(), image);
            }
            Err(error) => {
                warn!(
                    "Could not load location image {} for {}: {}",
                    reference, location.name, error
                );
            }
        }
    }

    commands.insert_resource(LocationArtAssets { images: loaded });
}

fn load_celestial_art(
    mut images: ResMut<Assets<Image>>,
    mut celestial: ResMut<CelestialArtAssets>,
) {
    const REFERENCE: &str = "world/celestial/sun-moon.png";
    match crate::asset_loading::load_image_from_data(&mut images, REFERENCE) {
        Ok(image) => {
            let Some(source) = images.get(&image) else {
                warn!("Could not inspect celestial world art {REFERENCE}");
                return;
            };
            let Some((sun_rect, moon_rect)) = celestial_source_rects(source.size()) else {
                warn!("Celestial world art {REFERENCE} must be a landscape two-panel image");
                return;
            };
            celestial.image = Some(image);
            celestial.sun_rect = Some(sun_rect);
            celestial.moon_rect = Some(moon_rect);
        }
        Err(error) => {
            warn!("Could not load celestial world art {REFERENCE}: {error}");
        }
    }
}

fn celestial_source_rects(size: UVec2) -> Option<(Rect, Rect)> {
    if size.x <= size.y || size.x < 2 || size.y == 0 {
        return None;
    }

    let width = size.x as f32;
    let height = size.y as f32;
    Some((
        Rect::new(width * 0.04, height * 0.10, width * 0.49, height * 0.90),
        Rect::new(width * 0.52, height * 0.10, width * 0.98, height * 0.90),
    ))
}

fn accepts_gameplay_input(
    current_screen: Option<ScreenId>,
    gameplay_screen: GameplayScreen,
) -> bool {
    current_screen == Some(ScreenId::Gameplay)
        || (current_screen == Some(ScreenId::SpatialExploration)
            && matches!(
                gameplay_screen,
                GameplayScreen::Pause | GameplayScreen::WorldMap
            ))
}

fn pause_parent_screen(current_screen: Option<ScreenId>) -> ScreenId {
    if current_screen == Some(ScreenId::SpatialExploration) {
        ScreenId::SpatialExploration
    } else {
        ScreenId::Gameplay
    }
}

fn gameplay_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut gameplay: ResMut<GameplayState>,
    mut runtime_state: ResMut<GameplayRuntimeState>,
    mut navigation_state: ResMut<NavigationState>,
    mut combat_state: ResMut<CombatState>,
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
                if gameplay.screen == GameplayScreen::Pause {
                    gameplay.pause_selected = 0;
                } else {
                    gameplay.selected = 0;
                }
                navigation_state.selected = 0;
                gameplay.dirty = true;
            }
            InputEvent::End => {
                move_selection_to_end(&mut gameplay, &mut navigation_state, &lifecycle)
            }
            InputEvent::OpenSecondaryNavigation => {
                if gameplay.screen == GameplayScreen::Dashboard {
                    open_pause(&mut gameplay, &mut runtime_state, &mut navigation_state);
                }
            }
            InputEvent::OpenDeveloperConsole => {
                if gameplay.screen == GameplayScreen::Dashboard {
                    open_dedicated_screen(&mut navigation_state, ScreenId::Console);
                }
            }
            InputEvent::Cancel | InputEvent::Backspace => match gameplay.screen {
                GameplayScreen::WorldMap => {
                    if navigation_state.current_screen == Some(ScreenId::SpatialExploration) {
                        gameplay.screen = GameplayScreen::Pause;
                        gameplay.selected = 0;
                        gameplay.message = None;
                        gameplay.dirty = true;
                        navigation_state.selected = gameplay.pause_selected;
                        spatial_state.request_presentation_refresh();
                    } else {
                        gameplay.screen = GameplayScreen::Dashboard;
                        gameplay.selected = 0;
                        gameplay.message = None;
                        gameplay.dirty = true;
                        navigation_state.selected = 0;
                    }
                }
                GameplayScreen::Pause => {
                    let return_screen = pause_parent_screen(navigation_state.current_screen);
                    close_pause(
                        &mut gameplay,
                        &mut runtime_state,
                        &mut navigation_state,
                        return_screen,
                    );
                    if return_screen == ScreenId::SpatialExploration {
                        spatial_state.request_presentation_refresh();
                    }
                }
                GameplayScreen::Dashboard => {
                    // Android BrowserBack and desktop Escape both use Cancel,
                    // so gameplay Back opens the unified menu.
                    open_pause(&mut gameplay, &mut runtime_state, &mut navigation_state);
                }
            },
            InputEvent::Confirm => {
                activate_selection(
                    &mut lifecycle,
                    &mut gameplay,
                    &mut runtime_state,
                    &mut navigation_state,
                    &mut combat_state,
                    &mut spatial_state,
                );
            }
            _ => {}
        }
    }
}

fn move_selection(
    gameplay: &mut GameplayState,
    navigation_state: &mut NavigationState,
    direction: isize,
    lifecycle: &LifecycleState,
) {
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let count = match gameplay.screen {
        GameplayScreen::Dashboard => dashboard_actions(&session.state).len(),
        GameplayScreen::WorldMap => {
            world_map::build_view(&session.state)
                .fast_travel_destinations
                .len()
                + 1
        }
        GameplayScreen::Pause => pause_action_count(
            navigation_state.current_screen == Some(ScreenId::SpatialExploration),
            meditation_is_available(lifecycle),
        ),
    };
    if count == 0 {
        return;
    }
    if gameplay.screen == GameplayScreen::Pause {
        gameplay.pause_selected =
            (gameplay.pause_selected as isize + direction).rem_euclid(count as isize) as usize;
        navigation_state.selected = gameplay.pause_selected;
    } else {
        gameplay.selected =
            (gameplay.selected as isize + direction).rem_euclid(count as isize) as usize;
        navigation_state.selected = gameplay.selected;
    }
    gameplay.dirty = true;
}

fn move_selection_to_end(
    gameplay: &mut GameplayState,
    navigation_state: &mut NavigationState,
    lifecycle: &LifecycleState,
) {
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let count = match gameplay.screen {
        GameplayScreen::Dashboard => dashboard_actions(&session.state).len(),
        GameplayScreen::WorldMap => {
            world_map::build_view(&session.state)
                .fast_travel_destinations
                .len()
                + 1
        }
        GameplayScreen::Pause => pause_action_count(
            navigation_state.current_screen == Some(ScreenId::SpatialExploration),
            meditation_is_available(lifecycle),
        ),
    };
    if count > 0 {
        if gameplay.screen == GameplayScreen::Pause {
            gameplay.pause_selected = count - 1;
            navigation_state.selected = gameplay.pause_selected;
        } else {
            gameplay.selected = count - 1;
            navigation_state.selected = gameplay.selected;
        }
        gameplay.dirty = true;
    }
}

pub(crate) fn open_pause(
    gameplay: &mut GameplayState,
    runtime_state: &mut GameplayRuntimeState,
    navigation_state: &mut NavigationState,
) {
    gameplay.screen = GameplayScreen::Pause;
    gameplay.pause_selected = 0;
    gameplay.dirty = true;
    runtime_state.set_paused(true);
    navigation_state.selected = 0;
}

fn close_pause(
    gameplay: &mut GameplayState,
    runtime_state: &mut GameplayRuntimeState,
    navigation_state: &mut NavigationState,
    return_screen: ScreenId,
) {
    gameplay.screen = GameplayScreen::Dashboard;
    gameplay.dirty = true;
    runtime_state.set_paused(false);
    navigation_state.current_screen = Some(return_screen);
    navigation_state.selected = if return_screen == ScreenId::SpatialExploration {
        0
    } else {
        gameplay.selected
    };
}

fn sync_gameplay_menu_selection(
    navigation_state: Res<NavigationState>,
    gameplay: Res<GameplayState>,
    mut commands: Commands,
    buttons: Query<(
        Entity,
        &GameplayMenuChoice,
        Option<&bevy_presentation::UiSelected>,
    )>,
) {
    if !matches!(
        navigation_state.current_screen,
        Some(ScreenId::Gameplay | ScreenId::SpatialExploration)
    ) {
        return;
    }
    let selected_index = match gameplay.screen {
        GameplayScreen::Pause => gameplay.pause_selected,
        GameplayScreen::WorldMap => gameplay.selected,
        GameplayScreen::Dashboard => return,
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

fn pause_action_count(spatial_menu: bool, meditation_available: bool) -> usize {
    pause_actions_from(
        crate::bevy_lifecycle::has_available_saves(),
        spatial_menu,
        meditation_available,
    )
    .len()
}

fn pause_actions(spatial_menu: bool, meditation_available: bool) -> Vec<PauseAction> {
    pause_actions_from(
        crate::bevy_lifecycle::has_available_saves(),
        spatial_menu,
        meditation_available,
    )
}

fn pause_actions_from(
    has_load: bool,
    spatial_menu: bool,
    meditation_available: bool,
) -> Vec<PauseAction> {
    let mut actions = vec![PauseAction::Resume];
    if spatial_menu {
        actions.push(PauseAction::Map);
        if meditation_available {
            actions.push(PauseAction::Meditate);
        }
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
    gameplay: &mut GameplayState,
    runtime_state: &mut GameplayRuntimeState,
    navigation_state: &mut NavigationState,
    combat_state: &mut CombatState,
    spatial_state: &mut crate::bevy_spatial::SpatialExplorationState,
) {
    match gameplay.screen {
        GameplayScreen::Pause => {
            let spatial_menu =
                navigation_state.current_screen == Some(ScreenId::SpatialExploration);
            let actions = pause_actions(spatial_menu, meditation_is_available(lifecycle));
            let Some(action) = actions.get(gameplay.pause_selected).copied() else {
                gameplay.pause_selected = 0;
                navigation_state.selected = 0;
                return;
            };

            match action {
                PauseAction::Meditate => {
                    gameplay.dirty = true;
                    open_dedicated_screen(navigation_state, ScreenId::Meditation);
                }
                PauseAction::Map => {
                    gameplay.screen = GameplayScreen::WorldMap;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                    navigation_state.selected = 0;
                    spatial_state.request_presentation_refresh();
                }
                PauseAction::Resume => {
                    let return_screen = pause_parent_screen(navigation_state.current_screen);
                    close_pause(gameplay, runtime_state, navigation_state, return_screen);
                    if return_screen == ScreenId::SpatialExploration {
                        spatial_state.request_presentation_refresh();
                    }
                }
                PauseAction::Character => {
                    gameplay.dirty = true;
                    open_dedicated_screen(navigation_state, ScreenId::Character);
                }
                PauseAction::NewGame => {
                    gameplay.screen = GameplayScreen::Dashboard;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.pause_selected = 0;
                    gameplay.dirty = true;
                    runtime_state.set_paused(false);
                    lifecycle.start_new_game(navigation_state);
                }
                PauseAction::LoadGame => {
                    if lifecycle.start_load_game(navigation_state) {
                        gameplay.screen = GameplayScreen::Dashboard;
                        gameplay.selected = 0;
                        gameplay.message = None;
                        gameplay.pause_selected = 0;
                        gameplay.dirty = true;
                        runtime_state.set_paused(false);
                    }
                }
                PauseAction::Options => {
                    gameplay.dirty = true;
                    open_dedicated_screen(navigation_state, ScreenId::Options);
                }
                PauseAction::Quit => {
                    let return_screen = pause_parent_screen(navigation_state.current_screen);
                    lifecycle.start_quit_confirmation(navigation_state, Some(return_screen));
                }
            }
        }
        GameplayScreen::Dashboard => {
            let Some(session) = lifecycle.session.as_mut() else {
                return;
            };
            let actions = dashboard_actions(&session.state);
            let Some(action) = actions.get(gameplay.selected).copied() else {
                return;
            };
            match action {
                GameplayAction::Map => {
                    gameplay.screen = GameplayScreen::WorldMap;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                    navigation_state.selected = 0;
                }
                GameplayAction::Investigate => {
                    match crate::bevy_combat::begin(&mut session.state, combat_state) {
                        Ok(()) => {
                            gameplay.selected = 0;
                            gameplay.message = None;
                            navigation_state.open_screen(ScreenId::Combat);
                        }
                        Err(message) => {
                            gameplay.message = Some(message);
                            gameplay.dirty = true;
                        }
                    }
                }
                GameplayAction::Meditate => {
                    reset_dashboard_selection(gameplay);
                    open_dedicated_screen(navigation_state, ScreenId::Meditation);
                }
                GameplayAction::Explore => {
                    crate::bevy_spatial::enter(spatial_state, &session.state, navigation_state);
                    gameplay.message = None;
                    gameplay.dirty = true;
                }
            }
        }
        GameplayScreen::WorldMap => {
            let map_view = lifecycle
                .session
                .as_ref()
                .map(|session| world_map::build_view(&session.state));
            let Some(destination) = map_view
                .as_ref()
                .and_then(|view| view.fast_travel_destinations.get(gameplay.selected))
            else {
                if navigation_state.current_screen == Some(ScreenId::SpatialExploration) {
                    gameplay.screen = GameplayScreen::Pause;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                    navigation_state.selected = gameplay.pause_selected;
                    spatial_state.request_presentation_refresh();
                } else {
                    gameplay.screen = GameplayScreen::Dashboard;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                    navigation_state.selected = 0;
                }
                return;
            };
            let target_id = destination.location.id;

            if navigation_state.current_screen == Some(ScreenId::SpatialExploration) {
                match crate::bevy_spatial::fast_travel_to_known_location(
                    spatial_state,
                    lifecycle,
                    navigation_state,
                    target_id,
                ) {
                    Ok(()) => {
                        gameplay.screen = GameplayScreen::Dashboard;
                        gameplay.selected = 0;
                        gameplay.pause_selected = 0;
                        gameplay.message = None;
                        gameplay.dirty = true;
                        runtime_state.set_paused(false);
                        if lifecycle.phase == LifecyclePhase::Death {
                            navigation_state.current_screen = Some(ScreenId::Lifecycle);
                            navigation_state.return_screen = None;
                            navigation_state.selected = lifecycle.selected;
                        }
                    }
                    Err(message) => {
                        gameplay.message = Some(message.to_string());
                        gameplay.dirty = true;
                        spatial_state.request_presentation_refresh();
                    }
                }
            } else {
                let Some(session) = lifecycle.session.as_mut() else {
                    return;
                };
                let old_turn = session.state.character.turn;
                match actions::fast_travel_to(&mut session.state, target_id) {
                    Ok(_) => {
                        let current_location = session
                            .state
                            .world
                            .location_by_id(session.state.character.location_id)
                            .map(|location| location.name.clone())
                            .unwrap_or_else(|| "an unknown location".to_string());
                        let message = if session.state.character.location_id != target_id {
                            Some(format!(
                                "Fast travel paused at {current_location}; an encounter or danger interrupted the journey."
                            ))
                        } else {
                            session
                                .state
                                .world
                                .history
                                .iter()
                                .rev()
                                .find(|entry| entry.turn > old_turn && entry.outcome.is_some())
                                .and_then(|entry| entry.outcome.clone())
                        };
                        if !session.state.character.alive {
                            lifecycle.phase = LifecyclePhase::Death;
                            lifecycle.selected = 0;
                            lifecycle.mark_dirty();
                        }
                        gameplay.screen = GameplayScreen::Dashboard;
                        gameplay.selected = 0;
                        gameplay.message = message;
                        gameplay.dirty = true;
                    }
                    Err(_) => {
                        gameplay.message = Some(
                            "That location is no longer reachable by known routes.".to_string(),
                        );
                        gameplay.dirty = true;
                    }
                }
            }
        }
    }
    if navigation_state.current_screen == Some(ScreenId::Gameplay) {
        navigation_state.selected = if gameplay.screen == GameplayScreen::Pause {
            gameplay.pause_selected
        } else {
            gameplay.selected
        };
    }
}

fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    mut gameplay: ResMut<GameplayState>,
    navigation_state: Res<NavigationState>,
    location_art: Res<LocationArtAssets>,
    celestial_art: Res<CelestialArtAssets>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || lifecycle.session.is_none()
        || navigation_state.current_screen != Some(ScreenId::Gameplay)
        || !gameplay.dirty
    {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }

    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let view = crate::game::world_view::build_view(&session.state);
    match gameplay.screen {
        GameplayScreen::Dashboard => {
            let actions = dashboard_actions(&session.state);
            render_dashboard(
                &mut commands,
                &view,
                &location_art,
                &celestial_art,
                &actions,
                gameplay.selected,
                gameplay.message.as_deref(),
            );
        }
        GameplayScreen::WorldMap => {
            let map_view = world_map::build_view(&session.state);
            render_world_map_screen(
                &mut commands,
                &map_view,
                gameplay.selected,
                gameplay.message.as_deref(),
            );
        }
        GameplayScreen::Pause => {
            let actions = dashboard_actions(&session.state);
            let root = render_dashboard(
                &mut commands,
                &view,
                &location_art,
                &celestial_art,
                &actions,
                gameplay.selected,
                gameplay.message.as_deref(),
            );
            let pause_actions = pause_actions(false, false);
            render_pause(&mut commands, root, &pause_actions, gameplay.pause_selected);
        }
    }
    gameplay.dirty = false;
}

fn render_dashboard(
    commands: &mut Commands,
    view: &WorldView,
    location_art: &LocationArtAssets,
    celestial_art: &CelestialArtAssets,
    actions: &[GameplayAction],
    selected: usize,
    message: Option<&str>,
) -> Entity {
    let sky = sky_visual(view.time_points);
    let root = bevy_presentation::spawn_world_root(commands, bevy_presentation::THEME_BACKGROUND);

    let world = commands
        .spawn((
            BackgroundColor(sky.background),
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(world);

    spawn_world_clouds(commands, world);
    spawn_world_ground(commands, world);
    if sky.warm_horizon {
        commands.entity(world).with_children(|world| {
            world.spawn((
                Node {
                    width: percent(100),
                    height: percent(9),
                    position_type: PositionType::Absolute,
                    left: px(0),
                    bottom: percent(27),
                    ..default()
                },
                BackgroundColor(bevy_presentation::THEME_SKY_HORIZON_WARM),
            ));
        });
    }
    spawn_celestial(commands, world, sky, celestial_art);
    render_location(commands, world, view, location_art);
    render_world_context(commands, world, view, message);
    render_player_hud(commands, world, view);
    render_action_area(commands, root, actions, selected);

    root
}

fn spawn_world_clouds(commands: &mut Commands, parent: Entity) {
    let clouds = [
        (44.0, 3.5, 0.70, 14.0, 110.0, 0.08),
        (50.0, 2.8, 0.45, 10.0, 82.0, 0.06),
        (12.0, 2.0, 0.36, 8.0, 64.0, 0.04),
    ];
    for (base_x, drift, alpha, height, width, speed) in clouds {
        commands.entity(parent).with_children(|world| {
            world.spawn((
                WorldCloud {
                    base_x,
                    drift,
                    speed,
                },
                Node {
                    width: px(width),
                    height: px(height),
                    position_type: PositionType::Absolute,
                    left: percent(base_x),
                    top: percent(15.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.9, 0.87, 0.82, alpha)),
            ));
        });
    }
}

fn animate_world_clouds(
    time: Res<Time>,
    runtime_state: Res<GameplayRuntimeState>,
    mut clouds: Query<(&WorldCloud, &mut Node)>,
) {
    if runtime_state.is_paused() {
        return;
    }
    let elapsed = time.elapsed_secs();
    for (cloud, mut node) in &mut clouds {
        let x = cloud.base_x + (elapsed * cloud.speed).sin() * cloud.drift;
        node.left = percent(x.clamp(4.0, 88.0));
    }
}

fn spawn_world_ground(commands: &mut Commands, parent: Entity) {
    commands.entity(parent).with_children(|world| {
        world.spawn((
            Node {
                width: percent(100),
                height: percent(27),
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_PANEL),
        ));
    });
}

fn spawn_celestial(
    commands: &mut Commands,
    parent: Entity,
    sky: SkyVisual,
    assets: &CelestialArtAssets,
) {
    let Some(image) = assets.image.as_ref() else {
        return;
    };
    let rect = match sky.celestial {
        CelestialBody::Sun => assets.sun_rect,
        CelestialBody::Moon => assets.moon_rect,
    };
    let Some(rect) = rect else {
        return;
    };

    let size = bevy_presentation::responsive_vmin(26.0);
    let celestial = commands
        .spawn((
            Node {
                width: size,
                height: size,
                min_width: size,
                min_height: size,
                position_type: PositionType::Absolute,
                left: percent(sky.celestial_x),
                top: percent(sky.celestial_y),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(image.clone()).with_rect(rect),
        ))
        .id();
    commands.entity(parent).add_child(celestial);
}

fn render_location(
    commands: &mut Commands,
    parent: Entity,
    view: &WorldView,
    location_art: &LocationArtAssets,
) {
    let card = commands
        .spawn((
            bevy_presentation::UiSurface,
            Node {
                width: percent(62),
                min_width: px(190),
                height: percent(48),
                min_height: px(130),
                position_type: PositionType::Absolute,
                left: percent(5),
                bottom: percent(9),
                padding: UiRect::all(bevy_presentation::responsive_vmin(1.5)),
                flex_direction: FlexDirection::Column,
                row_gap: bevy_presentation::responsive_compact_gap(),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    commands.entity(parent).add_child(card);

    let location_name = view
        .location
        .as_ref()
        .map(|location| location.name.as_str())
        .unwrap_or("Unknown Place");
    bevy_presentation::spawn_label(commands, card, location_name);
    if let Some(location) = &view.location {
        bevy_presentation::spawn_muted_label(
            commands,
            card,
            format!("{} | {}", view.world_name, location.region_name),
        );
        let banner = location
            .scene_image
            .as_deref()
            .and_then(|reference| location_art.images.get(reference));
        if let Some(image) = banner {
            let art_entity = commands
                .spawn((
                    ImageNode::new(image.clone()),
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        aspect_ratio: Some(4.8),
                        flex_shrink: 0.0,
                        ..default()
                    },
                ))
                .id();
            commands.entity(card).add_child(art_entity);
        } else if let Some(art) = &view.art {
            let art_entity = commands
                .spawn((
                    Text::new(art.clone()),
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        flex_grow: 1.0,
                        min_height: px(40),
                        ..default()
                    },
                    bevy_presentation::TextContent,
                    TextFont::from_font_size(bevy_presentation::muted_font_size()),
                    TextColor(bevy_presentation::THEME_MUTED),
                ))
                .id();
            commands.entity(card).add_child(art_entity);
        }
        if !location.description.trim().is_empty() {
            bevy_presentation::spawn_muted_label(commands, card, location.description.clone());
        }
    } else {
        bevy_presentation::spawn_muted_label(commands, card, "You are lost in an unknown place.");
    }

    commands.entity(card).with_children(|card| {
        card.spawn((
            Node {
                width: px(170),
                height: px(6),
                position_type: PositionType::Absolute,
                left: percent(4),
                bottom: px(8),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.28)),
        ));
    });
}

fn render_world_context(
    commands: &mut Commands,
    parent: Entity,
    view: &WorldView,
    message: Option<&str>,
) {
    let text = message
        .map(str::to_string)
        .or_else(|| {
            view.threat
                .as_ref()
                .map(|threat| format!("{}: {}", threat.label, threat.description))
        })
        .or_else(|| view.atmosphere.clone());

    let Some(text) = text else {
        return;
    };

    let context = commands
        .spawn((
            bevy_presentation::UiContextMessage,
            Node {
                width: percent(58),
                min_width: px(180),
                max_width: percent(70),
                min_height: px(38),
                position_type: PositionType::Absolute,
                left: percent(5),
                top: percent(28),
                padding: UiRect::axes(
                    bevy_presentation::responsive_vmin(1.3),
                    bevy_presentation::responsive_vmin(0.9),
                ),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    commands.entity(parent).add_child(context);
    bevy_presentation::spawn_muted_label(commands, context, text);
}

fn render_player_hud(commands: &mut Commands, parent: Entity, view: &WorldView) {
    let controls = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            position_type: PositionType::Absolute,
            right: px(0),
            top: px(0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Start,
            padding: UiRect::all(bevy_presentation::responsive_vmin(1.0)),
            column_gap: bevy_presentation::responsive_compact_gap(),
            ..default()
        })
        .id();
    commands.entity(parent).add_child(controls);

    bevy_presentation::spawn_menu_button(commands, controls, "menu");

    let hud = commands
        .spawn((
            bevy_presentation::UiSurface,
            Node {
                width: percent(30),
                min_width: px(180),
                max_width: px(230),
                min_height: px(104),
                padding: UiRect::all(bevy_presentation::responsive_vmin(1.1)),
                flex_direction: FlexDirection::Column,
                row_gap: bevy_presentation::responsive_vmin(0.7),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    commands.entity(controls).add_child(hud);

    bevy_presentation::spawn_health_gauge(commands, hud, view.character.hp, view.character.max_hp);

    let condition_row = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            height: px(28),
            flex_direction: FlexDirection::Row,
            column_gap: px(6),
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    commands.entity(hud).add_child(condition_row);

    for condition in &view.conditions {
        spawn_condition_icon(commands, condition_row, condition);
    }
}

fn spawn_condition_icon(commands: &mut Commands, parent: Entity, condition: &ConditionView) {
    let icon = match condition.name.as_str() {
        "Wounded" => "condition-wounded",
        "Exhausted" => "condition-exhausted",
        _ => "condition-rested",
    };
    bevy_presentation::spawn_condition_indicator(commands, parent, icon);
}

fn render_action_area(
    commands: &mut Commands,
    parent: Entity,
    actions: &[GameplayAction],
    selected: usize,
) {
    let panel =
        bevy_presentation::spawn_surface(commands, parent, bevy_presentation::SurfaceTone::Strong);
    let row = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: bevy_presentation::responsive_compact_gap(),
            row_gap: bevy_presentation::responsive_compact_gap(),
            ..default()
        })
        .id();
    commands.entity(panel).add_child(row);

    for (index, action) in actions.iter().enumerate() {
        let (icon, label) = gameplay_action_visuals(*action);
        let button =
            bevy_presentation::spawn_primary_action_button(commands, row, index, icon, label);
        if index == selected {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }
}

fn gameplay_action_visuals(action: GameplayAction) -> (&'static str, &'static str) {
    match action {
        GameplayAction::Map => ("travel", "Map"),
        GameplayAction::Meditate => ("meditate", "Meditate"),
        GameplayAction::Explore => ("explore", "Explore"),
        GameplayAction::Investigate => ("investigate", "Investigate"),
    }
}

fn dashboard_actions(state: &crate::model::GameState) -> Vec<GameplayAction> {
    let mut actions = PRIMARY_GAMEPLAY_ACTIONS.to_vec();
    if menu::build_main_menu(state)
        .iter()
        .any(|entry| matches!(entry.action, menu::GameAction::InvestigateThreat))
    {
        actions.push(GameplayAction::Investigate);
    }
    actions
}

fn reset_dashboard_selection(gameplay: &mut GameplayState) {
    gameplay.selected = 0;
    gameplay.message = None;
    gameplay.dirty = true;
}

pub(crate) fn sky_visual(time_points: u32) -> SkyVisual {
    match time_points % 12 {
        0 => SkyVisual {
            background: bevy_presentation::THEME_SKY_NIGHT,
            celestial: CelestialBody::Moon,
            celestial_x: 78.0,
            celestial_y: 9.0,
            warm_horizon: false,
        },
        1 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DAWN,
            celestial: CelestialBody::Moon,
            celestial_x: 67.0,
            celestial_y: 11.0,
            warm_horizon: true,
        },
        2 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DAWN,
            celestial: CelestialBody::Sun,
            celestial_x: 58.0,
            celestial_y: 11.0,
            warm_horizon: true,
        },
        3 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DAY,
            celestial: CelestialBody::Sun,
            celestial_x: 50.0,
            celestial_y: 7.0,
            warm_horizon: false,
        },
        4 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DAY,
            celestial: CelestialBody::Sun,
            celestial_x: 42.0,
            celestial_y: 9.0,
            warm_horizon: false,
        },
        5 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DAY,
            celestial: CelestialBody::Sun,
            celestial_x: 34.0,
            celestial_y: 11.0,
            warm_horizon: false,
        },
        6 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DAY,
            celestial: CelestialBody::Sun,
            celestial_x: 25.0,
            celestial_y: 14.0,
            warm_horizon: false,
        },
        7 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DUSK,
            celestial: CelestialBody::Sun,
            celestial_x: 17.0,
            celestial_y: 18.0,
            warm_horizon: true,
        },
        8 => SkyVisual {
            background: bevy_presentation::THEME_SKY_DUSK,
            celestial: CelestialBody::Moon,
            celestial_x: 10.0,
            celestial_y: 13.0,
            warm_horizon: true,
        },
        9 => SkyVisual {
            background: bevy_presentation::THEME_SKY_NIGHT,
            celestial: CelestialBody::Moon,
            celestial_x: 21.0,
            celestial_y: 10.0,
            warm_horizon: false,
        },
        10 => SkyVisual {
            background: bevy_presentation::THEME_SKY_NIGHT,
            celestial: CelestialBody::Moon,
            celestial_x: 37.0,
            celestial_y: 8.0,
            warm_horizon: false,
        },
        11 => SkyVisual {
            background: bevy_presentation::THEME_SKY_NIGHT,
            celestial: CelestialBody::Moon,
            celestial_x: 55.0,
            celestial_y: 8.0,
            warm_horizon: false,
        },
        _ => unreachable!(),
    }
}

pub(crate) fn render_spatial_pause_overlay(
    commands: &mut Commands,
    parent: Entity,
    selected: usize,
    meditation_available: bool,
) {
    let actions = pause_actions(true, meditation_available);
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
            screen: GameplayScreen::Pause,
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

fn render_world_map_screen(
    commands: &mut Commands,
    view: &WorldMapView,
    selected: usize,
    message: Option<&str>,
) {
    let root = bevy_presentation::spawn_screen(commands, "WORLD MAP");
    let panel = bevy_presentation::spawn_panel(commands, root);
    render_world_map_content(commands, panel, view, selected, message);
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
            bevy_presentation::TouchScrollablePanel,
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

    render_world_map_graph(commands, panel, view);

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
        screen: GameplayScreen::WorldMap,
        index: back_index,
    });
    if selected == back_index {
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
    }
}

fn render_world_map_graph(commands: &mut Commands, parent: Entity, view: &WorldMapView) {
    let layout = build_world_map_layout(view);
    let viewport_height = layout.height.clamp(300.0, 460.0);
    let viewport = commands
        .spawn((
            bevy_presentation::TouchScrollablePanel,
            Node {
                width: percent(100),
                min_width: px(0),
                height: px(viewport_height),
                max_height: px(460.0),
                padding: UiRect::all(px(6)),
                overflow: Overflow::scroll(),
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
            Node {
                width: px(layout.width),
                height: px(layout.height),
                min_width: px(layout.width),
                position_type: PositionType::Relative,
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
                    screen: GameplayScreen::WorldMap,
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
