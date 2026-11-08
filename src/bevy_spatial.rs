//! World-first local spatial exploration and contextual interaction presentation.
//!
//! The renderer-neutral map model and character session remain separate from
//! Bevy presentation. Persistent maps and actor placements are synchronized with
//! world state, while the character's continuous position belongs to the session.

use std::collections::HashMap;

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{Justify, LineBreak, TextLayout};
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;
#[cfg(target_os = "android")]
use bevy::window::PrimaryWindow;

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId, UiButtonActivation,
};
use crate::input::InputEvent;
use crate::interaction_actions::{AvailableInteraction, InteractionActionKind, InteractionTarget};
use crate::model::{Condition, EntityId, ExplorationPosition, GameState};
use crate::spatial::{
    configure_map_exits, ensure_spatial_entity_placements, generate_local_map_with_layout,
    ExplorationSession, GridPosition, LocalMap, LocalMapConfig, MapExit, SpatialEntityKind,
    SpatialPropKind, Terrain,
};

const TILE_HALF_WIDTH: f32 = 24.0;
const TILE_HALF_HEIGHT: f32 = 12.0;
const WALL_HEIGHT: f32 = 24.0;
const PLAYER_SPRITE_WIDTH: f32 = 16.0;
const PLAYER_SPRITE_HEIGHT: f32 = 24.0;
const PLAYER_COLLISION_HALF_EXTENT: f32 = 0.15;
const INTERACTION_RANGE_PX: f32 = 34.0;
const PLAYER_SPEED: f32 = 180.0; // Screen-space pixels per second.
#[cfg(any(target_os = "android", test))]
const JOYSTICK_RADIUS_PX: f32 = 68.0;
#[cfg(any(target_os = "android", test))]
const JOYSTICK_DEAD_ZONE: f32 = 0.16;
#[cfg(target_os = "android")]
const JOYSTICK_MARGIN_PX: f32 = 20.0;
#[cfg(target_os = "android")]
const JOYSTICK_THUMB_RADIUS_PX: f32 = 20.0;
#[cfg(target_os = "android")]
const JOYSTICK_THUMB_TRAVEL_PX: f32 = 36.0;
const CAMERA_LERP_SPEED: f32 = 8.0;
const CAMERA_DEAD_ZONE_FRACTION: f32 = 0.15;
const DEPTH_STEP: f32 = 0.08;
const GROUND_TERRAIN_DEPTH: f32 = -1.0;
const WALL_TOP_DEPTH_BIAS: f32 = 0.002;
const WALL_FACE_DEPTH_BIAS: f32 = -0.002;
const PLAYER_DEPTH_OFFSET: f32 = 0.04;
const INTERACTION_WHEEL_ANIMATION_SECONDS: f32 = 0.16;
const INTERACTION_WHEEL_SIZE_PX: f32 = 300.0;
const INTERACTION_WHEEL_BUTTON_HEIGHT_PX: f32 = 48.0;

#[derive(Component)]
struct SpatialWorldEntity;

#[derive(Component)]
pub(crate) struct SpatialSceneRoot;

#[derive(Component)]
struct SpatialActorEntity;

#[derive(Component)]
struct SpatialPlayer;

#[derive(Component, Clone, Copy)]
struct SpatialTile(GridPosition);

#[derive(Component, Clone, Copy)]
struct SpatialWallSide {
    tile: GridPosition,
}

#[derive(Component)]
struct SpatialControlButton;

#[derive(Component)]
struct SpatialInteractionWheelRoot;

#[derive(Component, Clone, Copy)]
struct SpatialWheelEntry(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionWheelPhase {
    Opening,
    Open,
    Closing,
}

struct SpatialInteractionWheel {
    entries: Vec<AvailableInteraction>,
    selected: usize,
    phase: InteractionWheelPhase,
    elapsed: f32,
    pending: Option<AvailableInteraction>,
}

#[cfg(target_os = "android")]
#[derive(Component)]
struct SpatialJoystickThumb;

#[derive(Resource, Default)]
struct SpatialJoystickState {
    #[cfg(target_os = "android")]
    active_touch: Option<u64>,
    direction: Vec2,
}

#[derive(Component)]
struct SpatialInteractionPrompt;

#[derive(Component)]
struct SpatialPromptLabel;

/// Request to start an interaction with a world-owned actor. The scene remains
/// active; the interaction adapter presents the conversation/search overlay.
#[derive(Message, Debug, Clone, Copy)]
pub(crate) enum SpatialInteractionRequest {
    Talk(EntityId),
    SearchRemains(EntityId),
}

#[derive(Resource, Default)]
struct SpatialRenderAssets {
    tile_images: [Option<Handle<Image>>; 4],
    wall_face_images: [Option<Handle<Image>>; 2],
    player_image: Option<Handle<Image>>,
    passage_image: Option<Handle<Image>>,
    poi_image: Option<Handle<Image>>,
    npc_image: Option<Handle<Image>>,
    corpse_image: Option<Handle<Image>>,
    prop_images: [Option<Handle<Image>>; 2],
}

#[derive(Component, Clone, Copy)]
enum SpatialAction {
    Interact,
    Exit,
    SelectInteraction(usize),
    CloseWheel,
}

#[derive(Resource, Default)]
pub(crate) struct SpatialExplorationState {
    maps: HashMap<String, LocalMap>,
    session: Option<ExplorationSession>,
    position: Vec2,
    message: Option<String>,
    interaction_wheel: Option<SpatialInteractionWheel>,
    dirty: bool,
}

pub(crate) fn install(app: &mut App) {
    app.add_message::<SpatialInteractionRequest>()
        .init_resource::<SpatialExplorationState>()
        .init_resource::<SpatialJoystickState>()
        .add_systems(Startup, setup_spatial_render_assets)
        .add_systems(
            Update,
            (
                spatial_input,
                update_spatial_wheel_animation,
                update_spatial_world,
                update_spatial_context_prompt,
                update_spatial_control_feedback,
                render_spatial_screen,
                sync_spatial_interaction_wheel,
                update_spatial_wheel_anchor,
            )
                .chain(),
        );
    #[cfg(target_os = "android")]
    app.add_systems(Update, update_virtual_joystick.before(spatial_input));
}

pub(crate) fn enter(
    state: &mut SpatialExplorationState,
    game: &GameState,
    navigation: &mut NavigationState,
) {
    let Some(location) = game.world.location_by_id(game.character.location_id) else {
        return;
    };
    let location_id = format!("location.{}", location.id);
    let world_seed = game
        .world
        .generation
        .map(|generation| generation.seed)
        .unwrap_or(game.world.id);

    let mut map = game
        .world
        .local_maps
        .iter()
        .find(|map| map.location_id == location_id)
        .cloned()
        .unwrap_or_else(|| {
            generate_local_map_with_layout(
                world_seed,
                location_id.clone(),
                LocalMapConfig::default(),
                location.spatial_layout.as_deref(),
            )
        });
    // Migrate older saved maps without exits and configure new maps from the
    // actual world graph. Passage targets are stored as stable world IDs.
    if map.exits.is_empty() {
        configure_map_exits(&mut map, &location.exits);
    }
    if !location.dangerous {
        let prototype_poi_id = format!("{location_id}.poi.initial_discovery");
        map.points_of_interest
            .retain(|poi| poi.id != prototype_poi_id);
    }
    sync_map_interactable_placements(&mut map, game, location.id);
    state.maps.insert(location_id.clone(), map);
    let map = state.maps.get(&location_id).expect("map was just inserted");
    let saved_position = game
        .character
        .exploration_session
        .as_ref()
        .filter(|session| session.location_id == location_id)
        .and(game.character.exploration_position)
        .map(|position| Vec2::new(position.x, position.y))
        .or_else(|| {
            game.character
                .exploration_session
                .as_ref()
                .filter(|session| session.location_id == location_id)
                .map(|session| {
                    Vec2::new(
                        f32::from(session.position.x) + 0.5,
                        f32::from(session.position.y) + 0.5,
                    )
                })
        });
    let position = saved_position
        .filter(|position| {
            position.x >= 0.0
                && position.y >= 0.0
                && map.is_walkable(GridPosition {
                    x: position.x.floor() as u16,
                    y: position.y.floor() as u16,
                })
        })
        .unwrap_or(Vec2::new(
            f32::from(map.entry.x) + 0.5,
            f32::from(map.entry.y) + 0.5,
        ));
    state.position = position;
    state.interaction_wheel = None;
    state.session = Some(ExplorationSession {
        location_id,
        position: GridPosition {
            x: position.x.floor().max(0.0) as u16,
            y: position.y.floor().max(0.0) as u16,
        },
    });
    map_mark_discovered(state, position);
    state.message =
        Some("Explore the area. Move close to people or objects to interact.".to_string());
    state.dirty = true;
    navigation.return_screen = Some(ScreenId::Gameplay);
    navigation.current_screen = Some(ScreenId::SpatialExploration);
    navigation.selected = 0;
}

fn sync_map_interactable_placements(map: &mut LocalMap, game: &GameState, location_id: EntityId) {
    let mut entities = game
        .npcs
        .iter()
        .filter(|npc| npc.location_id == location_id)
        .map(|npc| (npc.id, SpatialEntityKind::Npc))
        .chain(
            game.corpses
                .iter()
                .filter(|corpse| corpse.location_id == location_id && !corpse.inventory.is_empty())
                .map(|corpse| (corpse.id, SpatialEntityKind::Corpse)),
        )
        .collect::<Vec<_>>();
    entities.sort_by_key(|(entity_id, kind)| {
        (*entity_id, matches!(*kind, SpatialEntityKind::Corpse) as u8)
    });
    ensure_spatial_entity_placements(map, &entities);
}

fn setup_spatial_render_assets(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let tile_images = [
        load_spatial_image(&mut images, "spatial/terrain_floor.png"),
        load_spatial_image(&mut images, "spatial/terrain_ash.png"),
        load_spatial_image(&mut images, "spatial/terrain_rubble.png"),
        load_spatial_image(&mut images, "spatial/terrain_wall_top.png"),
    ];
    let wall_face_images = [
        load_spatial_image(&mut images, "spatial/wall_face_left.png"),
        load_spatial_image(&mut images, "spatial/wall_face_right.png"),
    ];
    let player_image = load_spatial_image(&mut images, "spatial/player_ash_warden.png");
    let passage_image = load_spatial_image(&mut images, "spatial/passage.png");
    let poi_image = load_spatial_image(&mut images, "icons/ui/investigate.png");
    let npc_image = load_spatial_image(&mut images, "icons/ui/character.png");
    let corpse_image = load_spatial_image(&mut images, "icons/ui/remains.png");
    let prop_images = [
        load_spatial_image(&mut images, "spatial/prop_pillar.png"),
        load_spatial_image(&mut images, "spatial/prop_table.png"),
    ];

    commands.insert_resource(SpatialRenderAssets {
        tile_images,
        wall_face_images,
        player_image,
        passage_image,
        poi_image,
        npc_image,
        corpse_image,
        prop_images,
    });
}

fn load_spatial_image(images: &mut Assets<Image>, reference: &str) -> Option<Handle<Image>> {
    match crate::asset_loading::load_image_from_data(images, reference) {
        Ok(image) => Some(image),
        Err(error) => {
            warn!("Could not load spatial asset {reference}: {error}");
            None
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spatial_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut queue: ResMut<GameplayInputQueue>,
    mut state: ResMut<SpatialExplorationState>,
    joystick: Res<SpatialJoystickState>,
    interaction_state: Res<crate::bevy_interactions::BevyInteractionState>,
    mut requests: MessageWriter<SpatialInteractionRequest>,
    mut activations: MessageReader<UiButtonActivation>,
    action_buttons: Query<&SpatialAction>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || navigation.current_screen != Some(ScreenId::SpatialExploration)
        || crate::bevy_interactions::spatial_overlay_open(&interaction_state)
    {
        return;
    }

    let activated_buttons = activations
        .read()
        .map(|activation| activation.0)
        .collect::<Vec<_>>();

    for button in activated_buttons {
        let Ok(action) = action_buttons.get(button) else {
            continue;
        };
        match *action {
            SpatialAction::Interact => open_interaction_wheel(&mut state, &lifecycle),
            SpatialAction::Exit => exit_spatial(&mut state, &mut navigation),
            SpatialAction::SelectInteraction(index) => {
                close_interaction_wheel(&mut state, Some(index));
            }
            SpatialAction::CloseWheel => close_interaction_wheel(&mut state, None),
        }
    }

    if state.interaction_wheel.is_some() {
        if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
            move_interaction_wheel_selection(&mut state, Vec2::Y);
        }
        if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
            move_interaction_wheel_selection(&mut state, Vec2::NEG_Y);
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
            move_interaction_wheel_selection(&mut state, Vec2::NEG_X);
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
            move_interaction_wheel_selection(&mut state, Vec2::X);
        }
        if keyboard.just_pressed(KeyCode::KeyE) {
            confirm_interaction_wheel(&mut state);
        }
    } else {
        let mut direction = Vec2::ZERO;
        if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        direction += joystick.direction;

        if direction.length_squared() > 0.0 {
            let input_strength = direction.length().min(1.0);
            let Some(location_id) = state
                .session
                .as_ref()
                .map(|session| session.location_id.clone())
            else {
                return;
            };
            let Some(map) = state.maps.get(&location_id) else {
                return;
            };
            let world_direction = screen_direction_to_world(direction);
            let next = advance_position(
                map,
                state.position,
                world_direction,
                time.delta_secs(),
                PLAYER_SPEED * input_strength,
            );
            state.position = next;
            if let Some(session) = state.session.as_mut() {
                session.position = GridPosition {
                    x: next.x.floor().max(0.0) as u16,
                    y: next.y.floor().max(0.0) as u16,
                };
            }
            map_mark_discovered(&mut state, next);
        }

        if keyboard.just_pressed(KeyCode::KeyE) {
            open_interaction_wheel(&mut state, &lifecycle);
        }
    }

    let events = std::mem::take(&mut queue.0);
    for event in events {
        match event {
            InputEvent::Confirm if state.interaction_wheel.is_some() => {
                confirm_interaction_wheel(&mut state);
            }
            InputEvent::Cancel | InputEvent::Backspace
                if state.interaction_wheel.is_some() =>
            {
                close_interaction_wheel(&mut state, None);
            }
            InputEvent::Confirm => open_interaction_wheel(&mut state, &lifecycle),
            InputEvent::Cancel | InputEvent::Backspace => {
                exit_spatial(&mut state, &mut navigation);
            }
            _ => {}
        }
    }

    sync_spatial_to_game(&state, &mut lifecycle);
}

fn sync_spatial_to_game(state: &SpatialExplorationState, lifecycle: &mut LifecycleState) {
    let (Some(session), Some(game_session)) = (state.session.as_ref(), lifecycle.session.as_mut())
    else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };
    if let Some(existing) = game_session
        .state
        .world
        .local_maps
        .iter_mut()
        .find(|existing| existing.location_id == map.location_id)
    {
        if existing != map {
            *existing = map.clone();
        }
    } else {
        game_session.state.world.local_maps.push(map.clone());
    }
    if game_session.state.character.exploration_session.as_ref() != Some(session) {
        game_session.state.character.exploration_session = Some(session.clone());
    }
    let position = ExplorationPosition {
        x: state.position.x,
        y: state.position.y,
    };
    if game_session.state.character.exploration_position != Some(position) {
        game_session.state.character.exploration_position = Some(position);
    }
}

fn session_position(state: &SpatialExplorationState) -> Vec2 {
    state.position
}

fn map_mark_discovered(state: &mut SpatialExplorationState, position: Vec2) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    if let Some(map) = state.maps.get_mut(&session.location_id) {
        let center = GridPosition {
            x: position.x.floor().max(0.0) as u16,
            y: position.y.floor().max(0.0) as u16,
        };
        for y in center.y.saturating_sub(2)..=center.y.saturating_add(2).min(map.height - 1) {
            for x in center.x.saturating_sub(2)..=center.x.saturating_add(2).min(map.width - 1) {
                map.mark_tile_discovered(GridPosition { x, y });
            }
        }
    }
}

fn interact_with_poi(
    state: &mut SpatialExplorationState,
    lifecycle: &mut LifecycleState,
    poi_id: &str,
) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let location_id = session.location_id.clone();
    let position = session_position(state);
    let Some(map) = state.maps.get(&location_id) else {
        return;
    };
    let Some(poi) = map.points_of_interest.iter().find(|poi| poi.id == poi_id) else {
        state.message = Some("There is nothing here that answers your attention.".to_string());
        state.dirty = true;
        return;
    };
    if !poi_interaction_available_for(map, position, poi) {
        state.message = Some("Move closer and find a clear approach.".to_string());
        state.dirty = true;
        return;
    }
    let Some(map) = state.maps.get_mut(&location_id) else {
        return;
    };
    let Some(poi) = map.point_of_interest_mut(poi_id) else {
        return;
    };
    if poi.interacted {
        state.message =
            Some("The remains offer nothing more. The wrongness is still there.".to_string());
        state.dirty = true;
        return;
    }
    poi.discovered = true;
    poi.interacted = true;
    let title = poi.title.clone();
    let description = poi.description.clone();
    if let Some(session) = lifecycle.session.as_mut() {
        let turn = session.state.character.turn;
        session.state.world.record_history(
            turn,
            format!(
                "{}: {} {}",
                title, description, "The discovery has been recorded."
            ),
        );
    }
    state.message = Some(format!(
        "{}. {} You have the distinct impression that something noticed.",
        title, description
    ));
    state.dirty = true;
}

fn open_interaction_wheel(
    state: &mut SpatialExplorationState,
    lifecycle: &LifecycleState,
) {
    if state.interaction_wheel.is_some() {
        return;
    }
    let entries = state
        .session
        .as_ref()
        .and_then(|session| state.maps.get(&session.location_id))
        .and_then(|map| {
            lifecycle.session.as_ref().map(|game_session| {
                available_interactions(map, &game_session.state, state.position)
            })
        })
        .unwrap_or_default();

    if entries.is_empty() {
        state.message =
            Some("There is nothing within reach that can be approached.".to_string());
        state.dirty = true;
        return;
    }

    state.message = None;
    state.interaction_wheel = Some(SpatialInteractionWheel {
        entries,
        selected: 0,
        phase: InteractionWheelPhase::Opening,
        elapsed: 0.0,
        pending: None,
    });
}

fn close_interaction_wheel(
    state: &mut SpatialExplorationState,
    selected: Option<usize>,
) {
    let Some(wheel) = state.interaction_wheel.as_mut() else {
        return;
    };
    if wheel.phase == InteractionWheelPhase::Closing {
        return;
    }
    if let Some(index) = selected {
        let Some(interaction) = wheel.entries.get(index).cloned() else {
            return;
        };
        wheel.selected = index;
        wheel.pending = Some(interaction);
    } else {
        wheel.pending = None;
    }
    wheel.phase = InteractionWheelPhase::Closing;
    wheel.elapsed = 0.0;
}

fn confirm_interaction_wheel(state: &mut SpatialExplorationState) {
    let selected = state
        .interaction_wheel
        .as_ref()
        .filter(|wheel| wheel.phase != InteractionWheelPhase::Closing)
        .map(|wheel| wheel.selected);
    if let Some(selected) = selected {
        close_interaction_wheel(state, Some(selected));
    }
}

fn move_interaction_wheel_selection(
    state: &mut SpatialExplorationState,
    direction: Vec2,
) {
    let Some(wheel) = state.interaction_wheel.as_mut() else {
        return;
    };
    if wheel.phase == InteractionWheelPhase::Closing {
        return;
    }
    if let Some(selected) =
        interaction_wheel_index_for_direction(wheel.entries.len(), direction)
    {
        wheel.selected = selected;
    }
}

fn execute_available_interaction(
    state: &mut SpatialExplorationState,
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    requests: &mut MessageWriter<SpatialInteractionRequest>,
    interaction: AvailableInteraction,
) {
    let still_available = state
        .session
        .as_ref()
        .and_then(|session| state.maps.get(&session.location_id))
        .zip(lifecycle.session.as_ref())
        .is_some_and(|(map, game_session)| {
            available_interactions(map, &game_session.state, state.position)
                .iter()
                .any(|candidate| candidate.stable_key() == interaction.stable_key())
        });
    if !still_available {
        state.message = Some("That action is no longer available.".to_string());
        state.dirty = true;
        return;
    }

    match (interaction.action, interaction.target) {
        (InteractionActionKind::Talk, InteractionTarget::Npc(id)) => {
            requests.write(SpatialInteractionRequest::Talk(id));
        }
        (InteractionActionKind::SearchRemains, InteractionTarget::Corpse(id)) => {
            requests.write(SpatialInteractionRequest::SearchRemains(id));
        }
        (InteractionActionKind::Examine, InteractionTarget::PointOfInterest(id)) => {
            interact_with_poi(state, lifecycle, &id);
        }
        (
            InteractionActionKind::Enter,
            InteractionTarget::Passage {
                id,
                destination_id,
            },
        ) => {
            use_map_exit(state, navigation, &id, destination_id);
        }
        _ => {
            state.message = Some("That action is not available for this target.".to_string());
            state.dirty = true;
        }
    }
}

fn update_spatial_wheel_animation(
    time: Res<Time>,
    mut state: ResMut<SpatialExplorationState>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut requests: MessageWriter<SpatialInteractionRequest>,
    mut wheel_roots: Query<&mut Transform, With<SpatialInteractionWheelRoot>>,
) {
    let mut finished_close = false;
    let mut pending = None;
    if let Some(wheel) = state.interaction_wheel.as_mut() {
        wheel.elapsed += time.delta_secs();
        if wheel.phase == InteractionWheelPhase::Opening
            && wheel.elapsed >= INTERACTION_WHEEL_ANIMATION_SECONDS
        {
            wheel.phase = InteractionWheelPhase::Open;
            wheel.elapsed = 0.0;
        } else if wheel.phase == InteractionWheelPhase::Closing
            && wheel.elapsed >= INTERACTION_WHEEL_ANIMATION_SECONDS
        {
            pending = wheel.pending.take();
            finished_close = true;
        }
        let scale = interaction_wheel_scale(wheel);
        for mut transform in &mut wheel_roots {
            transform.scale = Vec3::splat(scale);
        }
    }

    if finished_close {
        state.interaction_wheel = None;
        if let Some(interaction) = pending {
            execute_available_interaction(
                &mut state,
                &mut lifecycle,
                &mut navigation,
                &mut requests,
                interaction,
            );
        }
    }
}

fn use_map_exit(
    state: &mut SpatialExplorationState,
    navigation: &mut NavigationState,
    exit_id: &str,
    target_location_id: EntityId,
) {
    let available = state
        .session
        .as_ref()
        .and_then(|session| state.maps.get(&session.location_id))
        .is_some_and(|map| {
            map.exits
                .iter()
                .find(|exit| exit.id == exit_id && exit.target_location_id == target_location_id)
                .is_some_and(|exit| exit_interaction_available(map, state.position, exit))
        });
    if !available {
        state.message = Some("Move closer to the passage and find a clear approach.".to_string());
        state.dirty = true;
        return;
    }

    // Until every connected location has a spatial map, using a physical
    // passage behaves like Leave. Its stable destination ID is retained in
    // LocalMap so this can become a true connected-location transition later.
    exit_spatial(state, navigation);
}

fn exit_spatial(state: &mut SpatialExplorationState, navigation: &mut NavigationState) {
    state.message = None;
    state.interaction_wheel = None;
    state.dirty = true;
    navigation.current_screen = Some(ScreenId::Gameplay);
    navigation.return_screen = None;
    navigation.selected = 0;
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Bevy systems require independent resources and queries.
fn update_spatial_world(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<SpatialExplorationState>,
    navigation: Res<NavigationState>,
    assets: Res<SpatialRenderAssets>,
    mut camera: Query<(&mut Transform, &Camera), (With<Camera2d>, Without<SpatialPlayer>)>,
    mut player: Query<&mut Transform, With<SpatialPlayer>>,
    mut tiles: Query<(&SpatialTile, &mut Sprite), Without<SpatialWallSide>>,
    mut wall_sides: Query<(&SpatialWallSide, &mut Sprite), Without<SpatialTile>>,
    world_entities: Query<Entity, With<SpatialWorldEntity>>,
) {
    if navigation.current_screen != Some(ScreenId::SpatialExploration) {
        for entity in &world_entities {
            commands.entity(entity).despawn();
        }
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };
    let position = session_position(&state);
    let target = map_to_world(position, Some(map));
    for (tile, mut sprite) in &mut tiles {
        let terrain = map.terrain_at(tile.0).unwrap_or(Terrain::Wall);
        if let Some(desired) = assets.tile_images[terrain_image_index(terrain)].clone() {
            if sprite.image != desired {
                sprite.image = desired;
            }
            sprite.color = terrain_tint(map.is_tile_discovered(tile.0));
        }
    }
    for (side, mut sprite) in &mut wall_sides {
        sprite.color = if map.is_tile_discovered(side.tile) {
            Color::WHITE
        } else {
            terrain_tint(false)
        };
    }
    for mut transform in &mut player {
        transform.translation.x = target.x;
        transform.translation.y = target.y + PLAYER_SPRITE_HEIGHT * 0.5;
        transform.translation.z = depth_for_position(position) + PLAYER_DEPTH_OFFSET;
    }
    for (mut transform, camera_component) in &mut camera {
        let viewport = camera_component
            .logical_viewport_size()
            .unwrap_or(Vec2::ZERO);
        let current = transform.translation.truncate();
        let dead_zone_target = camera_target_with_dead_zone(target, current, viewport);
        let camera_target = clamp_camera_target(dead_zone_target, map, viewport);
        let amount = (CAMERA_LERP_SPEED * time.delta_secs()).clamp(0.0, 1.0);
        transform.translation.x += (camera_target.x - transform.translation.x) * amount;
        transform.translation.y += (camera_target.y - transform.translation.y) * amount;
    }
}

fn render_spatial_screen(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    assets: Res<SpatialRenderAssets>,
    mut state: ResMut<SpatialExplorationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
    world_entities: Query<Entity, With<SpatialWorldEntity>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || navigation.current_screen != Some(ScreenId::SpatialExploration)
        || !state.dirty
    {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    for entity in &world_entities {
        commands.entity(entity).despawn();
    }

    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };

    for y in 0..map.height {
        for x in 0..map.width {
            let position = GridPosition { x, y };
            let terrain = map.terrain_at(position).unwrap_or(Terrain::Wall);
            let discovered = map.is_tile_discovered(position);
            let tile_center = Vec2::new(f32::from(x) + 0.5, f32::from(y) + 0.5);
            let world_position = map_to_world(tile_center, Some(map));

            if terrain == Terrain::Wall {
                for (face_index, right_face) in [(0, false), (1, true)] {
                    let Some(image) = assets.wall_face_images[face_index].clone() else {
                        continue;
                    };
                    commands.spawn((
                        SpatialWorldEntity,
                        SpatialWallSide { tile: position },
                        Sprite {
                            image,
                            custom_size: Some(Vec2::new(
                                TILE_HALF_WIDTH,
                                WALL_HEIGHT + TILE_HALF_HEIGHT,
                            )),
                            ..default()
                        },
                        Transform::from_xyz(
                            world_position.x
                                + if right_face {
                                    TILE_HALF_WIDTH * 0.5
                                } else {
                                    -TILE_HALF_WIDTH * 0.5
                                },
                            world_position.y + TILE_HALF_HEIGHT * 0.5,
                            wall_face_depth(tile_center),
                        ),
                    ));
                }
            }

            let top_offset = if terrain == Terrain::Wall {
                WALL_HEIGHT
            } else {
                0.0
            };
            let top_depth = terrain_depth(terrain, tile_center);
            if let Some(image) = assets.tile_images[terrain_image_index(terrain)].clone() {
                let mut tile_sprite = Sprite::from_image(image);
                tile_sprite.custom_size =
                    Some(Vec2::new(TILE_HALF_WIDTH * 2.0, TILE_HALF_HEIGHT * 2.0));
                tile_sprite.color = terrain_tint(discovered);
                commands.spawn((
                    SpatialWorldEntity,
                    SpatialTile(position),
                    tile_sprite,
                    Transform::from_xyz(world_position.x, world_position.y + top_offset, top_depth),
                ));
            }
        }
    }

    for prop in &map.props {
        let (image_index, ground_position, sprite_size) = match prop.kind {
            SpatialPropKind::Pillar => (
                0,
                Vec2::new(
                    f32::from(prop.position.x) + 0.5,
                    f32::from(prop.position.y) + 0.5,
                ),
                Vec2::new(32.0, 40.0),
            ),
            SpatialPropKind::Table => (
                1,
                Vec2::new(
                    f32::from(prop.position.x) + 1.0,
                    f32::from(prop.position.y) + 0.5,
                ),
                Vec2::new(72.0, 48.0),
            ),
        };
        let Some(image) = assets.prop_images[image_index].clone() else {
            continue;
        };
        let projected = map_to_world(ground_position, Some(map));
        let mut sprite = Sprite::from_image(image);
        sprite.custom_size = Some(sprite_size);
        sprite.color = if map.is_tile_discovered(prop.position) {
            Color::WHITE
        } else {
            terrain_tint(false)
        };
        commands.spawn((
            SpatialWorldEntity,
            sprite,
            Transform::from_xyz(
                projected.x,
                projected.y + sprite_size.y * 0.5,
                prop_depth(ground_position),
            ),
        ));
    }

    for exit in &map.exits {
        let exit_grid = Vec2::new(
            f32::from(exit.position.x) + 0.5,
            f32::from(exit.position.y) + 0.5,
        );
        let exit_position = map_to_world(exit_grid, Some(map));
        if let Some(image) = assets.passage_image.clone() {
            commands.spawn((
                SpatialWorldEntity,
                Sprite {
                    image,
                    custom_size: Some(Vec2::new(24.0, 32.0)),
                    ..default()
                },
                Transform::from_xyz(
                    exit_position.x,
                    exit_position.y + 16.0,
                    depth_for(exit_grid) + PLAYER_DEPTH_OFFSET,
                ),
            ));
        }
    }

    if let Some(poi) = map.points_of_interest.first() {
        let poi_grid = Vec2::new(
            f32::from(poi.position.x) + 0.5,
            f32::from(poi.position.y) + 0.5,
        );
        let poi_position = map_to_world(poi_grid, Some(map));
        if let Some(image) = assets.poi_image.clone() {
            let mut sprite = Sprite::from_image(image);
            sprite.custom_size = Some(Vec2::new(24.0, 24.0));
            if poi.interacted {
                sprite.color = Color::srgb(0.62, 0.57, 0.62);
            }
            commands.spawn((
                SpatialWorldEntity,
                sprite,
                Transform::from_xyz(
                    poi_position.x,
                    poi_position.y + PLAYER_SPRITE_HEIGHT * 0.5,
                    depth_for(poi_grid) + PLAYER_DEPTH_OFFSET,
                ),
            ));
        }
    }

    let game = lifecycle.session.as_ref().map(|session| &session.state);
    for placement in &map.interactables {
        let exists = game.is_some_and(|game| match placement.kind {
            SpatialEntityKind::Npc => game.npcs.iter().any(|npc| {
                npc.id == placement.entity_id && npc.location_id == game.character.location_id
            }),
            SpatialEntityKind::Corpse => game.corpses.iter().any(|corpse| {
                corpse.id == placement.entity_id
                    && corpse.location_id == game.character.location_id
                    && !corpse.inventory.is_empty()
            }),
        });
        if !exists {
            continue;
        }
        let image = match placement.kind {
            SpatialEntityKind::Npc => assets.npc_image.clone(),
            SpatialEntityKind::Corpse => assets.corpse_image.clone(),
        };
        let Some(image) = image else {
            continue;
        };
        let grid_position = Vec2::new(
            f32::from(placement.position.x) + 0.5,
            f32::from(placement.position.y) + 0.5,
        );
        let projected = map_to_world(grid_position, Some(map));
        let mut sprite = Sprite::from_image(image);
        sprite.custom_size = Some(match placement.kind {
            SpatialEntityKind::Npc => Vec2::new(17.0, 22.0),
            SpatialEntityKind::Corpse => Vec2::new(21.0, 13.0),
        });
        sprite.color = match placement.kind {
            SpatialEntityKind::Npc => Color::srgb(0.76, 0.80, 0.84),
            SpatialEntityKind::Corpse => Color::srgb(0.76, 0.43, 0.38),
        };
        commands.spawn((
            SpatialWorldEntity,
            SpatialActorEntity,
            sprite,
            Transform::from_xyz(
                projected.x,
                projected.y
                    + if placement.kind == SpatialEntityKind::Npc {
                        11.0
                    } else {
                        5.0
                    },
                depth_for_position(grid_position) + PLAYER_DEPTH_OFFSET - 0.005,
            ),
        ));
    }

    let player_ground_position = session_position(&state);
    let player_position = map_to_world(player_ground_position, Some(map));
    if let Some(image) = assets.player_image.clone() {
        commands.spawn((
            SpatialWorldEntity,
            SpatialPlayer,
            Sprite {
                image,
                custom_size: Some(Vec2::new(PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(
                player_position.x,
                player_position.y + PLAYER_SPRITE_HEIGHT * 0.5,
                depth_for_position(player_ground_position) + PLAYER_DEPTH_OFFSET,
            ),
        ));
    }

    let root = commands
        .spawn((
            BevyScreenRoot,
            SpatialSceneRoot,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(bevy_presentation::responsive_vmin(2.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();

    let header = commands
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    commands.entity(root).add_child(header);
    let location_name = lifecycle
        .session
        .as_ref()
        .and_then(|session| {
            session
                .state
                .world
                .location_by_id(session.state.character.location_id)
        })
        .map(|location| location.name.to_uppercase())
        .unwrap_or_else(|| "UNKNOWN LOCATION".to_string());
    spawn_label(
        &mut commands,
        header,
        format!("{location_name} - LOCAL EXPLORATION"),
    );

    // Keep essential character status anchored to the spatial view. All values
    // come from the authoritative character, not a parallel HUD state.
    let character = lifecycle
        .session
        .as_ref()
        .map(|session| &session.state.character);
    let (character_name, character_level, current_hp, maximum_hp, condition_summary) = character
        .map(|character| {
            (
                character.name.clone(),
                character.level,
                character.hp,
                character.max_hp,
                format_condition_summary(&character.conditions),
            )
        })
        .unwrap_or_else(|| {
            (
                "Character".to_string(),
                1,
                0,
                0,
                "Conditions: unavailable".to_string(),
            )
        });
    let vitals = commands
        .spawn((
            Node {
                width: px(250),
                max_width: percent(36),
                min_width: px(176),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE),
        ))
        .id();
    commands.entity(header).add_child(vitals);
    spawn_label(
        &mut commands,
        vitals,
        format!("{character_name} / LVL {character_level}"),
    );
    bevy_presentation::spawn_health_gauge(&mut commands, vitals, current_hp, maximum_hp);
    spawn_compact_label(&mut commands, vitals, condition_summary);

    spawn_control(
        &mut commands,
        header,
        "Leave",
        SpatialAction::Exit,
        Display::Flex,
    );

    let status = commands
        .spawn((
            Node {
                width: percent(100),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    commands.entity(root).add_child(status);
    spawn_label(
        &mut commands,
        status,
        state
            .message
            .as_deref()
            .unwrap_or("Move carefully. The walls are not all stone."),
    );

    let footer = commands
        .spawn(Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            ..default()
        })
        .id();
    commands.entity(root).add_child(footer);

    #[cfg(target_os = "android")]
    spawn_virtual_joystick(&mut commands, root);

    let interaction_available = lifecycle.session.as_ref().is_some_and(|game_session| {
        nearest_interaction_target(map, &game_session.state, session_position(&state)).is_some()
    });
    let interaction_display = if interaction_available {
        Display::Flex
    } else {
        Display::None
    };
    spawn_control(
        &mut commands,
        footer,
        "Interact",
        SpatialAction::Interact,
        interaction_display,
    );
    state.dirty = false;
}

#[cfg(target_os = "android")]
fn spawn_virtual_joystick(commands: &mut Commands, parent: Entity) {
    let base = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(JOYSTICK_MARGIN_PX),
                bottom: px(JOYSTICK_MARGIN_PX),
                width: px(JOYSTICK_RADIUS_PX * 2.0),
                height: px(JOYSTICK_RADIUS_PX * 2.0),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::MAX,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.07, 0.07, 0.07, 0.70)),
            BorderColor::all(bevy_presentation::THEME_BORDER),
        ))
        .id();
    commands.entity(parent).add_child(base);

    let thumb = commands
        .spawn((
            SpatialJoystickThumb,
            Node {
                position_type: PositionType::Absolute,
                left: px(JOYSTICK_RADIUS_PX - JOYSTICK_THUMB_RADIUS_PX),
                top: px(JOYSTICK_RADIUS_PX - JOYSTICK_THUMB_RADIUS_PX),
                width: px(JOYSTICK_THUMB_RADIUS_PX * 2.0),
                height: px(JOYSTICK_THUMB_RADIUS_PX * 2.0),
                border_radius: BorderRadius::MAX,
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_ACCENT),
            BorderColor::all(bevy_presentation::THEME_TEXT),
        ))
        .id();
    commands.entity(base).add_child(thumb);
}

#[cfg(target_os = "android")]
fn update_virtual_joystick(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    navigation: Res<NavigationState>,
    spatial: Res<SpatialExplorationState>,
    mut joystick: ResMut<SpatialJoystickState>,
    mut thumbs: Query<&mut Node, With<SpatialJoystickThumb>>,
) {
    if navigation.current_screen != Some(ScreenId::SpatialExploration)
        || spatial.interaction_wheel.is_some()
    {
        joystick.active_touch = None;
        joystick.direction = Vec2::ZERO;
        return;
    }

    let center = Vec2::new(
        JOYSTICK_MARGIN_PX + JOYSTICK_RADIUS_PX,
        window.height() - JOYSTICK_MARGIN_PX - JOYSTICK_RADIUS_PX,
    );

    if joystick.active_touch.is_none() {
        joystick.active_touch = touches
            .iter_just_pressed()
            .find(|touch| touch.position().distance(center) <= JOYSTICK_RADIUS_PX)
            .map(|touch| touch.id());
    }

    let offset = joystick.active_touch.and_then(|id| {
        touches
            .iter()
            .find(|touch| touch.id() == id)
            .map(|touch| touch.position() - center)
    });
    if let Some(offset) = offset {
        joystick.direction = virtual_joystick_direction(offset);
        let thumb_offset = offset.clamp_length_max(JOYSTICK_THUMB_TRAVEL_PX);
        for mut node in &mut thumbs {
            node.left = px(JOYSTICK_RADIUS_PX - JOYSTICK_THUMB_RADIUS_PX + thumb_offset.x);
            node.top = px(JOYSTICK_RADIUS_PX - JOYSTICK_THUMB_RADIUS_PX + thumb_offset.y);
        }
    } else {
        joystick.active_touch = None;
        joystick.direction = Vec2::ZERO;
        for mut node in &mut thumbs {
            node.left = px(JOYSTICK_RADIUS_PX - JOYSTICK_THUMB_RADIUS_PX);
            node.top = px(JOYSTICK_RADIUS_PX - JOYSTICK_THUMB_RADIUS_PX);
        }
    }
}

#[cfg(any(target_os = "android", test))]
fn virtual_joystick_direction(screen_offset: Vec2) -> Vec2 {
    let normalized_distance = (screen_offset.length() / JOYSTICK_RADIUS_PX).clamp(0.0, 1.0);
    if normalized_distance <= JOYSTICK_DEAD_ZONE {
        return Vec2::ZERO;
    }

    // Touch coordinates increase downward, but spatial movement treats positive Y as up.
    Vec2::new(screen_offset.x, -screen_offset.y).normalize()
        * ((normalized_distance - JOYSTICK_DEAD_ZONE) / (1.0 - JOYSTICK_DEAD_ZONE))
}

fn spatial_player_screen_position(
    position: Vec2,
    map: &LocalMap,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Option<(Vec2, Vec2)> {
    let projected = map_to_world(position, Some(map));
    let player_world = Vec3::new(
        projected.x,
        projected.y + PLAYER_SPRITE_HEIGHT * 0.5,
        0.0,
    );
    cameras.iter().find_map(|(camera, camera_transform)| {
        let screen_position = camera
            .world_to_viewport(camera_transform, player_world)
            .ok()?;
        let viewport = camera.logical_viewport_size()?;
        Some((screen_position, viewport))
    })
}

fn interaction_wheel_root_position(anchor: Vec2, viewport: Vec2) -> Vec2 {
    let maximum = Vec2::new(
        (viewport.x - INTERACTION_WHEEL_SIZE_PX - 8.0).max(8.0),
        (viewport.y - INTERACTION_WHEEL_SIZE_PX - 8.0).max(8.0),
    );
    Vec2::new(
        (anchor.x - INTERACTION_WHEEL_SIZE_PX * 0.5).clamp(8.0, maximum.x),
        (anchor.y - INTERACTION_WHEEL_SIZE_PX * 0.5).clamp(8.0, maximum.y),
    )
}

fn sync_spatial_interaction_wheel(
    mut commands: Commands,
    state: Res<SpatialExplorationState>,
    navigation: Res<NavigationState>,
    spatial_roots: Query<Entity, With<SpatialSceneRoot>>,
    wheel_roots: Query<Entity, With<SpatialInteractionWheelRoot>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if navigation.current_screen != Some(ScreenId::SpatialExploration) {
        for root in &wheel_roots {
            commands.entity(root).despawn();
        }
        return;
    }
    let Some(wheel) = state.interaction_wheel.as_ref() else {
        for root in &wheel_roots {
            commands.entity(root).despawn();
        }
        return;
    };
    if !wheel_roots.is_empty() {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };
    let Some((anchor, viewport)) =
        spatial_player_screen_position(state.position, map, &cameras)
    else {
        return;
    };
    let Some(parent) = spatial_roots.iter().next() else {
        return;
    };
    spawn_interaction_wheel(
        &mut commands,
        parent,
        wheel,
        interaction_wheel_root_position(anchor, viewport),
    );
}

fn update_spatial_wheel_anchor(
    state: Res<SpatialExplorationState>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut wheel_roots: Query<&mut Node, With<SpatialInteractionWheelRoot>>,
) {
    if state.interaction_wheel.is_none() {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };
    let Some((anchor, viewport)) =
        spatial_player_screen_position(state.position, map, &cameras)
    else {
        return;
    };
    let root_position = interaction_wheel_root_position(anchor, viewport);
    for mut node in &mut wheel_roots {
        node.left = px(root_position.x);
        node.top = px(root_position.y);
    }
}

fn interaction_wheel_entry_position(index: usize, count: usize, button_width: f32) -> Vec2 {
    let angle = -std::f32::consts::FRAC_PI_2
        + std::f32::consts::TAU * index as f32 / count as f32;
    let radius = 102.0;
    let center = Vec2::splat(INTERACTION_WHEEL_SIZE_PX * 0.5);
    let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
    center + offset - Vec2::new(button_width * 0.5, INTERACTION_WHEEL_BUTTON_HEIGHT_PX * 0.5)
}

fn spawn_interaction_wheel(
    commands: &mut Commands,
    parent: Entity,
    wheel: &SpatialInteractionWheel,
    position: Vec2,
) {
    let root = commands
        .spawn((
            SpatialInteractionWheelRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                top: px(position.y),
                width: px(INTERACTION_WHEEL_SIZE_PX),
                height: px(INTERACTION_WHEEL_SIZE_PX),
                ..default()
            },
            Transform::from_scale(Vec3::splat(interaction_wheel_scale(wheel))),
            BackgroundColor(Color::NONE),
        ))
        .id();
    commands.entity(parent).add_child(root);

    let ring_size = 224.0;
    let ring = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px((INTERACTION_WHEEL_SIZE_PX - ring_size) * 0.5),
                top: px((INTERACTION_WHEEL_SIZE_PX - ring_size) * 0.5),
                width: px(ring_size),
                height: px(ring_size),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(Color::srgba(0.035, 0.035, 0.04, 0.88)),
        ))
        .id();
    commands.entity(root).add_child(ring);

    let count = wheel.entries.len();
    let button_width = if count <= 4 {
        104.0
    } else if count <= 6 {
        92.0
    } else {
        78.0
    };
    for (index, interaction) in wheel.entries.iter().enumerate() {
        let entry_position = interaction_wheel_entry_position(index, count, button_width);
        spawn_interaction_wheel_entry(
            commands,
            root,
            index,
            &interaction.label,
            entry_position,
            button_width,
        );
    }
    spawn_interaction_wheel_close_button(commands, root);
}

fn spawn_interaction_wheel_entry(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    label: &str,
    position: Vec2,
    width: f32,
) {
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            bevy_presentation::UiTouchTargetShape::MinimumBounds {
                width: width as u32,
                height: INTERACTION_WHEEL_BUTTON_HEIGHT_PX as u32,
            },
            SpatialControlButton,
            SpatialWheelEntry(index),
            SpatialAction::SelectInteraction(index),
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                top: px(position.y),
                width: px(width),
                height: px(INTERACTION_WHEEL_BUTTON_HEIGHT_PX),
                padding: UiRect::axes(px(5), px(4)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    let text = commands
        .spawn((
            Text::new(label.to_string()),
            Node {
                width: percent(100),
                ..default()
            },
            TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
            TextFont::from_font_size(FontSize::Rem(if width < 90.0 { 0.68 } else { 0.78 })),
            TextColor(bevy_presentation::THEME_TEXT),
        ))
        .id();
    commands.entity(button).add_child(text);
    commands.entity(parent).add_child(button);
}

fn spawn_interaction_wheel_close_button(commands: &mut Commands, parent: Entity) {
    let close = commands
        .spawn((
            Button,
            Hovered::default(),
            bevy_presentation::UiTouchTargetShape::MinimumBounds {
                width: 44,
                height: 44,
            },
            SpatialControlButton,
            SpatialAction::CloseWheel,
            Node {
                position_type: PositionType::Absolute,
                left: px((INTERACTION_WHEEL_SIZE_PX - 44.0) * 0.5),
                top: px((INTERACTION_WHEEL_SIZE_PX - 44.0) * 0.5),
                width: px(44),
                height: px(44),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    let label = commands
        .spawn((
            Text::new("X"),
            TextFont::from_font_size(FontSize::Rem(0.9)),
            TextColor(bevy_presentation::THEME_TEXT),
        ))
        .id();
    commands.entity(close).add_child(label);
    commands.entity(parent).add_child(close);
}

fn interaction_wheel_scale(wheel: &SpatialInteractionWheel) -> f32 {
    let progress = (wheel.elapsed / INTERACTION_WHEEL_ANIMATION_SECONDS).clamp(0.0, 1.0);
    let eased = progress * progress * (3.0 - 2.0 * progress);
    match wheel.phase {
        InteractionWheelPhase::Opening => 0.78 + 0.22 * eased,
        InteractionWheelPhase::Open => 1.0,
        InteractionWheelPhase::Closing => 1.0 - 0.22 * eased,
    }
}

fn interaction_wheel_index_for_direction(count: usize, direction: Vec2) -> Option<usize> {
    if count == 0 || direction.length_squared() <= f32::EPSILON {
        return None;
    }
    let direction = direction.normalize();
    (0..count).max_by(|left, right| {
        let angle_for = |index: usize| {
            std::f32::consts::FRAC_PI_2 - std::f32::consts::TAU * index as f32 / count as f32
        };
        let left_score =
            Vec2::new(angle_for(*left).cos(), angle_for(*left).sin()).dot(direction);
        let right_score =
            Vec2::new(angle_for(*right).cos(), angle_for(*right).sin()).dot(direction);
        left_score.total_cmp(&right_score)
    })
}

fn spawn_control(
    commands: &mut Commands,
    parent: Entity,
    label: impl Into<String>,
    action: SpatialAction,
    display: Display,
) -> Entity {
    let mut entity = commands.spawn((
        Button,
        Hovered::default(),
        bevy_presentation::UiTouchTargetShape::MinimumBounds {
            width: 52,
            height: 48,
        },
        SpatialControlButton,
        action,
        Node {
            display,
            min_width: px(52),
            min_height: px(48),
            padding: UiRect::axes(px(10), px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            ..default()
        },
        BorderColor::all(bevy_presentation::THEME_BORDER),
        BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
    ));
    if matches!(action, SpatialAction::Interact) {
        entity.insert(SpatialInteractionPrompt);
    }
    let entity = entity.id();
    let text = commands
        .spawn((
            Text::new(label.into()),
            TextFont::from_font_size(FontSize::Rem(0.9)),
            TextColor(bevy_presentation::THEME_TEXT),
        ))
        .id();
    if matches!(action, SpatialAction::Interact) {
        commands.entity(text).insert(SpatialPromptLabel);
    }
    commands.entity(entity).add_child(text);
    commands.entity(parent).add_child(entity);
    entity
}

fn spawn_label(commands: &mut Commands, parent: Entity, label: impl Into<String>) {
    let label = commands
        .spawn((
            Text::new(label.into()),
            TextFont::from_font_size(FontSize::Rem(0.9)),
            TextColor(bevy_presentation::THEME_TEXT),
        ))
        .id();
    commands.entity(parent).add_child(label);
}

fn spawn_compact_label(commands: &mut Commands, parent: Entity, label: impl Into<String>) {
    let label = commands
        .spawn((
            Text::new(label.into()),
            TextFont::from_font_size(FontSize::Rem(0.648)),
            TextColor(bevy_presentation::THEME_MUTED),
            Node {
                width: percent(100),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(label);
}

fn format_condition_summary(conditions: &[Condition]) -> String {
    if conditions.is_empty() {
        "Conditions: none".to_string()
    } else {
        let names = conditions
            .iter()
            .map(|condition| condition.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!("Conditions: {names}")
    }
}

fn update_spatial_context_prompt(
    state: Res<SpatialExplorationState>,
    navigation: Res<NavigationState>,
    lifecycle: Res<LifecycleState>,
    mut poi_prompts: Query<&mut Node, With<SpatialInteractionPrompt>>,
    mut prompt_labels: Query<&mut Text, With<SpatialPromptLabel>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || navigation.current_screen != Some(ScreenId::SpatialExploration)
    {
        return;
    }
    let current_map = state
        .session
        .as_ref()
        .and_then(|session| state.maps.get(&session.location_id));
    let game = lifecycle.session.as_ref().map(|session| &session.state);
    let interactions = current_map
        .zip(game)
        .map(|(map, game)| available_interactions(map, game, session_position(&state)))
        .unwrap_or_default();
    let interaction_display =
        if !interactions.is_empty() && state.interaction_wheel.is_none() {
            Display::Flex
        } else {
            Display::None
        };
    for mut node in &mut poi_prompts {
        if node.display != interaction_display {
            node.display = interaction_display;
        }
    }
    let label = interactions
        .first()
        .map(|interaction| interaction_prompt_label(interaction, interactions.len()))
        .unwrap_or_else(|| "Interact".to_string());
    for mut text in &mut prompt_labels {
        *text = Text::new(label.clone());
    }
}

fn update_spatial_control_feedback(
    state: Res<SpatialExplorationState>,
    mut controls: Query<(
        &Hovered,
        Has<Pressed>,
        Option<&SpatialWheelEntry>,
        &mut BackgroundColor,
        &mut BorderColor,
    ), With<SpatialControlButton>>,
) {
    for (hovered, pressed, wheel_entry, mut background, mut border) in &mut controls {
        let selected = wheel_entry.is_some_and(|entry| {
            state
                .interaction_wheel
                .as_ref()
                .is_some_and(|wheel| wheel.selected == entry.0)
        });
        background.0 = if selected {
            bevy_presentation::THEME_SELECTED
        } else if pressed {
            bevy_presentation::THEME_PRESSED
        } else if hovered.get() {
            bevy_presentation::THEME_HOVER
        } else {
            bevy_presentation::THEME_SURFACE_STRONG
        };
        *border = BorderColor::all(if selected {
            bevy_presentation::THEME_ACCENT
        } else {
            bevy_presentation::THEME_BORDER
        });
    }
}

fn terrain_image_index(terrain: Terrain) -> usize {
    match terrain {
        Terrain::Floor => 0,
        Terrain::Ash => 1,
        Terrain::Rubble => 2,
        Terrain::Wall => 3,
    }
}

fn terrain_tint(discovered: bool) -> Color {
    if discovered {
        Color::WHITE
    } else {
        Color::srgb(0.30, 0.28, 0.34)
    }
}

fn map_to_world(position: Vec2, map: Option<&LocalMap>) -> Vec2 {
    let (width, height) = map
        .map(|map| (f32::from(map.width), f32::from(map.height)))
        .unwrap_or((24.0, 16.0));
    Vec2::new(
        (position.x - position.y - (width - height) * 0.5) * TILE_HALF_WIDTH,
        ((width + height) * 0.5 - position.x - position.y) * TILE_HALF_HEIGHT,
    )
}

fn depth_for(position: Vec2) -> f32 {
    (position.x + position.y) * DEPTH_STEP
}

/// Ground terrain is a background surface, not an occluder. Only wall tops
/// participate in the painter's order with actors and other world objects.
fn terrain_depth(terrain: Terrain, tile_center: Vec2) -> f32 {
    match terrain {
        Terrain::Floor | Terrain::Ash | Terrain::Rubble => GROUND_TERRAIN_DEPTH,
        // A wall top's ground-contact line is the tile center. Adjacent cells
        // can share the same ground depth in this projection; favor the actor on
        // that tie so the full top diamond does not hide a laterally adjacent player.
        Terrain::Wall => depth_for(tile_center) + PLAYER_DEPTH_OFFSET - WALL_TOP_DEPTH_BIAS,
    }
}

fn wall_face_depth(tile_center: Vec2) -> f32 {
    // Both wall faces meet the ground at the tile's front edge, one depth step
    // closer than the tile center. This keeps occlusion aligned with their base.
    depth_for(tile_center + Vec2::splat(0.5)) + PLAYER_DEPTH_OFFSET + WALL_FACE_DEPTH_BIAS
}

/// Props sort by their ground-contact line, like moving actors. The small
/// negative bias gives the player priority on an exact depth tie.
fn prop_depth(ground_position: Vec2) -> f32 {
    depth_for(ground_position) + PLAYER_DEPTH_OFFSET - WALL_TOP_DEPTH_BIAS
}

/// Moving actors sort by their continuous ground-contact position. Ground
/// terrain uses its own background layer, so smooth depth changes cannot put
/// the player behind a floor diamond while crossing a tile.
fn depth_for_position(position: Vec2) -> f32 {
    depth_for(position)
}

fn screen_direction_to_world(screen_direction: Vec2) -> Vec2 {
    if screen_direction.length_squared() == 0.0 {
        return Vec2::ZERO;
    }
    let direction = screen_direction.normalize();
    Vec2::new(
        direction.x / (2.0 * TILE_HALF_WIDTH) - direction.y / (2.0 * TILE_HALF_HEIGHT),
        -direction.x / (2.0 * TILE_HALF_WIDTH) - direction.y / (2.0 * TILE_HALF_HEIGHT),
    )
}

fn camera_target_with_dead_zone(player: Vec2, camera: Vec2, viewport: Vec2) -> Vec2 {
    if viewport.x <= 0.0 || viewport.y <= 0.0 {
        return player;
    }

    let half_dead_zone = viewport * (CAMERA_DEAD_ZONE_FRACTION * 0.5);
    let offset = player - camera;
    let correction = Vec2::new(
        if offset.x > half_dead_zone.x {
            offset.x - half_dead_zone.x
        } else if offset.x < -half_dead_zone.x {
            offset.x + half_dead_zone.x
        } else {
            0.0
        },
        if offset.y > half_dead_zone.y {
            offset.y - half_dead_zone.y
        } else if offset.y < -half_dead_zone.y {
            offset.y + half_dead_zone.y
        } else {
            0.0
        },
    );
    camera + correction
}

fn clamp_camera_axis(target: f32, min: f32, max: f32, viewport_half_extent: f32) -> f32 {
    let lower = min + viewport_half_extent;
    let upper = max - viewport_half_extent;
    if lower > upper {
        (min + max) * 0.5
    } else {
        target.clamp(lower, upper)
    }
}

fn clamp_camera_target(target: Vec2, map: &LocalMap, viewport: Vec2) -> Vec2 {
    let map_half_width = (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_WIDTH * 0.5;
    let map_half_height = (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_HEIGHT * 0.5;
    // Wall tops extend farther upward than the floor tiles extend downward.
    let min_y = -map_half_height - (WALL_HEIGHT - TILE_HALF_HEIGHT);
    let max_y = map_half_height;
    Vec2::new(
        clamp_camera_axis(target.x, -map_half_width, map_half_width, viewport.x * 0.5),
        clamp_camera_axis(target.y, min_y, max_y, viewport.y * 0.5),
    )
}

pub(crate) fn available_interactions(
    map: &LocalMap,
    game: &GameState,
    position: Vec2,
) -> Vec<AvailableInteraction> {
    let mut candidates = Vec::new();

    for poi in &map.points_of_interest {
        if !poi_interaction_available_for(map, position, poi) {
            continue;
        }
        let Some(distance) = poi_distance_px_for(map, position, poi) else {
            continue;
        };
        let item = AvailableInteraction {
            action: InteractionActionKind::Examine,
            target: InteractionTarget::PointOfInterest(poi.id.clone()),
            label: format!("Examine — {}", poi.title),
        };
        candidates.push((distance, 2_u8, item.stable_key(), item));
    }

    for placement in &map.interactables {
        let target_position = Vec2::new(
            f32::from(placement.position.x) + 0.5,
            f32::from(placement.position.y) + 0.5,
        );
        let distance =
            map_to_world(position, Some(map)).distance(map_to_world(target_position, Some(map)));
        if distance > INTERACTION_RANGE_PX || !has_line_of_sight(map, position, target_position) {
            continue;
        }
        let (priority, item) = match placement.kind {
            SpatialEntityKind::Npc => {
                let Some(npc) = game.npcs.iter().find(|npc| {
                    npc.id == placement.entity_id && npc.location_id == game.character.location_id
                }) else {
                    continue;
                };
                (
                    0_u8,
                    AvailableInteraction {
                        action: InteractionActionKind::Talk,
                        target: InteractionTarget::Npc(npc.id),
                        label: format!("Talk — {}", npc.name),
                    },
                )
            }
            SpatialEntityKind::Corpse => {
                let Some(corpse) = game.corpses.iter().find(|corpse| {
                    corpse.id == placement.entity_id
                        && corpse.location_id == game.character.location_id
                        && !corpse.inventory.is_empty()
                }) else {
                    continue;
                };
                (
                    1_u8,
                    AvailableInteraction {
                        action: InteractionActionKind::SearchRemains,
                        target: InteractionTarget::Corpse(corpse.id),
                        label: format!("Search — {}", corpse.former_name),
                    },
                )
            }
        };
        candidates.push((distance, priority, item.target.stable_key(), item));
    }

    for exit in &map.exits {
        if !exit_interaction_available(map, position, exit) {
            continue;
        }
        let label = game
            .world
            .location_by_id(exit.target_location_id)
            .map(|location| format!("Enter — {}", location.name))
            .unwrap_or_else(|| "Use passage".to_string());
        let item = AvailableInteraction {
            action: InteractionActionKind::Enter,
            target: InteractionTarget::Passage {
                id: exit.id.clone(),
                destination_id: exit.target_location_id,
            },
            label,
        };
        candidates.push((
            exit_distance_px(map, position, exit),
            3_u8,
            item.stable_key(),
            item,
        ));
    }

    candidates.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then(a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    candidates.into_iter().map(|(_, _, _, item)| item).collect()
}

fn nearest_interaction_target(
    map: &LocalMap,
    game: &GameState,
    position: Vec2,
) -> Option<AvailableInteraction> {
    available_interactions(map, game, position)
        .into_iter()
        .next()
}

fn interaction_prompt_label(interaction: &AvailableInteraction, count: usize) -> String {
    if count > 1 {
        format!("{} (+{})", interaction.label, count - 1)
    } else {
        interaction.label.clone()
    }
}

fn exit_distance_px(map: &LocalMap, position: Vec2, exit: &MapExit) -> f32 {
    let target = Vec2::new(
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    map_to_world(position, Some(map)).distance(map_to_world(target, Some(map)))
}

fn exit_interaction_available(map: &LocalMap, position: Vec2, exit: &MapExit) -> bool {
    if !map.contains(exit.position)
        || !map.is_walkable(exit.position)
        || exit_distance_px(map, position, exit) > INTERACTION_RANGE_PX
    {
        return false;
    }
    let target = Vec2::new(
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    has_line_of_sight(map, position, target)
}

fn poi_distance_px_for(
    map: &LocalMap,
    position: Vec2,
    poi: &crate::spatial::PointOfInterest,
) -> Option<f32> {
    if !map.contains(poi.position) {
        return None;
    }
    let target = Vec2::new(
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );
    Some(map_to_world(position, Some(map)).distance(map_to_world(target, Some(map))))
}

fn poi_interaction_available_for(
    map: &LocalMap,
    position: Vec2,
    poi: &crate::spatial::PointOfInterest,
) -> bool {
    if poi_distance_px_for(map, position, poi)
        .is_none_or(|distance| distance > INTERACTION_RANGE_PX)
    {
        return false;
    }
    let target = Vec2::new(
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );
    has_line_of_sight(map, position, target)
}

fn has_line_of_sight(map: &LocalMap, start: Vec2, target: Vec2) -> bool {
    let distance = start.distance(target);
    let steps = (distance * 8.0).ceil() as u32;
    for step in 1..steps {
        let sample = start.lerp(target, step as f32 / steps.max(1) as f32);
        if sample.x < 0.0 || sample.y < 0.0 {
            return false;
        }
        let tile = GridPosition {
            x: sample.x.floor() as u16,
            y: sample.y.floor() as u16,
        };
        if !map.is_walkable(tile) {
            return false;
        }
    }
    true
}

fn advance_position(
    map: &LocalMap,
    position: Vec2,
    world_units_per_pixel: Vec2,
    delta_secs: f32,
    speed_pixels_per_second: f32,
) -> Vec2 {
    if world_units_per_pixel.length_squared() == 0.0 {
        return position;
    }
    let delta = world_units_per_pixel * speed_pixels_per_second * delta_secs.clamp(0.0, 0.1);
    let mut result = position;
    let candidate_x = result.x + delta.x;
    if is_position_walkable(map, Vec2::new(candidate_x, result.y)) {
        result.x = candidate_x;
    }
    let candidate_y = result.y + delta.y;
    if is_position_walkable(map, Vec2::new(result.x, candidate_y)) {
        result.y = candidate_y;
    }
    result
}

fn is_position_walkable(map: &LocalMap, position: Vec2) -> bool {
    let radius = PLAYER_COLLISION_HALF_EXTENT;
    let samples = [
        Vec2::ZERO,
        Vec2::new(-radius, 0.0),
        Vec2::new(radius, 0.0),
        Vec2::new(0.0, -radius),
        Vec2::new(0.0, radius),
        Vec2::new(-radius, -radius),
        Vec2::new(-radius, radius),
        Vec2::new(radius, -radius),
        Vec2::new(radius, radius),
    ];
    samples.into_iter().all(|offset| {
        let sample = position + offset;
        if sample.x < 0.0
            || sample.y < 0.0
            || sample.x >= f32::from(map.width)
            || sample.y >= f32::from(map.height)
        {
            return false;
        }
        map.is_walkable(GridPosition {
            x: sample.x.floor() as u16,
            y: sample.y.floor() as u16,
        })
    })
}

#[cfg(test)]
#[path = "../tests/unit/bevy_spatial.rs"]
mod tests;
