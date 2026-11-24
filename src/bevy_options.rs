//! Options-owned mod management and interface preferences.

use std::collections::HashMap;
#[cfg(target_os = "android")]
use std::path::PathBuf;

use bevy::prelude::*;

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

#[derive(Component, Clone, Copy)]
struct OptionsContent {
    list: Entity,
    rows: Entity,
    messages: Entity,
    footer: Entity,
    tab: usize,
}

#[derive(Component)]
struct ModListRow {
    key: String,
    title: Entity,
    detail: Entity,
    buttons: [Entity; MOD_ACTIONS_PER_MOD],
    button_labels: [Entity; MOD_ACTIONS_PER_MOD],
}

#[derive(Component)]
struct OptionsEmptyState;

#[derive(Resource)]
pub(crate) struct OptionsState {
    selected: usize,
    tab: usize,
    message: Option<String>,
    active: bool,
    dirty: bool,
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
    let entries = match std::fs::read_dir(&paths.mod_imports_dir) {
        Ok(entries) => entries,
        Err(error) => {
            options.message = Some(format!("Could not inspect imported mods: {error}"));
            options.dirty = true;
            return;
        }
    };

    let mut changed = false;
    for entry in entries.flatten() {
        let source = entry.path();
        if !source.is_file()
            || source.extension().and_then(|value| value.to_str()) != Some("zip")
            || options.processed_imports.contains(&source)
        {
            continue;
        }

        options.processed_imports.insert(source.clone());
        match crate::mod_manager::install_mod_archive(&source) {
            Ok(item) => {
                let _ = std::fs::remove_file(&source);
                options.pending_restart = true;
                options.message = Some(format!("Installed {}.", item.manifest.name));
                changed = true;
            }
            Err(error) => {
                options.message = Some(format!("Could not import {}: {error}", source.display()));
                options.dirty = true;
            }
        }
    }
    if changed {
        refresh_catalogs(&mut options);
        options.dirty = true;
    }
}

#[cfg(not(target_os = "android"))]
fn refresh_storage(
    _time: Res<Time>,
    _navigation: Res<NavigationState>,
    _options: ResMut<OptionsState>,
) {
}

fn options_input(
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut options: ResMut<OptionsState>,
    mut ui_settings: ResMut<crate::ui_settings::UiSettings>,
    mut audio_settings: ResMut<crate::audio_settings::AudioSettings>,
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
            InputEvent::SetSelection(index) => {
                options.selected = index.min(options_selection_end(&options));
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
                0 => activate_mods_action(&mut options, &mut lifecycle, &mut navigation),
                1 => activate_interface_action(
                    &mut options,
                    &mut ui_settings,
                    &mut lifecycle,
                    &mut navigation,
                ),
                2 => activate_audio_action(
                    &mut options,
                    &mut audio_settings,
                    &mut lifecycle,
                    &mut navigation,
                ),
                _ => {}
            },
            InputEvent::Cancel => close_options(&mut lifecycle, &mut navigation, &mut options),
            _ => {}
        }
        options.dirty = true;
    }
    navigation.selected = options.selected;
}

fn options_selection_end(options: &OptionsState) -> usize {
    match options.tab {
        0 => mod_back_action_index(options.mods.len()),
        1 | 2 => 5,
        _ => 0,
    }
}

fn activate_mods_action(
    options: &mut OptionsState,
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
) {
    if options.selected == mod_install_action_index(options.mods.len()) {
        import_mod(options);
        return;
    }
    if options.selected == mod_back_action_index(options.mods.len()) {
        close_options(lifecycle, navigation, options);
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
            close_options(lifecycle, navigation, options);
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

fn activate_audio_action(
    options: &mut OptionsState,
    settings: &mut crate::audio_settings::AudioSettings,
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
) {
    let changed = match options.selected {
        0 => settings.adjust_music_volume(-crate::audio_settings::VOLUME_STEP),
        1 => settings.adjust_music_volume(crate::audio_settings::VOLUME_STEP),
        2 => settings.adjust_sfx_volume(-crate::audio_settings::VOLUME_STEP),
        3 => settings.adjust_sfx_volume(crate::audio_settings::VOLUME_STEP),
        4 => {
            settings.reset();
            true
        }
        5 => {
            close_options(lifecycle, navigation, options);
            return;
        }
        _ => false,
    };

    if changed {
        match settings.save() {
            Ok(()) => {
                options.message = Some(format!(
                    "Music volume: {}%. Sound effects: {}%.",
                    settings.music_volume_percent(),
                    settings.sfx_volume_percent()
                ));
            }
            Err(error) => {
                options.message = Some(format!("Could not save audio settings: {error}"));
            }
        }
    } else {
        options.message = Some("The selected volume is already at its limit.".to_string());
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
) {
    let return_screen = navigation.return_to_previous_screen(ScreenId::SpatialExploration);
    options.active = false;
    options.selected = 0;
    options.tab = 0;
    options.message = None;
    options.pending_remove_id = None;
    options.dirty = true;
    if return_screen == ScreenId::Lifecycle {
        navigation.selected = lifecycle.selected;
        lifecycle.mark_dirty();
    }
}

#[allow(clippy::too_many_arguments)]
fn render_if_active(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut options: ResMut<OptionsState>,
    ui_settings: Res<crate::ui_settings::UiSettings>,
    audio_settings: Res<crate::audio_settings::AudioSettings>,
    roots: Query<Entity, With<BevyScreenRoot>>,
    options_screens: Query<(), With<OptionsScreen>>,
    options_content: Query<(Entity, &OptionsContent)>,
    mod_rows: Query<(Entity, &ModListRow)>,
    empty_states: Query<(), With<OptionsEmptyState>>,
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
        if let Some((content, retained)) = options_content.iter().next() {
            let mut hosts = *retained;
            let tab_changed = hosts.tab != options.tab;
            if tab_changed {
                commands
                    .entity(hosts.list)
                    .insert(bevy::ui::ScrollPosition::default());
                bevy_presentation::clear_content_host(&mut commands, hosts.list, &children);
                spawn_options_list_header(&mut commands, hosts.list, options.tab);
                hosts.rows = spawn_options_rows_host(&mut commands, hosts.list);
                hosts.tab = options.tab;
                commands.entity(content).insert(hosts);
                render_options_list_content(
                    &mut commands,
                    hosts.rows,
                    &options,
                    &ui_settings,
                    &audio_settings,
                    &children,
                    &mod_rows,
                    &empty_states,
                );
            } else {
                match options.tab {
                    0 => reconcile_mod_rows(
                        &mut commands,
                        hosts.rows,
                        &options,
                        &children,
                        &mod_rows,
                        &empty_states,
                    ),
                    1 => {
                        bevy_presentation::clear_content_host(&mut commands, hosts.rows, &children);
                        render_interface_tab(&mut commands, hosts.rows, &options, &ui_settings);
                    }
                    2 => {
                        bevy_presentation::clear_content_host(&mut commands, hosts.rows, &children);
                        render_audio_tab(&mut commands, hosts.rows, &options, &audio_settings);
                    }
                    _ => {}
                }
            }

            clear_options_content_hosts(&mut commands, hosts, &children);
            render_options_aux_content(&mut commands, hosts.messages, hosts.footer, &options);
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
        &["Mods", "Interface", "Audio"],
        options.tab,
    );

    let content = bevy_presentation::spawn_tab_content(&mut commands, panel);
    let hosts = spawn_options_content_hosts(&mut commands, content, options.tab);
    render_options_list_content(
        &mut commands,
        hosts.rows,
        &options,
        &ui_settings,
        &audio_settings,
        &children,
        &mod_rows,
        &empty_states,
    );
    render_options_aux_content(&mut commands, hosts.messages, hosts.footer, &options);
    options.dirty = false;
}

fn spawn_options_content_hosts(
    commands: &mut Commands,
    parent: Entity,
    tab: usize,
) -> OptionsContent {
    let list = bevy_presentation::spawn_scrollable_list(commands, parent);
    spawn_options_list_header(commands, list, tab);
    let rows = spawn_options_rows_host(commands, list);
    let messages = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            flex_shrink: 0.0,
            ..default()
        })
        .id();
    commands.entity(parent).add_child(messages);
    let footer = bevy_presentation::spawn_option_footer(commands, parent);
    let hosts = OptionsContent {
        list,
        rows,
        messages,
        footer,
        tab,
    };
    commands.entity(parent).insert(hosts);
    hosts
}

fn spawn_options_list_header(commands: &mut Commands, list: Entity, tab: usize) {
    if tab == 0 {
        bevy_presentation::spawn_label(commands, list, "MOD MANAGEMENT");
    }
}

fn spawn_options_rows_host(commands: &mut Commands, parent: Entity) -> Entity {
    let rows = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: responsive_option_row_gap(),
            ..default()
        })
        .id();
    commands.entity(parent).add_child(rows);
    rows
}

fn responsive_option_row_gap() -> Val {
    bevy_presentation::responsive_vmin(1.389)
}

fn clear_options_content_hosts(
    commands: &mut Commands,
    hosts: OptionsContent,
    children: &Query<&Children>,
) {
    for host in [hosts.messages, hosts.footer] {
        bevy_presentation::clear_content_host(commands, host, children);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_options_list_content(
    commands: &mut Commands,
    rows: Entity,
    options: &OptionsState,
    ui_settings: &crate::ui_settings::UiSettings,
    audio_settings: &crate::audio_settings::AudioSettings,
    children: &Query<&Children>,
    mod_rows: &Query<(Entity, &ModListRow)>,
    empty_states: &Query<(), With<OptionsEmptyState>>,
) {
    match options.tab {
        0 => render_mods_tab(commands, rows, options, children, mod_rows, empty_states),
        1 => render_interface_tab(commands, rows, options, ui_settings),
        2 => render_audio_tab(commands, rows, options, audio_settings),
        _ => {}
    }
}

fn render_options_aux_content(
    commands: &mut Commands,
    messages: Entity,
    footer: Entity,
    options: &OptionsState,
) {
    if options.pending_restart {
        bevy_presentation::spawn_context_message(
            commands,
            messages,
            "Mod changes are pending. Restart the game to apply them.",
        );
    }
    if let Some(message) = &options.message {
        bevy_presentation::spawn_context_message(commands, messages, message);
    }
    match options.tab {
        0 => render_mods_footer(commands, footer, options),
        1 => render_interface_footer(commands, footer, options),
        2 => render_audio_footer(commands, footer, options),
        _ => {}
    }
}

fn render_mods_tab(
    commands: &mut Commands,
    rows: Entity,
    options: &OptionsState,
    children: &Query<&Children>,
    mod_rows: &Query<(Entity, &ModListRow)>,
    empty_states: &Query<(), With<OptionsEmptyState>>,
) {
    reconcile_mod_rows(commands, rows, options, children, mod_rows, empty_states);
}

fn mod_row_detail(item: &InstalledMod) -> String {
    format!(
        "id={} | version={} | priority={} | assets={}",
        item.manifest.id, item.manifest.version, item.priority, item.asset_count
    )
}

fn mod_row_action_labels(
    item: &InstalledMod,
    options: &OptionsState,
) -> [String; MOD_ACTIONS_PER_MOD] {
    [
        (if item.enabled { "Disable" } else { "Enable" }).to_string(),
        (if options.pending_remove_id.as_deref() == Some(item.manifest.id.as_str()) {
            "Confirm"
        } else {
            "Delete"
        })
        .to_string(),
        "Priority+".to_string(),
        "Priority:".to_string(),
    ]
}

fn reconcile_mod_rows(
    commands: &mut Commands,
    rows_host: Entity,
    options: &OptionsState,
    children: &Query<&Children>,
    existing_rows: &Query<(Entity, &ModListRow)>,
    empty_states: &Query<(), With<OptionsEmptyState>>,
) {
    let mut existing = HashMap::new();
    let mut empty_state = None;
    if let Ok(current_children) = children.get(rows_host) {
        for child in current_children.iter() {
            if let Ok((entity, row)) = existing_rows.get(child) {
                existing.insert(row.key.clone(), (entity, row));
            } else if empty_states.get(child).is_ok() {
                empty_state = Some(child);
            }
        }
    }

    let mut ordered = Vec::with_capacity(options.mods.len().saturating_add(1));
    for (mod_index, item) in options.mods.iter().enumerate() {
        let key = item.manifest.id.clone();
        let title = item.manifest.name.clone();
        let detail = mod_row_detail(item);
        let action_labels = mod_row_action_labels(item, options);
        let base = mod_action_start_index(mod_index);
        let entity = if let Some((entity, row)) = existing.remove(&key) {
            commands.entity(row.title).insert(Text::new(title));
            commands.entity(row.detail).insert(Text::new(detail));
            for (action, action_label) in action_labels.iter().enumerate() {
                commands
                    .entity(row.button_labels[action])
                    .insert(Text::new(action_label.clone()));
                update_option_row_button(
                    commands,
                    row.buttons[action],
                    base + action,
                    options.selected == base + action,
                );
            }
            entity
        } else {
            let row = bevy_presentation::spawn_option_row(commands, rows_host);
            let title_entity = bevy_presentation::spawn_label(commands, row.label, title);
            let detail_entity = bevy_presentation::spawn_muted_label(commands, row.label, detail);
            let mut buttons = Vec::with_capacity(MOD_ACTIONS_PER_MOD);
            let mut button_labels = Vec::with_capacity(MOD_ACTIONS_PER_MOD);
            for (action, action_label) in action_labels.iter().enumerate() {
                let (button, button_label) =
                    bevy_presentation::spawn_compact_action_button_with_text(
                        commands,
                        row.controls,
                        base + action,
                        action_label.clone(),
                    );
                update_option_row_button(
                    commands,
                    button,
                    base + action,
                    options.selected == base + action,
                );
                buttons.push(button);
                button_labels.push(button_label);
            }
            commands.entity(row.row).insert(ModListRow {
                key,
                title: title_entity,
                detail: detail_entity,
                buttons: buttons.try_into().expect("mod row has four action buttons"),
                button_labels: button_labels
                    .try_into()
                    .expect("mod row has four action labels"),
            });
            row.row
        };
        ordered.push(entity);
    }

    for (_, (entity, _)) in existing {
        commands.entity(entity).despawn();
    }

    if options.mods.is_empty() {
        let entity = empty_state.unwrap_or_else(|| {
            let label =
                bevy_presentation::spawn_muted_label(commands, rows_host, "No mods installed.");
            commands.entity(label).insert(OptionsEmptyState);
            label
        });
        commands
            .entity(entity)
            .insert(Text::new("No mods installed."));
        ordered.push(entity);
    } else if let Some(entity) = empty_state {
        commands.entity(entity).despawn();
    }
    commands.entity(rows_host).replace_children(&ordered);
}

fn update_option_row_button(commands: &mut Commands, button: Entity, index: usize, selected: bool) {
    commands
        .entity(button)
        .insert(bevy_presentation::ChoiceButton { index });
    if selected {
        commands.entity(button).insert(UiSelected);
    } else {
        commands.entity(button).remove::<UiSelected>();
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

fn render_audio_tab(
    commands: &mut Commands,
    list: Entity,
    options: &OptionsState,
    settings: &crate::audio_settings::AudioSettings,
) {
    bevy_presentation::spawn_label(commands, list, "AUDIO");
    bevy_presentation::spawn_muted_label(
        commands,
        list,
        "No sound-effect or ambience assets are installed yet.",
    );
    for (label, value, decrease, increase) in [
        ("Music volume", settings.music_volume_percent(), 0, 1),
        ("SFX / ambience", settings.sfx_volume_percent(), 2, 3),
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

fn render_audio_footer(commands: &mut Commands, footer: Entity, options: &OptionsState) {
    spawn_option_control_button(commands, footer, 4, "Reset", options.selected == 4);
    spawn_option_control_button(commands, footer, 5, "Back", options.selected == 5);
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

#[cfg(test)]
#[path = "../tests/unit/bevy_options.rs"]
mod tests;
