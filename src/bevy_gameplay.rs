//! Bevy presentation and interaction for the primary gameplay/world screen.
//!
//! This module adapts the existing gameplay state and world-navigation rules
//! into Bevy UI. It does not reimplement gameplay simulation.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::bevy_combat::CombatState;
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::game::{actions, menu, navigation};
use crate::input::InputEvent;
use crate::presentation::{ConditionView, NavigationView, WorldView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameplayScreen {
    Dashboard,
    Navigation,
    Pause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PauseAction {
    Resume,
    Travel,
    Character,
    NewGame,
    LoadGame,
    Options,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameplayAction {
    Travel,
    Meditate,
    Explore,
    Investigate,
}

const PRIMARY_GAMEPLAY_ACTIONS: [GameplayAction; 3] = [
    GameplayAction::Travel,
    GameplayAction::Meditate,
    GameplayAction::Explore,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CelestialBody {
    Sun,
    Moon,
}

#[derive(Debug, Clone, Copy)]
struct SkyVisual {
    background: Color,
    celestial: CelestialBody,
    celestial_x: f32,
    celestial_y: f32,
    warm_horizon: bool,
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
struct CelestialArtAssets {
    image: Option<Handle<Image>>,
    sun_rect: Option<Rect>,
    moon_rect: Option<Rect>,
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
                GameplayScreen::Pause | GameplayScreen::Navigation
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
                GameplayScreen::Navigation => {
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
        GameplayScreen::Navigation => navigation::build_view(&session.state).destinations.len() + 1,
        GameplayScreen::Pause => pause_action_count(
            navigation_state.current_screen == Some(ScreenId::SpatialExploration),
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
        GameplayScreen::Navigation => navigation::build_view(&session.state).destinations.len() + 1,
        GameplayScreen::Pause => pause_action_count(
            navigation_state.current_screen == Some(ScreenId::SpatialExploration),
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
        GameplayScreen::Navigation => gameplay.selected,
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

fn pause_action_count(spatial_menu: bool) -> usize {
    pause_actions_from(crate::bevy_lifecycle::has_available_saves(), spatial_menu).len()
}

fn pause_actions(spatial_menu: bool) -> Vec<PauseAction> {
    pause_actions_from(crate::bevy_lifecycle::has_available_saves(), spatial_menu)
}

fn pause_actions_from(has_load: bool, spatial_menu: bool) -> Vec<PauseAction> {
    let mut actions = vec![PauseAction::Resume];
    if spatial_menu {
        actions.push(PauseAction::Travel);
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
            let actions = pause_actions(
                navigation_state.current_screen == Some(ScreenId::SpatialExploration),
            );
            let Some(action) = actions.get(gameplay.pause_selected).copied() else {
                gameplay.pause_selected = 0;
                navigation_state.selected = 0;
                return;
            };

            match action {
                PauseAction::Travel => {
                    gameplay.screen = GameplayScreen::Navigation;
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
                GameplayAction::Travel => {
                    gameplay.screen = GameplayScreen::Navigation;
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
        GameplayScreen::Navigation => {
            if navigation_state.current_screen == Some(ScreenId::SpatialExploration) {
                let Some(target_id) = lifecycle.session.as_ref().and_then(|session| {
                    navigation::build_view(&session.state)
                        .destinations
                        .get(gameplay.selected)
                        .map(|destination| destination.id)
                }) else {
                    gameplay.screen = GameplayScreen::Pause;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                    navigation_state.selected = gameplay.pause_selected;
                    spatial_state.request_presentation_refresh();
                    return;
                };

                match crate::bevy_spatial::travel_to_destination(
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
                let view = navigation::build_view(&session.state);
                if gameplay.selected >= view.destinations.len() {
                    gameplay.screen = GameplayScreen::Dashboard;
                    gameplay.selected = 0;
                    gameplay.message = None;
                    gameplay.dirty = true;
                    navigation_state.selected = 0;
                    return;
                }
                if let Some(destination) = view.destinations.get(gameplay.selected) {
                    let old_turn = session.state.character.turn;
                    let target_id = destination.id;
                    match actions::travel_to(&mut session.state, target_id) {
                        Ok(()) => {
                            let message = session
                                .state
                                .world
                                .history
                                .iter()
                                .rev()
                                .find(|entry| entry.turn > old_turn)
                                .map(|entry| {
                                    entry.outcome.clone().unwrap_or_else(|| entry.text.clone())
                                });
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
                            gameplay.message =
                                Some("That route is no longer available.".to_string());
                            gameplay.dirty = true;
                        }
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
        GameplayScreen::Navigation => {
            render_navigation(
                &mut commands,
                &navigation::build_view(&session.state),
                gameplay.selected,
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
            let pause_actions = pause_actions(false);
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
        GameplayAction::Travel => ("travel", "Travel"),
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

fn sky_visual(time_points: u32) -> SkyVisual {
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
) {
    let actions = pause_actions(true);
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
        PauseAction::Travel => ("travel", "Travel"),
        PauseAction::Character => ("character", "Character"),
        PauseAction::NewGame => ("new-game", "New Game"),
        PauseAction::LoadGame => ("load-game", "Load Game"),
        PauseAction::Options => ("options", "Options"),
        PauseAction::Quit => ("quit", "Quit"),
    }
}

fn render_navigation(commands: &mut Commands, view: &NavigationView, selected: usize) {
    let root = bevy_presentation::spawn_screen(commands, "WORLD NAVIGATION");
    let panel = bevy_presentation::spawn_panel(commands, root);
    render_navigation_content(commands, panel, view, selected, None);
}

pub(crate) fn render_spatial_navigation_overlay(
    commands: &mut Commands,
    parent: Entity,
    view: &NavigationView,
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
    bevy_presentation::spawn_label(commands, surface, "WORLD NAVIGATION");
    render_navigation_content(commands, surface, view, selected, message);
}

fn render_navigation_content(
    commands: &mut Commands,
    panel: Entity,
    view: &NavigationView,
    selected: usize,
    message: Option<&str>,
) {
    if let Some(current) = &view.current_location {
        bevy_presentation::spawn_label(commands, panel, current.name.clone());
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            format!("Region: {}", current.region_name),
        );
        if current.dangerous {
            bevy_presentation::spawn_muted_label(
                commands,
                panel,
                "Danger: this location is unsafe.",
            );
        }
        if !current.description.trim().is_empty() {
            bevy_presentation::spawn_muted_label(commands, panel, current.description.clone());
        }
    } else {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "Your current location can no longer be resolved.",
        );
    }
    if let Some(message) = message {
        bevy_presentation::spawn_muted_label(commands, panel, message);
    }

    if view.destinations.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "No known routes lead onward.");
    } else {
        for (index, destination) in view.destinations.iter().enumerate() {
            let button = bevy_presentation::spawn_action_button(
                commands,
                panel,
                index,
                "travel",
                destination.name.clone(),
            );
            commands.entity(button).insert(GameplayMenuChoice {
                screen: GameplayScreen::Navigation,
                index,
            });
            if index == selected {
                commands
                    .entity(button)
                    .insert(bevy_presentation::UiSelected);
            }
        }
    }
    let back_index = view.destinations.len();
    let button =
        bevy_presentation::spawn_action_button(commands, panel, back_index, "back", "Back");
    commands.entity(button).insert(GameplayMenuChoice {
        screen: GameplayScreen::Navigation,
        index: back_index,
    });
    if selected == back_index {
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
    }
}

#[cfg(test)]
#[path = "../tests/unit/bevy_gameplay.rs"]
mod tests;
