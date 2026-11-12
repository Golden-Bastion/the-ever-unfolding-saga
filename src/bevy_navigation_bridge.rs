//! Bevy navigation glue for lifecycle completion and spatial exploration hand-off.

use bevy::prelude::*;

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{NavigationState, ScreenId};
use crate::bevy_spatial::SpatialExplorationState;

pub(crate) fn install(app: &mut App) {
    app.add_systems(Update, bridge_navigation);
}

fn bridge_navigation(
    lifecycle: Res<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut spatial_state: ResMut<SpatialExplorationState>,
) {
    if lifecycle.phase != LifecyclePhase::Complete || lifecycle.session.is_none() {
        return;
    }

    if should_enter_spatial_exploration(
        navigation.current_screen,
        lifecycle.session.is_some(),
        lifecycle.pending_state.is_some(),
        navigation.return_screen,
    ) {
        if let Some(session) = lifecycle.session.as_ref() {
            crate::bevy_spatial::enter(&mut spatial_state, &session.state, &mut navigation);
        }
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

#[cfg(test)]
#[path = "../tests/unit/bevy_navigation_bridge.rs"]
mod tests;
