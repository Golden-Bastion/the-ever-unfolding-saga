//! Bevy presentation and interaction for NPC dialogue and character remains.
//!
//! The interaction rules remain in the game modules. This adapter only owns
//! screen state, selection, and rendering for the graphical frontend.

use bevy::prelude::*;
use bevy::ui::UiTransform;

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, PresentationTransition, ScreenId,
};
use crate::bevy_spatial::{SpatialInteractionRequest, SpatialSceneRoot};
use crate::game::{interactions, legacy};
use crate::input::InputEvent;
use crate::model::EntityId;
use crate::presentation::{
    CharacterPortraitLayerView, CharacterPortraitView, ConversationView, RemainsResultView,
    RemainsView, TalkView,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionScreen {
    Talk,
    Conversation,
    Remains,
    RemainsResult,
}

#[derive(Resource)]
pub(crate) struct BevyInteractionState {
    screen: InteractionScreen,
    selected: usize,
    npc_id: Option<EntityId>,
    result: Option<RemainsResultView>,
    message: Option<String>,
    spatial_overlay: bool,
    spatial_overlay_closing: bool,
    corpse_id: Option<EntityId>,
    dirty: bool,
}

impl Default for BevyInteractionState {
    fn default() -> Self {
        Self {
            screen: InteractionScreen::Talk,
            selected: 0,
            npc_id: None,
            result: None,
            message: None,
            spatial_overlay: false,
            spatial_overlay_closing: false,
            corpse_id: None,
            dirty: true,
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<BevyInteractionState>()
        .init_resource::<PortraitImageCache>()
        .add_systems(
            Update,
            (
                read_spatial_requests,
                sync_screen,
                interaction_input,
                begin_spatial_overlay_exit,
                render_if_active,
                finish_spatial_overlay_exit
                    .after(bevy_presentation::update_presentation_transitions),
            )
                .chain(),
        );
}

#[derive(Resource, Default)]
pub(crate) struct PortraitImageCache {
    images: HashMap<String, Handle<Image>>,
    pub(crate) missing: HashSet<String>,
    pub(crate) failures: HashMap<String, String>,
}

#[derive(Component)]
struct SpatialInteractionOverlayRoot;

pub(crate) fn spatial_overlay_open(state: &BevyInteractionState) -> bool {
    state.spatial_overlay
}

fn read_spatial_requests(
    mut requests: MessageReader<SpatialInteractionRequest>,
    mut state: ResMut<BevyInteractionState>,
    mut navigation: ResMut<NavigationState>,
) {
    if navigation.current_screen != Some(ScreenId::SpatialExploration) {
        requests.clear();
        return;
    }
    for request in requests.read() {
        match request {
            SpatialInteractionRequest::Talk(npc_id) => {
                state.screen = InteractionScreen::Conversation;
                state.npc_id = Some(*npc_id);
                state.corpse_id = None;
            }
            SpatialInteractionRequest::SearchRemains(corpse_id) => {
                state.screen = InteractionScreen::Remains;
                state.npc_id = None;
                state.corpse_id = Some(*corpse_id);
            }
        }
        state.selected = 0;
        state.result = None;
        state.message = None;
        state.spatial_overlay = true;
        state.spatial_overlay_closing = false;
        state.dirty = true;
        navigation.selected = 0;
    }
}

fn is_interaction_screen(screen: Option<ScreenId>) -> bool {
    matches!(
        screen,
        Some(ScreenId::Talk | ScreenId::Conversation | ScreenId::RemainsResult)
    )
}

fn sync_screen(
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut state: ResMut<BevyInteractionState>,
) {
    if navigation.current_screen == Some(ScreenId::SpatialExploration) {
        return;
    }
    if state.spatial_overlay {
        state.spatial_overlay = false;
        state.spatial_overlay_closing = false;
        state.corpse_id = None;
        state.message = None;
        state.dirty = true;
    }
    if lifecycle.phase != LifecyclePhase::Complete
        || !is_interaction_screen(navigation.current_screen)
    {
        return;
    }
    let Some(screen) = navigation.current_screen else {
        return;
    };

    let target = match screen {
        ScreenId::Talk => InteractionScreen::Talk,
        ScreenId::Conversation => InteractionScreen::Conversation,
        ScreenId::RemainsResult => InteractionScreen::RemainsResult,
        _ => return,
    };
    if state.screen == target && state.dirty {
        return;
    }
    if state.screen != target {
        state.screen = target;
        state.selected = navigation.selected;
        state.message = None;
        state.dirty = true;
    }
}

fn selection_count(state: &BevyInteractionState, lifecycle: &LifecycleState) -> usize {
    let Some(session) = lifecycle.session.as_ref() else {
        return 1;
    };
    if state.spatial_overlay {
        if state.message.is_some() {
            return 1;
        }
        return match state.screen {
            InteractionScreen::Conversation => state
                .npc_id
                .and_then(|npc_id| interactions::npc_index_by_id(&session.state, npc_id))
                .and_then(|npc_index| {
                    interactions::build_conversation_view_for_bevy(&session.state, npc_index)
                })
                .filter(|view| view.available)
                .map(|view| view.options.len() + 1)
                .unwrap_or(1),
            InteractionScreen::Remains => state
                .corpse_id
                .filter(|corpse_id| {
                    legacy::build_remains_view_for_bevy(&session.state)
                        .remains
                        .iter()
                        .any(|remains| remains.id == *corpse_id && !remains.items.is_empty())
                })
                .map(|_| 2)
                .unwrap_or(1),
            InteractionScreen::RemainsResult | InteractionScreen::Talk => 1,
        };
    }
    match state.screen {
        InteractionScreen::Talk => {
            interactions::build_talk_view_for_bevy(&session.state)
                .npcs
                .len()
                + 1
        }
        InteractionScreen::Conversation => state
            .npc_id
            .and_then(|npc_id| interactions::npc_index_by_id(&session.state, npc_id))
            .and_then(|npc_index| {
                interactions::build_conversation_view_for_bevy(&session.state, npc_index)
                    .map(|view| view.options.len() + 1)
            })
            .unwrap_or(1),
        InteractionScreen::Remains => {
            legacy::build_remains_view_for_bevy(&session.state)
                .remains
                .len()
                + 1
        }
        InteractionScreen::RemainsResult => 1,
    }
}

fn move_selection(
    state: &mut BevyInteractionState,
    lifecycle: &LifecycleState,
    navigation: &mut NavigationState,
    direction: isize,
) {
    let count = selection_count(state, lifecycle);
    if count == 0 {
        return;
    }
    state.selected = (state.selected as isize + direction).rem_euclid(count as isize) as usize;
    navigation.selected = state.selected;
    state.dirty = true;
}

fn move_to_end(
    state: &mut BevyInteractionState,
    lifecycle: &LifecycleState,
    navigation: &mut NavigationState,
) {
    let count = selection_count(state, lifecycle);
    if count > 0 {
        state.selected = count - 1;
        navigation.selected = state.selected;
        state.dirty = true;
    }
}

fn close_to_gameplay(state: &mut BevyInteractionState, navigation: &mut NavigationState) {
    state.selected = 0;
    state.npc_id = None;
    state.corpse_id = None;
    state.result = None;
    state.message = None;
    state.spatial_overlay = false;
    state.spatial_overlay_closing = false;
    state.dirty = true;
    navigation.current_screen = Some(ScreenId::Gameplay);
    navigation.return_screen = None;
    navigation.selected = 0;
}

fn interaction_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut state: ResMut<BevyInteractionState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    let spatial_overlay =
        navigation.current_screen == Some(ScreenId::SpatialExploration) && state.spatial_overlay;
    if spatial_overlay && state.spatial_overlay_closing {
        input_queue.0.clear();
        return;
    }
    if lifecycle.phase != LifecyclePhase::Complete
        || (!spatial_overlay && !is_interaction_screen(navigation.current_screen))
        || input_queue.0.is_empty()
    {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => move_selection(&mut state, &lifecycle, &mut navigation, -1),
            InputEvent::Down => move_selection(&mut state, &lifecycle, &mut navigation, 1),
            InputEvent::Home => {
                state.selected = 0;
                navigation.selected = 0;
                state.dirty = true;
            }
            InputEvent::End => move_to_end(&mut state, &lifecycle, &mut navigation),
            InputEvent::PageUp => move_selection(&mut state, &lifecycle, &mut navigation, -5),
            InputEvent::PageDown => move_selection(&mut state, &lifecycle, &mut navigation, 5),
            InputEvent::Confirm if spatial_overlay => {
                activate_spatial_overlay(&mut lifecycle, &mut state, &mut navigation)
            }
            InputEvent::Cancel if spatial_overlay => {
                close_spatial_overlay(&mut state, &mut navigation)
            }
            InputEvent::Confirm => activate(&mut lifecycle, &mut state, &mut navigation),
            InputEvent::Cancel => cancel(&mut state, &mut navigation),
            _ => {}
        }
    }
}

fn close_spatial_overlay(state: &mut BevyInteractionState, navigation: &mut NavigationState) {
    if !state.spatial_overlay {
        return;
    }
    state.spatial_overlay_closing = true;
    // Movement remains suspended until the outgoing scene has finished.
    state.dirty = false;
    navigation.selected = 0;
}

fn begin_spatial_overlay_exit(
    state: Res<BevyInteractionState>,
    mut roots: Query<
        (&mut PresentationTransition, &UiTransform),
        With<SpatialInteractionOverlayRoot>,
    >,
) {
    if !state.spatial_overlay_closing {
        return;
    }
    for (mut transition, transform) in &mut roots {
        transition.begin_exit(transform.scale.x);
    }
}

fn finish_spatial_overlay_exit(
    mut state: ResMut<BevyInteractionState>,
    roots: Query<&PresentationTransition, With<SpatialInteractionOverlayRoot>>,
) {
    if !state.spatial_overlay_closing {
        return;
    }
    let root_missing = roots.iter().next().is_none();
    let finished = roots.iter().any(PresentationTransition::is_finished);
    if root_missing || finished {
        state.spatial_overlay = false;
        state.spatial_overlay_closing = false;
        state.selected = 0;
        state.npc_id = None;
        state.corpse_id = None;
        state.result = None;
        state.message = None;
        state.dirty = true;
    }
}

fn activate_spatial_overlay(
    lifecycle: &mut LifecycleState,
    state: &mut BevyInteractionState,
    navigation: &mut NavigationState,
) {
    if state.message.is_some() {
        state.message = None;
        state.selected = 0;
        navigation.selected = 0;
        state.dirty = true;
        return;
    }
    let Some(session) = lifecycle.session.as_mut() else {
        close_spatial_overlay(state, navigation);
        return;
    };

    match state.screen {
        InteractionScreen::Conversation => {
            let Some(npc_id) = state.npc_id else {
                close_spatial_overlay(state, navigation);
                return;
            };
            let available_choice = interactions::npc_index_by_id(&session.state, npc_id)
                .and_then(|npc_index| {
                    interactions::build_conversation_view_for_bevy(&session.state, npc_index)
                })
                .is_some_and(|view| view.available && state.selected < view.options.len());
            if !available_choice {
                close_spatial_overlay(state, navigation);
                return;
            }
            let messages = interactions::perform_conversation_choice(
                &mut session.state,
                npc_id,
                state.selected,
            );
            state.message = Some(messages.join("\n"));
            state.selected = 0;
            navigation.selected = 0;
            state.dirty = true;
        }
        InteractionScreen::Remains => {
            if state.selected > 0 {
                close_spatial_overlay(state, navigation);
                return;
            }
            let Some(corpse_id) = state.corpse_id else {
                close_spatial_overlay(state, navigation);
                return;
            };
            let view = legacy::build_remains_view_for_bevy(&session.state);
            let Some(selection) = view
                .remains
                .iter()
                .filter(|remains| !remains.items.is_empty())
                .position(|remains| remains.id == corpse_id)
            else {
                state.message = Some("These remains have already been searched.".to_string());
                state.dirty = true;
                return;
            };
            match legacy::search_remains_for_bevy(&mut session.state, selection) {
                Ok(result) => {
                    state.result = Some(result);
                    state.screen = InteractionScreen::RemainsResult;
                    state.selected = 0;
                    navigation.selected = 0;
                    state.dirty = true;
                }
                Err(error) => {
                    state.message = Some(error);
                    state.dirty = true;
                }
            }
        }
        InteractionScreen::RemainsResult | InteractionScreen::Talk => {
            close_spatial_overlay(state, navigation)
        }
    }
}

fn activate(
    lifecycle: &mut LifecycleState,
    state: &mut BevyInteractionState,
    navigation: &mut NavigationState,
) {
    let Some(session) = lifecycle.session.as_mut() else {
        return;
    };

    match state.screen {
        InteractionScreen::Talk => {
            let view = interactions::build_talk_view_for_bevy(&session.state);
            if state.selected >= view.npcs.len() {
                close_to_gameplay(state, navigation);
                return;
            }
            let Some(npc_id) = interactions::npc_ids_at_location(
                &session.state,
                session.state.character.location_id,
            )
            .get(state.selected)
            .copied() else {
                state.message = Some("There is no one here to talk to.".to_string());
                state.dirty = true;
                return;
            };
            state.npc_id = Some(npc_id);
            state.screen = InteractionScreen::Conversation;
            state.selected = 0;
            state.message = None;
            navigation.current_screen = Some(ScreenId::Conversation);
            navigation.selected = 0;
            state.dirty = true;
        }
        InteractionScreen::Conversation => {
            if state.message.is_some() {
                state.message = None;
                state.selected = 0;
                navigation.selected = 0;
                state.dirty = true;
                return;
            }
            let Some(npc_id) = state.npc_id else {
                state.screen = InteractionScreen::Talk;
                navigation.current_screen = Some(ScreenId::Talk);
                state.selected = 0;
                state.dirty = true;
                return;
            };
            let option_count = interactions::npc_index_by_id(&session.state, npc_id)
                .and_then(|npc_index| {
                    interactions::build_conversation_view_for_bevy(&session.state, npc_index)
                        .map(|view| view.options.len())
                })
                .unwrap_or(0);
            if state.selected >= option_count {
                state.screen = InteractionScreen::Talk;
                state.selected = 0;
                navigation.current_screen = Some(ScreenId::Talk);
                navigation.selected = 0;
                state.dirty = true;
                return;
            }
            let messages = interactions::perform_conversation_choice(
                &mut session.state,
                npc_id,
                state.selected,
            );
            state.message = Some(messages.join("\n"));
            state.selected = 0;
            navigation.selected = 0;
            state.dirty = true;
        }
        InteractionScreen::Remains => {
            let view = legacy::build_remains_view_for_bevy(&session.state);
            if state.selected >= view.remains.len() {
                close_to_gameplay(state, navigation);
                return;
            }
            match legacy::search_remains_for_bevy(&mut session.state, state.selected) {
                Ok(result) => {
                    state.result = Some(result);
                    state.screen = InteractionScreen::RemainsResult;
                    state.selected = 0;
                    navigation.current_screen = Some(ScreenId::RemainsResult);
                    navigation.selected = 0;
                    state.dirty = true;
                }
                Err(error) => {
                    state.message = Some(error);
                    state.dirty = true;
                }
            }
        }
        InteractionScreen::RemainsResult => close_to_gameplay(state, navigation),
    }
}

fn cancel(state: &mut BevyInteractionState, navigation: &mut NavigationState) {
    match state.screen {
        InteractionScreen::Talk | InteractionScreen::Remains => {
            close_to_gameplay(state, navigation)
        }
        InteractionScreen::Conversation => {
            state.screen = InteractionScreen::Talk;
            state.selected = 0;
            state.message = None;
            navigation.current_screen = Some(ScreenId::Talk);
            navigation.selected = 0;
            state.dirty = true;
        }
        InteractionScreen::RemainsResult => close_to_gameplay(state, navigation),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut state: ResMut<BevyInteractionState>,
    mut images: ResMut<Assets<Image>>,
    mut portrait_images: ResMut<PortraitImageCache>,
    roots: Query<Entity, With<BevyScreenRoot>>,
    spatial_roots: Query<Entity, With<SpatialSceneRoot>>,
    overlay_roots: Query<Entity, With<SpatialInteractionOverlayRoot>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete {
        return;
    }
    if navigation.current_screen == Some(ScreenId::SpatialExploration) {
        if state.spatial_overlay_closing {
            state.dirty = false;
            return;
        }
        if !state.dirty {
            return;
        }
        for root in &overlay_roots {
            commands.entity(root).despawn();
        }
        if state.spatial_overlay {
            let Some(parent) = spatial_roots.iter().next() else {
                return;
            };
            let Some(session) = lifecycle.session.as_ref() else {
                state.spatial_overlay = false;
                state.spatial_overlay_closing = false;
                state.dirty = false;
                return;
            };
            render_spatial_interaction_overlay(
                &mut commands,
                parent,
                &session.state,
                &state,
                &mut images,
                &mut portrait_images,
            );
        }
        state.dirty = false;
        return;
    }
    if !is_interaction_screen(navigation.current_screen) || !state.dirty {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };

    match state.screen {
        InteractionScreen::Talk => {
            let view = interactions::build_talk_view_for_bevy(&session.state);
            render_talk(&mut commands, &view, state.selected);
        }
        InteractionScreen::Conversation => {
            let view = state
                .npc_id
                .and_then(|npc_id| interactions::npc_index_by_id(&session.state, npc_id))
                .and_then(|npc_index| {
                    interactions::build_conversation_view_for_bevy(&session.state, npc_index)
                });
            render_conversation(
                &mut commands,
                view.as_ref(),
                state.selected,
                state.message.as_deref(),
                &mut images,
                &mut portrait_images,
            );
        }
        InteractionScreen::Remains => {
            let view = legacy::build_remains_view_for_bevy(&session.state);
            render_remains(
                &mut commands,
                &view,
                state.selected,
                state.message.as_deref(),
            );
        }
        InteractionScreen::RemainsResult => {
            render_remains_result(&mut commands, state.result.as_ref())
        }
    }
    state.dirty = false;
}

#[allow(clippy::too_many_arguments)]
fn render_spatial_interaction_overlay(
    commands: &mut Commands,
    parent: Entity,
    game: &crate::model::GameState,
    state: &BevyInteractionState,
    images: &mut Assets<Image>,
    portrait_images: &mut PortraitImageCache,
) {
    let is_dialogue = state.screen == InteractionScreen::Conversation;
    let overlay = commands
        .spawn((
            SpatialInteractionOverlayRoot,
            PresentationTransition::entering(),
            UiTransform {
                scale: Vec2::splat(0.96),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                width: percent(100),
                height: percent(100),
                padding: if is_dialogue {
                    UiRect::all(px(12))
                } else {
                    UiRect::all(px(0))
                },
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(if is_dialogue {
                bevy_presentation::THEME_BACKGROUND
            } else {
                Color::NONE
            }),
        ))
        .id();
    commands.entity(parent).add_child(overlay);

    match state.screen {
        InteractionScreen::Conversation => {
            let view = state
                .npc_id
                .and_then(|npc_id| interactions::npc_index_by_id(game, npc_id))
                .and_then(|npc_index| {
                    interactions::build_conversation_view_for_bevy(game, npc_index)
                });
            render_dialogue_scene_content(
                commands,
                overlay,
                view.as_ref(),
                state.selected,
                state.message.as_deref(),
                images,
                portrait_images,
            );
        }
        InteractionScreen::Remains | InteractionScreen::RemainsResult | InteractionScreen::Talk => {
            let panel = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(14),
                        bottom: px(14),
                        width: percent(44),
                        min_width: px(240),
                        max_width: px(560),
                        max_height: percent(76),
                        min_height: px(150),
                        padding: UiRect::all(px(12)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        border: UiRect::all(px(1)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BorderColor::all(bevy_presentation::THEME_BORDER),
                    BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
                ))
                .id();
            commands.entity(overlay).add_child(panel);
            let content = bevy_presentation::spawn_scrollable_list(commands, panel);
            let footer = bevy_presentation::spawn_option_footer(commands, panel);
            match state.screen {
                InteractionScreen::Remains => {
                    let view = legacy::build_remains_view_for_bevy(game);
                    render_spatial_remains_content(
                        commands,
                        content,
                        footer,
                        &view,
                        state.corpse_id,
                        state.selected,
                        state.message.as_deref(),
                    );
                }
                InteractionScreen::RemainsResult => {
                    render_remains_result_content(commands, content, footer, state.result.as_ref())
                }
                InteractionScreen::Talk => {
                    bevy_presentation::spawn_muted_label(
                        commands,
                        content,
                        "There is no conversation to continue.",
                    );
                    let button = bevy_presentation::spawn_action_button(
                        commands, footer, 0, "back", "Close",
                    );
                    commands
                        .entity(button)
                        .insert(bevy_presentation::UiSelected);
                }
                InteractionScreen::Conversation => {}
            }
        }
    }
}

fn ordered_portrait_layers(portrait: &CharacterPortraitView) -> Vec<&CharacterPortraitLayerView> {
    let mut layers = portrait.layers.iter().collect::<Vec<_>>();
    layers.sort_by(|left, right| {
        left.layer
            .cmp(&right.layer)
            .then_with(|| left.slot.cmp(&right.slot))
            .then_with(|| left.id.cmp(&right.id))
    });
    layers
}

fn load_portrait_layer_image(
    images: &mut Assets<Image>,
    cache: &mut PortraitImageCache,
    layer: &CharacterPortraitLayerView,
) -> Option<Handle<Image>> {
    if let Some(image) = cache.images.get(&layer.asset) {
        return Some(image.clone());
    }
    match crate::asset_loading::load_image_from_data(images, &layer.asset) {
        Ok(image) => {
            cache.images.insert(layer.asset.clone(), image.clone());
            Some(image)
        }
        Err(error) => {
            if cache.missing.insert(layer.asset.clone()) {
                warn!(
                    "Could not load portrait layer {} ({}): {}",
                    layer.id, layer.asset, error
                );
                cache.failures.insert(layer.asset.clone(), error);
            }
            None
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_character_portrait(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    detail: Option<&str>,
    portrait: &CharacterPortraitView,
    fallback: Option<&str>,
    images: &mut Assets<Image>,
    cache: &mut PortraitImageCache,
) {
    let panel = commands
        .spawn((
            Node {
                width: percent(50),
                height: percent(100),
                min_width: px(0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                padding: UiRect::all(px(6)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(4),
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_SURFACE),
        ))
        .id();
    commands.entity(parent).add_child(panel);
    bevy_presentation::spawn_label(commands, panel, name);
    if let Some(detail) = detail.filter(|value| !value.trim().is_empty()) {
        bevy_presentation::spawn_muted_label(commands, panel, detail);
    }

    let canvas = commands
        .spawn((
            Node {
                width: Val::Auto,
                height: percent(78),
                max_width: percent(100),
                max_height: px(320),
                aspect_ratio: Some(2.0 / 3.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                flex_shrink: 1.0,
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_PANEL),
            BorderColor::all(bevy_presentation::THEME_BORDER),
        ))
        .id();
    commands.entity(panel).add_child(canvas);

    let mut rendered = 0usize;
    for layer in ordered_portrait_layers(portrait) {
        let Some(image) = load_portrait_layer_image(images, cache, layer) else {
            continue;
        };
        let node = commands
            .spawn((
                ImageNode::new(image),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
            ))
            .id();
        commands.entity(canvas).add_child(node);
        rendered += 1;
    }
    if rendered == 0 {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            fallback.unwrap_or("Portrait unavailable"),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_dialogue_scene_content(
    commands: &mut Commands,
    parent: Entity,
    view: Option<&ConversationView>,
    selected: usize,
    message: Option<&str>,
    images: &mut Assets<Image>,
    portrait_images: &mut PortraitImageCache,
) {
    let header = commands
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        })
        .id();
    commands.entity(parent).add_child(header);
    bevy_presentation::spawn_label(commands, header, "CONVERSATION");
    bevy_presentation::spawn_muted_label(commands, header, "THE EVER-UNFOLDING SAGA");

    let stage = commands
        .spawn(Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        })
        .id();
    commands.entity(parent).add_child(stage);

    if let Some(view) = view {
        let player_detail = format!("Health: {}/{}", view.player.hp, view.player.max_hp);
        render_character_portrait(
            commands,
            stage,
            &view.player.display_name(),
            Some(&player_detail),
            &view.player_portrait,
            None,
            images,
            portrait_images,
        );
        render_character_portrait(
            commands,
            stage,
            &view.npc.display_name(),
            view.npc.faction_name.as_deref(),
            &view.npc_portrait,
            view.portrait.as_deref(),
            images,
            portrait_images,
        );
    } else {
        bevy_presentation::spawn_muted_label(
            commands,
            stage,
            "That person can no longer be found.",
        );
    }

    let dialogue = commands
        .spawn((
            Node {
                width: percent(100),
                min_height: px(112),
                max_height: percent(46),
                padding: UiRect::all(px(10)),
                border: UiRect::all(px(1)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(bevy_presentation::THEME_BORDER),
            BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
        ))
        .id();
    commands.entity(parent).add_child(dialogue);

    let Some(view) = view else {
        bevy_presentation::spawn_muted_label(
            commands,
            dialogue,
            "The conversation can no longer continue.",
        );
        let actions = commands
            .spawn(Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8),
                row_gap: px(6),
                ..default()
            })
            .id();
        commands.entity(dialogue).add_child(actions);
        let button = bevy_presentation::spawn_compact_action_button(commands, actions, 0, "Leave");
        if selected == 0 {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
        return;
    };

    bevy_presentation::spawn_label(commands, dialogue, view.npc.display_name());
    if let Some(memory) = &view.memory {
        bevy_presentation::spawn_muted_label(commands, dialogue, format!("Memory: {memory}"));
    }
    if let Some(message) = message {
        for line in message.lines() {
            bevy_presentation::spawn_muted_label(commands, dialogue, line);
        }
    } else if !view.available {
        bevy_presentation::spawn_muted_label(
            commands,
            dialogue,
            view.unavailable_message
                .as_deref()
                .unwrap_or("They are unavailable right now."),
        );
    } else {
        bevy_presentation::spawn_muted_label(commands, dialogue, "Choose your response.");
    }

    let actions = commands
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(6),
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    commands.entity(dialogue).add_child(actions);

    if message.is_some() {
        let button =
            bevy_presentation::spawn_compact_action_button(commands, actions, 0, "Continue");
        if selected == 0 {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    } else if view.available {
        for (index, option) in view.options.iter().enumerate() {
            let button =
                bevy_presentation::spawn_compact_action_button(commands, actions, index, option);
            if selected == index {
                commands
                    .entity(button)
                    .insert(bevy_presentation::UiSelected);
            }
        }
        let leave_index = view.options.len();
        let button =
            bevy_presentation::spawn_compact_action_button(commands, actions, leave_index, "Leave");
        if selected == leave_index {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    } else {
        let button = bevy_presentation::spawn_compact_action_button(commands, actions, 0, "Leave");
        if selected == 0 {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }
}

fn render_spatial_remains_content(
    commands: &mut Commands,
    parent: Entity,
    footer: Entity,
    view: &RemainsView,
    corpse_id: Option<EntityId>,
    selected: usize,
    message: Option<&str>,
) {
    let remains = corpse_id.and_then(|id| {
        view.remains
            .iter()
            .find(|remains| remains.id == id && !remains.items.is_empty())
    });
    let Some(remains) = remains else {
        bevy_presentation::spawn_muted_label(commands, parent, "These remains are no longer here.");
        let button = bevy_presentation::spawn_action_button(commands, footer, 0, "back", "Leave");
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
        return;
    };

    bevy_presentation::spawn_label(commands, parent, remains.label.clone());
    bevy_presentation::spawn_muted_label(commands, parent, format!("At {}", view.location_name));
    if remains.scavenged {
        bevy_presentation::spawn_muted_label(
            commands,
            parent,
            "Someone has already picked through these remains.",
        );
    }
    for item in &remains.items {
        bevy_presentation::spawn_label(commands, parent, item.name.clone());
        if !item.description.is_empty() {
            bevy_presentation::spawn_muted_label(commands, parent, item.description.clone());
        }
    }
    if let Some(message) = message {
        bevy_presentation::spawn_muted_label(commands, parent, message);
        let button =
            bevy_presentation::spawn_action_button(commands, footer, 0, "continue", "Continue");
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
    } else {
        let search = bevy_presentation::spawn_action_button(
            commands,
            footer,
            0,
            "remains",
            "Search remains",
        );
        if selected == 0 {
            commands
                .entity(search)
                .insert(bevy_presentation::UiSelected);
        }
        let leave = bevy_presentation::spawn_action_button(commands, footer, 1, "back", "Leave");
        if selected == 1 {
            commands.entity(leave).insert(bevy_presentation::UiSelected);
        }
    }
}

fn render_remains_result_content(
    commands: &mut Commands,
    parent: Entity,
    footer: Entity,
    result: Option<&RemainsResultView>,
) {
    let Some(result) = result else {
        bevy_presentation::spawn_muted_label(commands, parent, "Nothing was recovered.");
        let button =
            bevy_presentation::spawn_action_button(commands, footer, 0, "continue", "Continue");
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
        return;
    };
    bevy_presentation::spawn_label(
        commands,
        parent,
        format!(
            "{} the {} at {}",
            result.former_name, result.former_title, result.location_name
        ),
    );
    for item in &result.items {
        bevy_presentation::spawn_label(commands, parent, format!("Recovered: {}", item.name));
        if !item.description.is_empty() {
            bevy_presentation::spawn_muted_label(commands, parent, item.description.clone());
        }
    }
    if let Some(item) = &result.hidden_item {
        bevy_presentation::spawn_label(commands, parent, format!("Discovered: {}", item.name));
        bevy_presentation::spawn_muted_label(commands, parent, item.description.clone());
    }
    for note in &result.notes {
        bevy_presentation::spawn_muted_label(commands, parent, note.clone());
    }
    let button =
        bevy_presentation::spawn_action_button(commands, footer, 0, "continue", "Continue");
    commands
        .entity(button)
        .insert(bevy_presentation::UiSelected);
}

fn render_talk(commands: &mut Commands, view: &TalkView, selected: usize) {
    let root = bevy_presentation::spawn_screen(commands, "TALK");
    let panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(commands, panel, "Choose someone to speak with.");
    if view.npcs.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "There is no one here to talk to.");
    } else {
        for (index, npc) in view.npcs.iter().enumerate() {
            let button = bevy_presentation::spawn_action_button(
                commands,
                panel,
                index,
                "character",
                npc.display_name(),
            );
            if index == selected {
                commands
                    .entity(button)
                    .insert(bevy_presentation::UiSelected);
            }
        }
    }
    let back_index = view.npcs.len();
    let button =
        bevy_presentation::spawn_action_button(commands, panel, back_index, "back", "Back");
    if selected == back_index {
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
    }
}

fn render_conversation(
    commands: &mut Commands,
    view: Option<&ConversationView>,
    selected: usize,
    message: Option<&str>,
    images: &mut Assets<Image>,
    portrait_images: &mut PortraitImageCache,
) {
    let root = commands
        .spawn((
            BevyScreenRoot,
            PresentationTransition::entering(),
            UiTransform {
                scale: Vec2::splat(0.96),
                ..default()
            },
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_BACKGROUND),
        ))
        .id();
    render_dialogue_scene_content(
        commands,
        root,
        view,
        selected,
        message,
        images,
        portrait_images,
    );
}

fn render_remains(
    commands: &mut Commands,
    view: &RemainsView,
    selected: usize,
    message: Option<&str>,
) {
    let root = bevy_presentation::spawn_screen(commands, "REMAINS");
    let panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        format!("Search the remains at {}.", view.location_name),
    );
    if let Some(message) = message {
        bevy_presentation::spawn_muted_label(commands, panel, message);
    }
    if view.remains.is_empty() {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "There are no remains worth searching here.",
        );
    } else {
        for (index, remains) in view.remains.iter().enumerate() {
            let button = bevy_presentation::spawn_action_button(
                commands,
                panel,
                index,
                "remains",
                remains.label.clone(),
            );
            if index == selected {
                commands
                    .entity(button)
                    .insert(bevy_presentation::UiSelected);
            }
        }
    }
    let back_index = view.remains.len();
    let button =
        bevy_presentation::spawn_action_button(commands, panel, back_index, "back", "Back");
    if selected == back_index {
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
    }
}

fn render_remains_result(commands: &mut Commands, result: Option<&RemainsResultView>) {
    let root = bevy_presentation::spawn_screen(commands, "REMAINS RECOVERED");
    let panel = bevy_presentation::spawn_panel(commands, root);
    let Some(result) = result else {
        bevy_presentation::spawn_muted_label(commands, panel, "Nothing was recovered.");
        let button = bevy_presentation::spawn_action_button(commands, panel, 0, "back", "Back");
        commands
            .entity(button)
            .insert(bevy_presentation::UiSelected);
        return;
    };
    bevy_presentation::spawn_label(
        commands,
        panel,
        format!(
            "{} the {} at {}",
            result.former_name, result.former_title, result.location_name
        ),
    );
    for item in &result.items {
        bevy_presentation::spawn_label(commands, panel, format!("Recovered: {}", item.name));
        if !item.description.is_empty() {
            bevy_presentation::spawn_muted_label(commands, panel, item.description.clone());
        }
    }
    if let Some(item) = &result.hidden_item {
        bevy_presentation::spawn_label(commands, panel, format!("Discovered: {}", item.name));
        bevy_presentation::spawn_muted_label(commands, panel, item.description.clone());
    }
    for note in &result.notes {
        bevy_presentation::spawn_muted_label(commands, panel, note.clone());
    }
    let button = bevy_presentation::spawn_action_button(commands, panel, 0, "continue", "Continue");
    commands
        .entity(button)
        .insert(bevy_presentation::UiSelected);
}

#[cfg(test)]
#[path = "../tests/unit/bevy_interactions.rs"]
mod tests;
