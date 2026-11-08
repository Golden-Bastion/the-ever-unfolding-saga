//! Options-owned save import/export and mod management workflows.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::bevy_gameplay::GameplayState;
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId, UiSelected,
};
use crate::input::InputEvent;
use crate::mod_manager::InstalledMod;

const MOD_ACTIONS_PER_MOD: usize = 4;
const MOD_ENABLE_ACTION: usize = 0;
const MOD_DELETE_ACTION: usize = 1;
const MOD_PRIORITY_UP_ACTION: usize = 2;
const MOD_PRIORITY_DOWN_ACTION: usize = 3;

struct SaveEntry {
    path: PathBuf,
    metadata: Option<crate::persistence::SaveMetadata>,
}

fn mod_action_start_index(mod_index: usize) -> usize {
    mod_index * MOD_ACTIONS_PER_MOD
}

fn mod_install_action_index(mod_count: usize) -> usize {
    mod_count * MOD_ACTIONS_PER_MOD
}

fn mod_back_action_index(mod_count: usize) -> usize {
    mod_install_action_index(mod_count) + 1
}

#[derive(Component)]
struct OptionsScreen;

#[derive(Component)]
struct OptionsContent;

#[derive(Resource)]
pub(crate) struct OptionsState {
    selected: usize,
    tab: usize,
    message: Option<String>,
    active: bool,
    dirty: bool,
    saves: Vec<SaveEntry>,
    mods: Vec<InstalledMod>,
    pending_restart: bool,
    pending_remove_id: Option<String>,
    #[cfg(target_os = "android")]
    last_storage_poll: f32,
    #[cfg(target_os = "android")]
    processed_imports: std::collections::HashSet<PathBuf>,
}

impl Default for OptionsState {
    fn default() -> Self {
        Self {
            selected: 0,
            tab: 0,
            message: None,
            active: false,
            dirty: true,
            saves: Vec::new(),
            mods: Vec::new(),
            pending_restart: false,
            pending_remove_id: None,
            #[cfg(target_os = "android")]
            last_storage_poll: 0.0,
            #[cfg(target_os = "android")]
            processed_imports: std::collections::HashSet::new(),
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<OptionsState>().add_systems(
        Update,
        (
            sync_screen,
            refresh_storage,
            options_input,
            render_if_active,
        )
            .chain(),
    );
}

fn sync_screen(
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
) {
    let active = matches!(
        lifecycle.phase,
        LifecyclePhase::Start | LifecyclePhase::Complete
    ) && navigation.current_screen == Some(ScreenId::Options);

    if active && !options.active {
        options.active = true;
        options.selected = 0;
        options.tab = 0;
        options.message = None;
        options.pending_remove_id = None;
        refresh_catalogs(&mut options);
        #[cfg(target_os = "android")]
        {
            options.last_storage_poll = 0.0;
        }
        options.dirty = true;
    } else if !active {
        options.active = false;
    }
}

fn refresh_catalogs(options: &mut OptionsState) {
    let paths = crate::game_paths::GamePaths::get();
    let mut save_paths = crate::persistence::find_save_files(&paths.saves_dir).unwrap_or_default();
    let legacy = crate::persistence::legacy_save_path(&paths.root);
    if legacy.is_file() && !save_paths.iter().any(|path| path == &legacy) {
        save_paths.push(legacy);
    }
    options.saves = save_paths
        .into_iter()
        .map(|path| SaveEntry {
            metadata: crate::persistence::read_save_metadata(&path).ok(),
            path,
        })
        .collect();
    options.mods = match crate::mod_manager::list_installed_mods() {
        Ok(mods) => mods,
        Err(error) => {
            options.message = Some(format!("Could not read installed mods: {error}"));
            Vec::new()
        }
    };
    let maximum_selection = options_selection_end(options);
    options.selected = options.selected.min(maximum_selection);
}

#[cfg(target_os = "android")]
fn refresh_storage(
    time: Res<Time>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
    mut lifecycle: ResMut<LifecycleState>,
) {
    if navigation.current_screen != Some(ScreenId::Options) {
        return;
    }
    let elapsed = time.elapsed_secs();
    if elapsed - options.last_storage_poll < 0.5 {
        return;
    }
    options.last_storage_poll = elapsed;

    let paths = crate::game_paths::GamePaths::get();
    let mut changed = false;
    for (directory, kind) in [
        (&paths.save_imports_dir, "save"),
        (&paths.mod_imports_dir, "mod"),
    ] {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) => {
                options.message = Some(format!("Could not inspect imported files: {error}"));
                options.dirty = true;
                continue;
            }
        };
        for entry in entries.flatten() {
            let source = entry.path();
            if !source.is_file() || options.processed_imports.contains(&source) {
                continue;
            }
            let result = match kind {
                "save" if source.extension().and_then(|value| value.to_str()) == Some("save") => {
                    crate::persistence::install_imported_save(&source).map(|_| None)
                }
                "mod" if source.extension().and_then(|value| value.to_str()) == Some("zip") => {
                    crate::mod_manager::install_mod_archive(&source)
                        .map(|item| Some(item.manifest.name))
                }
                _ => continue,
            };
            options.processed_imports.insert(source.clone());
            match result {
                Ok(installed_mod_name) => {
                    let _ = std::fs::remove_file(&source);
                    options.message = if let Some(name) = installed_mod_name {
                        options.pending_restart = true;
                        Some(format!("Installed {name}."))
                    } else {
                        Some("Save imported successfully.".to_string())
                    };
                    changed = true;
                }
                Err(error) => {
                    options.message =
                        Some(format!("Could not import {}: {error}", source.display()));
                    options.dirty = true;
                }
            }
        }
    }
    if changed {
        refresh_catalogs(&mut options);
        lifecycle.refresh_saved_games();
        options.dirty = true;
    }
}

#[cfg(not(target_os = "android"))]
fn refresh_storage(
    _time: Res<Time>,
    _navigation: Res<NavigationState>,
    _options: ResMut<OptionsState>,
    _lifecycle: ResMut<LifecycleState>,
) {
}

fn options_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut options: ResMut<OptionsState>,
    mut ui_settings: ResMut<crate::ui_settings::UiSettings>,
    gameplay: Res<GameplayState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if !matches!(
        lifecycle.phase,
        LifecyclePhase::Start | LifecyclePhase::Complete
    ) || navigation.current_screen != Some(ScreenId::Options)
        || input_queue.0.is_empty()
    {
        return;
    }

    let events = std::mem::take(&mut input_queue.0);
    for event in events {
        match event {
            InputEvent::Up => {
                options.selected = options.selected.saturating_sub(1);
                options.pending_remove_id = None;
            }
            InputEvent::Down => {
                options.selected = (options.selected + 1).min(options_selection_end(&options));
                options.pending_remove_id = None;
            }
            InputEvent::Home => {
                options.selected = 0;
                options.pending_remove_id = None;
            }
            InputEvent::End => {
                options.selected = options_selection_end(&options);
                options.pending_remove_id = None;
            }
            InputEvent::SelectTab(tab) if tab < 3 => {
                options.tab = tab;
                options.selected = 0;
                options.message = None;
                options.pending_remove_id = None;
            }
            InputEvent::Tab => {
                options.tab = (options.tab + 1) % 3;
                options.selected = 0;
                options.message = None;
                options.pending_remove_id = None;
            }
            InputEvent::Confirm => match options.tab {
                0 => {
                    activate_saves_action(&mut options, &mut lifecycle, &mut navigation, &gameplay)
                }
                1 => activate_mods_action(&mut options, &mut lifecycle, &mut navigation, &gameplay),
                2 => activate_interface_action(
                    &mut options,
                    &mut ui_settings,
                    &mut lifecycle,
                    &mut navigation,
                    &gameplay,
                ),
                _ => {}
            },
            InputEvent::Cancel => {
                close_options(&mut lifecycle, &mut navigation, &mut options, &gameplay)
            }
            _ => {}
        }
        options.dirty = true;
    }
    navigation.selected = options.selected;
}

fn options_selection_end(options: &OptionsState) -> usize {
    match options.tab {
        0 => options.saves.len() + 1,
        1 => mod_back_action_index(options.mods.len()),
        2 => 5,
        _ => 0,
    }
}

fn activate_saves_action(
    options: &mut OptionsState,
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    gameplay: &GameplayState,
) {
    if options.selected < options.saves.len() {
        let path = options.saves[options.selected].path.clone();
        export_save(options, &path);
    } else if options.selected == options.saves.len() {
        import_save(options, lifecycle);
    } else {
        close_options(lifecycle, navigation, options, gameplay);
    }
}

fn activate_mods_action(
    options: &mut OptionsState,
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    gameplay: &GameplayState,
) {
    if options.selected == mod_install_action_index(options.mods.len()) {
        import_mod(options);
        return;
    }
    if options.selected == mod_back_action_index(options.mods.len()) {
        close_options(lifecycle, navigation, options, gameplay);
        return;
    }

    let mod_index = options.selected / MOD_ACTIONS_PER_MOD;
    let action = options.selected % MOD_ACTIONS_PER_MOD;
    let Some(item) = options.mods.get(mod_index).cloned() else {
        return;
    };

    let result = match action {
        MOD_ENABLE_ACTION => crate::mod_manager::set_mod_enabled(&item.manifest.id, !item.enabled),
        MOD_DELETE_ACTION => {
            if options.pending_remove_id.as_deref() != Some(item.manifest.id.as_str()) {
                options.pending_remove_id = Some(item.manifest.id.clone());
                options.message = Some(format!(
                    "Select Confirm again to permanently remove {}.",
                    item.manifest.name
                ));
                return;
            }
            options.pending_remove_id = None;
            crate::mod_manager::remove_mod(&item.manifest.id)
        }
        MOD_PRIORITY_UP_ACTION => {
            crate::mod_manager::set_mod_priority(&item.manifest.id, item.priority.saturating_add(1))
        }
        MOD_PRIORITY_DOWN_ACTION => {
            crate::mod_manager::set_mod_priority(&item.manifest.id, item.priority.saturating_sub(1))
        }
        _ => return,
    };

    let success_message = match action {
        MOD_DELETE_ACTION => format!("Removed {}.", item.manifest.name),
        MOD_PRIORITY_UP_ACTION | MOD_PRIORITY_DOWN_ACTION => {
            format!("Updated priority for {}.", item.manifest.name)
        }
        _ => format!("Updated {}.", item.manifest.name),
    };

    match result {
        Ok(()) => {
            options.pending_restart = true;
            options.message = Some(success_message);
            refresh_catalogs(options);
        }
        Err(error) => {
            options.message = Some(format!(
                "Could not update mod {}: {error}",
                item.manifest.id
            ))
        }
    }
}

fn activate_interface_action(
    options: &mut OptionsState,
    settings: &mut crate::ui_settings::UiSettings,
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    gameplay: &GameplayState,
) {
    let changed = match options.selected {
        0 => settings.adjust_ui_scale(-crate::ui_settings::SETTING_STEP),
        1 => settings.adjust_ui_scale(crate::ui_settings::SETTING_STEP),
        2 => settings.adjust_font_size(-crate::ui_settings::SETTING_STEP),
        3 => settings.adjust_font_size(crate::ui_settings::SETTING_STEP),
        4 => {
            settings.reset();
            true
        }
        5 => {
            close_options(lifecycle, navigation, options, gameplay);
            return;
        }
        _ => false,
    };

    if changed {
        match settings.save() {
            Ok(()) => {
                options.message = Some(format!(
                    "UI scale: {}%. Font size: {}%.",
                    settings.ui_scale_percent(),
                    settings.font_size_percent()
                ));
            }
            Err(error) => {
                options.message = Some(format!("Could not save interface settings: {error}"));
            }
        }
    } else {
        options.message = Some("The selected setting is already at its limit.".to_string());
    }
}

fn import_save(options: &mut OptionsState, _lifecycle: &mut LifecycleState) {
    #[cfg(not(target_os = "android"))]
    {
        let Some(source) = rfd::FileDialog::new()
            .set_title("Import a save")
            .add_filter("The Ever-Unfolding Saga saves", &["gz", "json", "save"])
            .pick_file()
        else {
            options.message = Some("Save import cancelled.".to_string());
            return;
        };
        match crate::persistence::install_imported_save(&source) {
            Ok(target) => {
                options.message = Some(format!(
                    "Imported save as {}.",
                    target.file_name().unwrap_or_default().to_string_lossy()
                ));
                refresh_catalogs(options);
                _lifecycle.refresh_saved_games();
            }
            Err(error) => options.message = Some(format!("Could not import save: {error}")),
        }
    }
    #[cfg(target_os = "android")]
    {
        if android_request_storage_import("saves") {
            options.message =
                Some("Choose a save file in the Android document picker.".to_string());
        } else {
            options.message = Some("The Android document picker is unavailable.".to_string());
        }
    }
}

fn export_save(options: &mut OptionsState, source: &std::path::Path) {
    if let Err(error) = crate::persistence::validate_save(source) {
        options.message = Some(format!("Could not export invalid save: {error}"));
        return;
    }
    let file_name = source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "ever_unfolding_saga_save.json.gz".to_string());
    #[cfg(not(target_os = "android"))]
    {
        let Some(destination) = rfd::FileDialog::new()
            .set_title("Export save")
            .set_file_name(&file_name)
            .save_file()
        else {
            options.message = Some("Save export cancelled.".to_string());
            return;
        };
        match crate::persistence::export_save(source, &destination) {
            Ok(()) => {
                options.message = Some(format!("Exported save to {}.", destination.display()))
            }
            Err(error) => options.message = Some(format!("Could not export save: {error}")),
        }
    }
    #[cfg(target_os = "android")]
    {
        if android_request_storage_export(source, &file_name, "application/gzip") {
            options.message = Some("Choose a destination for the save export.".to_string());
        } else {
            options.message = Some("The Android document picker is unavailable.".to_string());
        }
    }
}

fn import_mod(options: &mut OptionsState) {
    #[cfg(not(target_os = "android"))]
    {
        let Some(source) = rfd::FileDialog::new()
            .set_title("Install a mod ZIP")
            .add_filter("ZIP archives", &["zip"])
            .pick_file()
        else {
            options.message = Some("Mod installation cancelled.".to_string());
            return;
        };
        match crate::mod_manager::install_mod_archive(&source) {
            Ok(item) => {
                options.pending_restart = true;
                options.message = Some(format!("Installed {}.", item.manifest.name));
                refresh_catalogs(options);
            }
            Err(error) => options.message = Some(format!("Could not install mod: {error}")),
        }
    }
    #[cfg(target_os = "android")]
    {
        if android_request_storage_import("mods") {
            options.message = Some("Choose a mod ZIP in the Android document picker.".to_string());
        } else {
            options.message = Some("The Android document picker is unavailable.".to_string());
        }
    }
}

fn close_options(
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
    options: &mut OptionsState,
    gameplay: &GameplayState,
) {
    let return_screen = navigation.return_screen.take();
    lifecycle.refresh_saved_games();
    options.active = false;
    options.selected = 0;
    options.tab = 0;
    options.message = None;
    options.pending_remove_id = None;
    options.dirty = true;
    match return_screen {
        Some(ScreenId::Lifecycle) => {
            navigation.current_screen = Some(ScreenId::Lifecycle);
            navigation.selected = lifecycle.selected;
            lifecycle.mark_dirty();
        }
        _ => {
            navigation.current_screen = Some(ScreenId::Gameplay);
            navigation.selected = gameplay.navigation_selection();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
    ui_settings: Res<crate::ui_settings::UiSettings>,
    roots: Query<Entity, With<BevyScreenRoot>>,
    options_screens: Query<(), With<OptionsScreen>>,
    options_content: Query<Entity, With<OptionsContent>>,
    children: Query<&Children>,
    tab_buttons: Query<(Entity, &bevy_presentation::UiTabButton)>,
) {
    if !matches!(
        lifecycle.phase,
        LifecyclePhase::Start | LifecyclePhase::Complete
    ) || navigation.current_screen != Some(ScreenId::Options)
        || !options.active
        || !options.dirty
    {
        return;
    }

    if !options_screens.is_empty() {
        if let Some(content) = options_content.iter().next() {
            bevy_presentation::clear_tab_content(&mut commands, content, &children);
            render_options_content(&mut commands, content, &options, &ui_settings);
            bevy_presentation::update_tab_strip_selection(&mut commands, &tab_buttons, options.tab);
            options.dirty = false;
            return;
        }
    }

    for root in &roots {
        commands.entity(root).despawn();
    }

    let root = bevy_presentation::spawn_screen(&mut commands, "OPTIONS");
    commands.entity(root).insert(OptionsScreen);
    let panel = bevy_presentation::spawn_fixed_panel(&mut commands, root);
    commands.entity(panel).insert((
        bevy_presentation::UiSurface,
        BorderColor::all(bevy_presentation::THEME_BORDER),
        BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
    ));
    bevy_presentation::spawn_tab_strip(
        &mut commands,
        panel,
        &["Saves", "Mods", "Interface"],
        options.tab,
    );

    let content = bevy_presentation::spawn_tab_content(&mut commands, panel);
    commands.entity(content).insert(OptionsContent);
    render_options_content(&mut commands, content, &options, &ui_settings);
    options.dirty = false;
}

fn render_options_content(
    commands: &mut Commands,
    content: Entity,
    options: &OptionsState,
    ui_settings: &crate::ui_settings::UiSettings,
) {
    let list = bevy_presentation::spawn_scrollable_list(commands, content);
    match options.tab {
        0 => render_saves_tab(commands, list, options),
        1 => render_mods_tab(commands, list, options),
        2 => render_interface_tab(commands, list, options, ui_settings),
        _ => {}
    }

    if options.pending_restart {
        bevy_presentation::spawn_context_message(
            commands,
            content,
            "Mod changes are pending. Restart the game to apply them.",
        );
    }
    if let Some(message) = &options.message {
        bevy_presentation::spawn_context_message(commands, content, message);
    }

    let footer = bevy_presentation::spawn_option_footer(commands, content);
    match options.tab {
        0 => render_saves_footer(commands, footer, options),
        1 => render_mods_footer(commands, footer, options),
        2 => render_interface_footer(commands, footer, options),
        _ => {}
    }
}

fn render_saves_tab(commands: &mut Commands, list: Entity, options: &OptionsState) {
    bevy_presentation::spawn_label(commands, list, "SAVES");
    bevy_presentation::spawn_muted_label(commands, list, "Imports never overwrite existing saves.");
    if options.saves.is_empty() {
        bevy_presentation::spawn_muted_label(commands, list, "No saves found.");
    }
    for (index, save) in options.saves.iter().enumerate() {
        let row = bevy_presentation::spawn_option_row(commands, list);
        if let Some(metadata) = &save.metadata {
            bevy_presentation::spawn_label(
                commands,
                row.label,
                metadata.character_display_name.clone(),
            );
            bevy_presentation::spawn_muted_label(commands, row.label, metadata.details_line());
        } else {
            let name = save
                .path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| save.path.display().to_string());
            bevy_presentation::spawn_label(commands, row.label, name);
            bevy_presentation::spawn_muted_label(commands, row.label, "Save metadata unavailable.");
        }
        spawn_option_control_button(
            commands,
            row.controls,
            index,
            "Export",
            options.selected == index,
        );
    }
}

fn render_saves_footer(commands: &mut Commands, footer: Entity, options: &OptionsState) {
    let import_index = options.saves.len();
    spawn_option_control_button(
        commands,
        footer,
        import_index,
        "Import save",
        options.selected == import_index,
    );
    spawn_option_control_button(
        commands,
        footer,
        import_index + 1,
        "Back",
        options.selected == import_index + 1,
    );
}

fn render_mods_tab(commands: &mut Commands, list: Entity, options: &OptionsState) {
    bevy_presentation::spawn_label(commands, list, "MOD MANAGEMENT");
    if options.mods.is_empty() {
        bevy_presentation::spawn_muted_label(commands, list, "No mods installed.");
    }
    for (mod_index, item) in options.mods.iter().enumerate() {
        let row = bevy_presentation::spawn_option_row(commands, list);
        bevy_presentation::spawn_label(commands, row.label, &item.manifest.name);
        bevy_presentation::spawn_muted_label(
            commands,
            row.label,
            format!(
                "id={} · version={} · priority={} · assets={}",
                item.manifest.id, item.manifest.version, item.priority, item.asset_count
            ),
        );

        let base = mod_action_start_index(mod_index);
        spawn_option_control_button(
            commands,
            row.controls,
            base + MOD_ENABLE_ACTION,
            if item.enabled { "Disable" } else { "Enable" },
            options.selected == base + MOD_ENABLE_ACTION,
        );
        spawn_option_control_button(
            commands,
            row.controls,
            base + MOD_DELETE_ACTION,
            if options.pending_remove_id.as_deref() == Some(item.manifest.id.as_str()) {
                "Confirm"
            } else {
                "Delete"
            },
            options.selected == base + MOD_DELETE_ACTION,
        );
        spawn_option_control_button(
            commands,
            row.controls,
            base + MOD_PRIORITY_UP_ACTION,
            "Priority+",
            options.selected == base + MOD_PRIORITY_UP_ACTION,
        );
        spawn_option_control_button(
            commands,
            row.controls,
            base + MOD_PRIORITY_DOWN_ACTION,
            "Priority—",
            options.selected == base + MOD_PRIORITY_DOWN_ACTION,
        );
    }
}

fn render_mods_footer(commands: &mut Commands, footer: Entity, options: &OptionsState) {
    let install_index = mod_install_action_index(options.mods.len());
    spawn_option_control_button(
        commands,
        footer,
        install_index,
        "Install mod ZIP",
        options.selected == install_index,
    );
    spawn_option_control_button(
        commands,
        footer,
        install_index + 1,
        "Back",
        options.selected == install_index + 1,
    );
}

fn spawn_option_control_button(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    label: impl Into<String>,
    selected: bool,
) {
    let button = bevy_presentation::spawn_compact_action_button(commands, parent, index, label);
    if selected {
        commands.entity(button).insert(UiSelected);
    }
}

fn render_interface_tab(
    commands: &mut Commands,
    list: Entity,
    options: &OptionsState,
    settings: &crate::ui_settings::UiSettings,
) {
    bevy_presentation::spawn_label(commands, list, "INTERFACE");
    for (label, value, decrease, increase) in [
        ("UI scale", settings.ui_scale_percent(), 0, 1),
        ("Font size", settings.font_size_percent(), 2, 3),
    ] {
        let row = bevy_presentation::spawn_option_row(commands, list);
        bevy_presentation::spawn_label(commands, row.label, label);
        spawn_option_control_button(
            commands,
            row.controls,
            decrease,
            "-",
            options.selected == decrease,
        );
        bevy_presentation::spawn_option_value(commands, row.controls, format!("{value}%"));
        spawn_option_control_button(
            commands,
            row.controls,
            increase,
            "+",
            options.selected == increase,
        );
    }
}

fn render_interface_footer(commands: &mut Commands, footer: Entity, options: &OptionsState) {
    spawn_option_control_button(commands, footer, 4, "Reset", options.selected == 4);
    spawn_option_control_button(commands, footer, 5, "Back", options.selected == 5);
}

#[cfg(target_os = "android")]
pub(crate) fn android_request_storage_import(kind: &str) -> bool {
    let Some(app) = bevy::android::ANDROID_APP.get().cloned() else {
        return false;
    };
    let kind = kind.to_owned();
    let java_app = app.clone();
    app.run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(java_app.vm_as_ptr().cast()) };
        if let Err(error) = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = java_app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let value = env.new_string(&kind)?;
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("requestStorageImport"),
                jni::jni_sig!("(Ljava/lang/String;)V"),
                &[jni::JValue::from(&value)],
            )?;
            Ok(())
        }) {
            bevy::log::warn!("Could not open Android document picker: {error}");
        }
    }));
    true
}

#[cfg(target_os = "android")]
pub(crate) fn android_request_storage_export(
    source: &std::path::Path,
    file_name: &str,
    mime_type: &str,
) -> bool {
    let Some(app) = bevy::android::ANDROID_APP.get().cloned() else {
        return false;
    };
    let source = source.to_string_lossy().to_string();
    let file_name = file_name.to_owned();
    let mime_type = mime_type.to_owned();
    let java_app = app.clone();
    app.run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(java_app.vm_as_ptr().cast()) };
        if let Err(error) = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = java_app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let source = env.new_string(&source)?;
            let file_name = env.new_string(&file_name)?;
            let mime_type = env.new_string(&mime_type)?;
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("requestStorageExport"),
                jni::jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
                &[
                    jni::JValue::from(&source),
                    jni::JValue::from(&file_name),
                    jni::JValue::from(&mime_type),
                ],
            )?;
            Ok(())
        }) {
            bevy::log::warn!("Could not open Android export picker: {error}");
        }
    }));
    true
}

#[cfg(test)]
#[path = "../tests/unit/bevy_options.rs"]
mod tests;
