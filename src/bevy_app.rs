//! Bevy application bootstrap for the graphical frontend.
//!
//! Engine setup stays here; lifecycle and gameplay behavior remain in their
//! frontend-independent systems and Bevy adapters.

#[cfg(not(target_os = "android"))]
use crate::desktop_window;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::prelude::*;
use bevy_picking::events::PointerRelease;

use crate::{
    bevy_combat, bevy_console, bevy_dev_tools, bevy_feedback, bevy_gameplay, bevy_interactions,
    bevy_lifecycle, bevy_navigation_bridge, bevy_options, bevy_presentation, bevy_records,
    bevy_runtime, bevy_spatial, bevy_touch, ui_settings,
};

const WINDOW_TITLE: &str = "The Ever-Unfolding Saga";

pub fn run() {
    let window_resolution = (1280, 720);

    let mut app = App::new();
    // EditableText reads PointerRelease even when no input-picking backend is
    // available. Register its message storage explicitly; touch gestures are
    // resolved by our adapter and suppressed before widget activation.
    app.add_message::<PointerRelease>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE.to_owned(),
            resolution: window_resolution.into(),
            #[cfg(not(target_os = "android"))]
            mode: desktop_window::fullscreen_mode(),
            resizable: false,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(TabNavigationPlugin);
    bevy_presentation::install(&mut app);
    ui_settings::install(&mut app);
    bevy_touch::install(&mut app);
    bevy_runtime::install(&mut app);
    bevy_lifecycle::install(&mut app);
    bevy_gameplay::install(&mut app);
    bevy_spatial::install(&mut app);
    bevy_combat::install(&mut app);
    bevy_console::install(&mut app);
    bevy_navigation_bridge::install(&mut app);
    bevy_options::install(&mut app);
    bevy_records::install(&mut app);
    bevy_interactions::install(&mut app);
    bevy_dev_tools::install(&mut app);
    bevy_feedback::install(&mut app);
    #[cfg(not(target_os = "android"))]
    app.add_systems(Update, desktop_window::apply_window_icon);
    app.add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
