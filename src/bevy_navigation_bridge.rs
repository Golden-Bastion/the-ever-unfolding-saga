//! Bevy navigation glue for lifecycle completion and gameplay screen hand-offs.
//!
//! Dedicated graphical screen transitions are owned by their originating
//! gameplay systems; this bridge only handles cross-system lifecycle completion
//! and the shared gameplay re-entry selection reset.

use bevy::prelude::*;

use crate::bevy_gameplay::{GameplayScreen, GameplayState};
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{GameplayInputQueue, NavigationState, ScreenId};
use crate::bevy_spatial::{self, SpatialExplorationState};
use crate::input::InputEvent;

#[derive(Resource, Default)]
pub(crate) struct NavigationBridgeState {
    last_screen: Option<ScreenId>,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<NavigationBridgeState>()
        .add_systems(Update, bridge_navigation);
}

fn bridge_navigation(
    lifecycle: Res<LifecycleState>,
    gameplay: Res<GameplayState>,
    mut navigation: ResMut<NavigationState>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
    mut spatial_state: ResMut<SpatialExplorationState>,
    mut bridge: ResMut<NavigationBridgeState>,
) {
    if lifecycle.phase != LifecyclePhase::Complete || lifecycle.session.is_none() {
        bridge.last_screen = navigation.current_screen;
        return;
    }

    if should_enter_spatial_exploration(
        navigation.current_screen,
        lifecycle.session.is_some(),
        lifecycle.pending_state.is_some(),
        navigation.return_screen,
    ) {
        if let Some(session) = lifecycle.session.as_ref() {
            spatial::enter(&mut spatial_state, &session.state, &mut navigation);
        }
    }

    if navigation.current_screen != bridge.last_screen {
        if should_reset_gameplay_selection(
            bridge.last_screen,
            navigation.current_screen,
            gameplay.screen,
        ) {
            gameplay_queue.0.push(InputEvent::Home);
        }
        bridge.last_screen = navigation.current_screen;
    }
}

fn should_enter_spatial_exploration(
    current_screen: Option<ScreenId>,
    has_session: bool,
    has_pending_state: bool,
    return_screen: Option<ScreenId>,
) -> bool {
    current_screen == Some(ScreenId::Lifecycle)
        && has_session
        && !has_pending_state
        && return_screen.is_none()
}

fn should_reset_gameplay_selection(
    previous_screen: Option<ScreenId>,
    current_screen: Option<ScreenId>,
    gameplay_screen: GameplayScreen,
) -> bool {
    current_screen == Some(ScreenId::Gameplay)
        && previous_screen != current_screen
        && gameplay_screen != GameplayScreen::Pause
}
#[cfg(test)]
#[path = "../tests/unit/bevy_navigation_bridge.rs"]
mod tests;
