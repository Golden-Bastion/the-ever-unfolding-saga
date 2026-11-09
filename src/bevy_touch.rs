//! Touch gesture handling for graphical UI controls.
//!
//! Normal buttons use Bevy's completed-click activation. This module retains the
//! gameplay menu's custom tap/long-press behavior for opening secondary navigation
//! or the developer console; scroll-panel gesture tracking lives in presentation.

use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::window::PrimaryWindow;

use crate::bevy_presentation::{
    menu_button_screen_active, physical_touch_position, GameplayInputQueue, NavigationState,
    UiMenuButton,
};
const TOUCH_TAP_THRESHOLD: f32 = 12.0;
const TOUCH_MENU_LONG_PRESS_SECS: f32 = 0.65;

#[derive(Resource, Default)]
struct TouchMenuState {
    active: Option<ActiveMenuTouch>,
}

#[derive(Clone, Copy)]
struct ActiveMenuTouch {
    id: u64,
    start: Vec2,
    button: Entity,
    started_at: f32,
    dragging: bool,
}

pub fn install(app: &mut App) {
    app.init_resource::<TouchMenuState>()
        .add_systems(
            PreUpdate,
            suppress_touch_menu_press.after(bevy::ui::UiSystems::Focus),
        )
        .add_systems(Update, complete_touch_menu);
}

fn suppress_touch_menu_press(
    mut commands: Commands,
    touches: Res<Touches>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchMenuState>,
    buttons: Query<
        (Entity, Option<&Pressed>, &ComputedNode, &UiGlobalTransform),
        With<UiMenuButton>,
    >,
) {
    for touch in touches.iter_just_pressed() {
        let physical_position = physical_touch_position(touch.position(), window.scale_factor());
        let Some((entity, _, _, _)) = buttons.iter().find(|(_, _, computed, transform)| {
            computed.contains_point(**transform, physical_position)
        }) else {
            continue;
        };

        state.active = Some(ActiveMenuTouch {
            id: touch.id(),
            start: touch.position(),
            button: entity,
            started_at: time.elapsed_secs(),
            dragging: false,
        });
    }

    let Some(active) = state.active else {
        return;
    };

    let Some(touch) = touches.iter().find(|touch| touch.id() == active.id) else {
        if touches.just_canceled(active.id) {
            state.active = None;
        }
        return;
    };

    if touch.position().distance(active.start) > TOUCH_TAP_THRESHOLD && !active.dragging {
        state.active = Some(ActiveMenuTouch {
            dragging: true,
            ..active
        });
    }

    if let Ok((_, Some(_), _, _)) = buttons.get(active.button) {
        // Menus retain custom tap and long-press handling.
        commands.entity(active.button).remove::<Pressed>();
    }
}

// A normal tap on the Gameplay or spatial pause button opens the pause menu;
// holding it for 650 ms without dragging opens the developer console on touchscreens.
fn should_open_developer_console(elapsed_secs: f32, started_at: f32, dragging: bool) -> bool {
    !dragging && elapsed_secs - started_at >= TOUCH_MENU_LONG_PRESS_SECS
}

fn complete_touch_menu(
    touches: Res<Touches>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchMenuState>,
    navigation: Res<NavigationState>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
    buttons: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<UiMenuButton>>,
) {
    let Some(active) = state.active else {
        return;
    };

    if !menu_button_screen_active(navigation.current_screen) {
        state.active = None;
        return;
    }

    if should_open_developer_console(time.elapsed_secs(), active.started_at, active.dragging) {
        state.active = None;
        gameplay_queue
            .0
            .push(crate::input::InputEvent::OpenDeveloperConsole);
        return;
    }

    let Some(released_touch) = touches
        .iter_just_released()
        .find(|touch| touch.id() == active.id)
    else {
        if touches.just_canceled(active.id) {
            state.active = None;
        }
        return;
    };

    state.active = None;
    if active.dragging {
        return;
    }

    let physical_release_position =
        physical_touch_position(released_touch.position(), window.scale_factor());
    let Some((_, computed, transform)) = buttons
        .iter()
        .find(|(entity, _, _)| *entity == active.button)
    else {
        return;
    };

    if computed.contains_point((**transform).into(), physical_release_position) {
        gameplay_queue
            .0
            .push(crate::input::InputEvent::OpenSecondaryNavigation);
    }
}

#[cfg(test)]
#[path = "../tests/unit/bevy_touch.rs"]
mod tests;
