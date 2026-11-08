mod asset_loading;
mod bevy_app;
mod bevy_combat;
mod bevy_console;
mod bevy_dev_tools;
mod bevy_feedback;
mod bevy_gameplay;
mod bevy_interactions;
mod bevy_lifecycle;
mod bevy_navigation_bridge;
mod bevy_options;
mod bevy_presentation;
mod bevy_records;
mod bevy_runtime;
mod bevy_spatial;
mod bevy_touch;
mod character_appearance;
mod content;
#[cfg(not(target_os = "android"))]
mod desktop_window;
mod events;
mod game;
mod game_paths;
mod input;
mod interaction_actions;
mod mod_manager;
mod model;
mod persistence;
mod presentation;
pub mod procedural;
pub mod procedural_authored;
pub mod procedural_characteristics;
pub mod procedural_entities;
pub mod procedural_opportunities;
pub mod procedural_relationships;
pub mod rng;
pub mod spatial;
mod spatial_animation;
mod spatial_fixtures;
mod ui_settings;

pub fn run() {
    if let Err(error) = game_paths::GamePaths::initialize() {
        panic!("failed to initialize The Ever-Unfolding Saga game storage: {error}");
    }
    mod_manager::initialize_active_mods();
    bevy_app::run();
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: bevy::android::android_activity::AndroidApp) {
    let _ = bevy::android::ANDROID_APP.set(android_app);
    run();
}
