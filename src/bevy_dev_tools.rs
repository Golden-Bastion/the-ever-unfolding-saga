//! Start-screen developer tools and isolated visual experimentation sessions.
//!
//! The workshop reads the same merged campaign catalogue as gameplay and calls
//! the dialogue portrait compositor, keeping preview output on the gameplay path.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use bevy::prelude::*;

use crate::bevy_interactions::{self, PortraitImageCache};
use crate::bevy_lifecycle::LifecycleState;
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::bevy_spatial::{self, SpatialExplorationState};
use crate::character_appearance::generate_character_portrait;
use crate::content::{loader::load_campaign_content_report, CharacterPartContent};
use crate::game_paths::GamePaths;
use crate::input::InputEvent;
use crate::presentation::{CharacterPortraitLayerView, CharacterPortraitView};
use crate::spatial::{
    generate_local_map_with_layout, GridPosition, LocalMap, LocalMapConfig, SpatialPropKind,
    Terrain,
};
use crate::spatial_fixtures::{
    list_spatial_fixtures, load_spatial_fixture, save_spatial_fixture, SpatialFixtureEntry,
};

const WORKSHOP_ACTIONS: &[(&str, &str)] = &[
    ("back", "Previous part"),
    ("continue", "Next part"),
    ("back", "Previous slot"),
    ("continue", "Next slot"),
    ("display", "Toggle layer visibility"),
    ("choice", "Randomize combination"),
    ("cancel", "Reset combination"),
    ("back", "Back to Developer Tools"),
];

const WORLD_LAB_ACTIONS: &[(&str, &str)] = &[
    ("back", "Decrease seed"),
    ("continue", "Increase seed"),
    ("display", "Cycle map dimensions"),
    ("choice", "Toggle spatial layout"),
    ("investigate", "Generate map"),
    ("edit", "Edit map"),
    ("explore", "Play-test map"),
    ("write", "Save / export fixture"),
    ("folder", "Browse / load fixtures"),
    ("cancel", "Reset settings"),
    ("back", "Back to Developer Tools"),
];

const WORLD_LAB_EDITOR_ACTIONS: &[(&str, &str)] = &[
    ("back", "Move cursor west"),
    ("continue", "Move cursor east"),
    ("back", "Move cursor north"),
    ("continue", "Move cursor south"),
    ("display", "Cycle selected terrain"),
    ("choice", "Cycle prop type"),
    ("investigate", "Place prop"),
    ("explore", "Pick up / drop prop"),
    ("cancel", "Remove prop"),
    ("back", "Back to World Lab"),
];

const WORLD_LAB_SIZES: [(u16, u16); 3] = [(24, 16), (32, 20), (40, 24)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorldLabOptions {
    seed: u64,
    size_index: usize,
    shrine_layout: bool,
}

impl Default for WorldLabOptions {
    fn default() -> Self {
        Self {
            seed: 1,
            size_index: 0,
            shrine_layout: false,
        }
    }
}

#[derive(Debug, Clone)]
struct WorkshopSlot {
    name: String,
    parts: Vec<CharacterPartContent>,
    selected_part: usize,
    visible: bool,
}

#[derive(Debug, Clone)]
struct WorldLabFixtureEntry {
    path: PathBuf,
    name: String,
    details: String,
    map: Option<LocalMap>,
    error: Option<String>,
}

impl From<SpatialFixtureEntry> for WorldLabFixtureEntry {
    fn from(entry: SpatialFixtureEntry) -> Self {
        Self {
            path: entry.path,
            name: entry.name,
            details: entry.details,
            map: entry.map,
            error: entry.error,
        }
    }
}

#[derive(Resource, Default)]
struct DeveloperToolsState {
    slots: Vec<WorkshopSlot>,
    selected_slot: usize,
    seed: u64,
    load_warnings: Vec<String>,
    world_lab: WorldLabOptions,
    world_lab_generated_options: Option<WorldLabOptions>,
    world_lab_status: Option<String>,
    world_lab_cursor: Option<GridPosition>,
    world_lab_prop_kind: Option<SpatialPropKind>,
    world_lab_held_prop: Option<String>,
    world_lab_fixtures: Vec<WorldLabFixtureEntry>,
    dirty: bool,
    rendered_screen: Option<ScreenId>,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<DeveloperToolsState>()
        .add_systems(Startup, initialize)
        .add_systems(Update, (developer_tools_input, render_if_active).chain());
}

fn initialize(mut state: ResMut<DeveloperToolsState>) {
    let report = load_campaign_content_report();
    state.slots = build_slots(report.content.character_parts);
    state.load_warnings = report.warnings;
    state.selected_slot = 0;
    state.seed = 0;
    state.dirty = true;
}

fn build_slots(parts: Vec<CharacterPartContent>) -> Vec<WorkshopSlot> {
    let mut grouped = BTreeMap::<String, Vec<CharacterPartContent>>::new();
    for part in parts {
        grouped.entry(part.slot.clone()).or_default().push(part);
    }

    grouped
        .into_iter()
        .map(|(name, mut parts)| {
            parts.sort_by(|left, right| {
                left.id
                    .cmp(&right.id)
                    .then_with(|| left.asset.cmp(&right.asset))
            });
            WorkshopSlot {
                name,
                parts,
                selected_part: 0,
                visible: true,
            }
        })
        .collect()
}

fn build_preview_portrait(slots: &[WorkshopSlot]) -> CharacterPortraitView {
    let mut layers = slots
        .iter()
        .filter(|slot| slot.visible)
        .filter_map(|slot| slot.parts.get(slot.selected_part))
        .filter(|part| !part.asset.trim().is_empty())
        .map(|part| CharacterPortraitLayerView {
            id: part.id.clone(),
            slot: part.slot.clone(),
            asset: part.asset.clone(),
            layer: part.layer,
        })
        .collect::<Vec<_>>();

    layers.sort_by(|left, right| {
        left.layer
            .cmp(&right.layer)
            .then_with(|| left.slot.cmp(&right.slot))
            .then_with(|| left.id.cmp(&right.id))
    });
    CharacterPortraitView { layers }
}

fn cycle_index(current: usize, len: usize, direction: isize) -> usize {
    if len == 0 {
        return 0;
    }
    match direction.cmp(&0) {
        std::cmp::Ordering::Less => {
            if current == 0 || current >= len {
                len - 1
            } else {
                current - 1
            }
        }
        std::cmp::Ordering::Greater => {
            if current >= len - 1 {
                0
            } else {
                current + 1
            }
        }
        std::cmp::Ordering::Equal => current.min(len - 1),
    }
}

fn cycle_current_part(state: &mut DeveloperToolsState, direction: isize) {
    let Some(slot) = state.slots.get_mut(state.selected_slot) else {
        return;
    };
    slot.selected_part = cycle_index(slot.selected_part, slot.parts.len(), direction);
    state.dirty = true;
}

fn cycle_current_slot(state: &mut DeveloperToolsState, direction: isize) {
    state.selected_slot = cycle_index(state.selected_slot, state.slots.len(), direction);
    state.dirty = true;
}

fn randomize_combination(state: &mut DeveloperToolsState) {
    let parts = state
        .slots
        .iter()
        .flat_map(|slot| slot.parts.iter().cloned())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return;
    }

    state.seed = state.seed.wrapping_add(1);
    let generated = generate_character_portrait(&parts, state.seed, &[]);
    for slot in &mut state.slots {
        let Some(layer) = generated.iter().find(|layer| layer.slot == slot.name) else {
            continue;
        };
        if let Some(index) = slot.parts.iter().position(|part| part.id == layer.id) {
            slot.selected_part = index;
        }
    }
    state.dirty = true;
}

fn reset_combination(state: &mut DeveloperToolsState) {
    for slot in &mut state.slots {
        slot.selected_part = 0;
        slot.visible = true;
    }
    state.selected_slot = 0;
    state.seed = 0;
    state.dirty = true;
}

fn generate_world_lab_map(options: WorldLabOptions) -> LocalMap {
    let (width, height) = WORLD_LAB_SIZES[options.size_index.min(WORLD_LAB_SIZES.len() - 1)];
    generate_local_map_with_layout(
        options.seed,
        "developer.world_lab",
        LocalMapConfig { width, height },
        options.shrine_layout.then_some("shrine_chamber"),
    )
}

fn world_lab_map_preview(map: &LocalMap) -> String {
    world_lab_map_preview_with_cursor(map, None, None)
}

fn world_lab_map_preview_with_cursor(
    map: &LocalMap,
    cursor: Option<GridPosition>,
    held_prop_id: Option<&str>,
) -> String {
    let mut preview = String::with_capacity(usize::from(map.width) * (usize::from(map.height) + 1));
    for y in 0..map.height {
        for x in 0..map.width {
            let position = GridPosition { x, y };
            let marker = if cursor == Some(position) {
                'X'
            } else if position == map.entry {
                '@'
            } else if map
                .points_of_interest
                .iter()
                .any(|poi| poi.position == position)
            {
                '?'
            } else if let Some(prop) = map
                .props
                .iter()
                .find(|prop| prop.footprint_contains(position, map.width))
            {
                if held_prop_id == Some(prop.id.as_str()) {
                    'H'
                } else {
                    'P'
                }
            } else {
                match map.terrain_at(position).unwrap_or(Terrain::Wall) {
                    Terrain::Floor => '.',
                    Terrain::Ash => ':',
                    Terrain::Rubble => '%',
                    Terrain::Wall => '#',
                }
            };
            preview.push(marker);
        }
        if y + 1 < map.height {
            preview.push('\n');
        }
    }
    preview
}

fn world_lab_terrain_name(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::Floor => "Floor",
        Terrain::Ash => "Ash",
        Terrain::Rubble => "Rubble",
        Terrain::Wall => "Wall",
    }
}

fn world_lab_prop_name(kind: SpatialPropKind) -> &'static str {
    match kind {
        SpatialPropKind::Pillar => "Pillar",
        SpatialPropKind::Table => "Table",
    }
}

fn world_lab_cursor(state: &DeveloperToolsState, map: &LocalMap) -> GridPosition {
    state
        .world_lab_cursor
        .filter(|position| map.contains(*position))
        .unwrap_or(map.entry)
}

fn move_world_lab_cursor(
    state: &mut DeveloperToolsState,
    spatial: &SpatialExplorationState,
    dx: i32,
    dy: i32,
) {
    let Some(map) = bevy_spatial::sandbox_map(spatial) else {
        return;
    };
    let current = world_lab_cursor(state, map);
    state.world_lab_cursor = Some(GridPosition {
        x: (i32::from(current.x) + dx).clamp(0, i32::from(map.width - 1)) as u16,
        y: (i32::from(current.y) + dy).clamp(0, i32::from(map.height - 1)) as u16,
    });
}

fn cycle_world_lab_terrain(state: &mut DeveloperToolsState, spatial: &mut SpatialExplorationState) {
    let position = state
        .world_lab_cursor
        .unwrap_or(GridPosition { x: 0, y: 0 });
    let Some(map) = bevy_spatial::sandbox_map_mut(spatial) else {
        state.world_lab_status = Some("No editable temporary map is available.".into());
        return;
    };
    let Some(current) = map.terrain_at(position) else {
        state.world_lab_status = Some("The selected map cell is outside the map.".into());
        return;
    };
    let next = match current {
        Terrain::Floor => Terrain::Ash,
        Terrain::Ash => Terrain::Rubble,
        Terrain::Rubble => Terrain::Wall,
        Terrain::Wall => Terrain::Floor,
    };
    if map.set_terrain_for_editor(position, next) {
        state.world_lab_status = Some(format!(
            "Changed ({}, {}) to {}.",
            position.x,
            position.y,
            world_lab_terrain_name(next)
        ));
    } else {
        state.world_lab_status =
            Some("That cell is reserved for an entry, actor, passage, discovery, or prop.".into());
    }
}

fn next_world_lab_prop_id(map: &LocalMap) -> String {
    for index in 1_u64.. {
        let id = format!("{}.editor_prop.{index}", map.location_id);
        if !map.props.iter().any(|prop| prop.id == id) {
            return id;
        }
    }
    unreachable!("the prop ID sequence is bounded by available memory");
}

fn place_world_lab_prop(state: &mut DeveloperToolsState, spatial: &mut SpatialExplorationState) {
    if state.world_lab_held_prop.is_some() {
        state.world_lab_status = Some("Drop the held prop before placing another.".into());
        return;
    }
    let position = state
        .world_lab_cursor
        .unwrap_or(GridPosition { x: 0, y: 0 });
    let kind = state.world_lab_prop_kind.unwrap_or(SpatialPropKind::Pillar);
    let placed = if let Some(map) = bevy_spatial::sandbox_map_mut(spatial) {
        let id = next_world_lab_prop_id(map);
        map.place_prop(id, kind, position)
    } else {
        false
    };
    state.world_lab_status = Some(if placed {
        format!(
            "Placed {} at ({}, {}).",
            world_lab_prop_name(kind),
            position.x,
            position.y
        )
    } else {
        "The prop needs a clear, walkable footprint away from reserved cells.".into()
    });
}

fn pick_up_or_drop_world_lab_prop(
    state: &mut DeveloperToolsState,
    spatial: &mut SpatialExplorationState,
) {
    let position = state
        .world_lab_cursor
        .unwrap_or(GridPosition { x: 0, y: 0 });
    if let Some(held_id) = state.world_lab_held_prop.clone() {
        let moved = bevy_spatial::sandbox_map_mut(spatial)
            .is_some_and(|map| map.move_prop(&held_id, position));
        if moved {
            state.world_lab_held_prop = None;
            state.world_lab_status =
                Some(format!("Moved prop to ({}, {}).", position.x, position.y));
        } else {
            state.world_lab_status =
                Some("The prop cannot fit there; choose a clear walkable footprint.".into());
        }
        return;
    }
    let selected = bevy_spatial::sandbox_map(spatial).and_then(|map| {
        map.props
            .iter()
            .find(|prop| prop.footprint_contains(position, map.width))
            .map(|prop| (prop.id.clone(), prop.kind, prop.position))
    });
    if let Some((id, kind, origin)) = selected {
        state.world_lab_held_prop = Some(id);
        state.world_lab_status = Some(format!(
            "Holding {} from ({}, {}). Move the cursor, then choose Pick up / drop prop.",
            world_lab_prop_name(kind),
            origin.x,
            origin.y
        ));
    } else {
        state.world_lab_status = Some("There is no prop under the selected cell.".into());
    }
}

fn remove_world_lab_prop(state: &mut DeveloperToolsState, spatial: &mut SpatialExplorationState) {
    let position = state
        .world_lab_cursor
        .unwrap_or(GridPosition { x: 0, y: 0 });
    let removed =
        bevy_spatial::sandbox_map_mut(spatial).and_then(|map| map.remove_prop_at(position));
    if let Some(prop) = removed {
        if state.world_lab_held_prop.as_deref() == Some(prop.id.as_str()) {
            state.world_lab_held_prop = None;
        }
        state.world_lab_status = Some(format!("Removed {}.", world_lab_prop_name(prop.kind)));
    } else {
        state.world_lab_status = Some("There is no prop under the selected cell.".into());
    }
}

fn world_lab_cell_details(map: &LocalMap, position: GridPosition) -> String {
    let terrain = map
        .terrain_at(position)
        .map(world_lab_terrain_name)
        .unwrap_or("Outside map");
    let mut markers = Vec::new();
    if position == map.entry {
        markers.push("entry".to_string());
    }
    for poi in &map.points_of_interest {
        if poi.position == position {
            markers.push(format!("point of interest: {}", poi.title));
        }
    }
    if map.exits.iter().any(|exit| exit.position == position) {
        markers.push("passage".to_string());
    }
    if map
        .interactables
        .iter()
        .any(|actor| actor.position == position)
    {
        markers.push("actor".to_string());
    }
    if let Some(prop) = map
        .props
        .iter()
        .find(|prop| prop.footprint_contains(position, map.width))
    {
        markers.push(format!("prop: {}", world_lab_prop_name(prop.kind)));
    }
    let markers = if markers.is_empty() {
        "unreserved cell".to_string()
    } else {
        markers.join(", ")
    };
    format!(
        "Selected cell ({}, {}) · {} · {}",
        position.x, position.y, terrain, markers
    )
}


fn refresh_world_lab_fixtures(state: &mut DeveloperToolsState) {
    let paths = GamePaths::get();
    match list_spatial_fixtures(&[
        paths.developer_fixtures_dir.as_path(),
        paths.spatial_fixture_imports_dir.as_path(),
    ]) {
        Ok(fixtures) => {
            state.world_lab_fixtures = fixtures.into_iter().map(Into::into).collect();
        }
        Err(error) => {
            state.world_lab_status = Some(format!("Could not read World Lab fixtures: {error}"));
        }
    }
    state.dirty = true;
}

fn activate_world_lab_map(
    state: &mut DeveloperToolsState,
    spatial: &mut SpatialExplorationState,
    navigation: &mut NavigationState,
    map: LocalMap,
    name: &str,
) {
    state.world_lab_cursor = Some(map.entry);
    state.world_lab_held_prop = None;
    bevy_spatial::set_sandbox_map(spatial, map);
    state.world_lab_generated_options = Some(state.world_lab);
    state.world_lab_status = Some(format!("Loaded fixture: {name}. The map is ready to edit or play-test."));
    navigation.current_screen = Some(ScreenId::WorldLab);
    navigation.selected = 5;
    state.dirty = true;
}

fn export_world_lab_fixture(
    state: &mut DeveloperToolsState,
    spatial: &SpatialExplorationState,
) {
    let Some(map) = bevy_spatial::sandbox_map(spatial) else {
        state.world_lab_status = Some("Generate or load a map before exporting a fixture.".into());
        return;
    };
    let name = format!("World Lab Seed {}", map.seed);
    let paths = GamePaths::get();
    let saved = match save_spatial_fixture(&paths.developer_fixtures_dir, &name, map) {
        Ok(path) => path,
        Err(error) => {
            state.world_lab_status = Some(format!("Could not save World Lab fixture: {error}"));
            return;
        }
    };
    let file_name = saved
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "world-lab-fixture.teus-map.json".to_string());

    #[cfg(not(target_os = "android"))]
    {
        let Some(destination) = rfd::FileDialog::new()
            .set_title("Export World Lab fixture")
            .set_file_name(&file_name)
            .add_filter("TEUS spatial map fixtures", &["json"])
            .save_file()
        else {
            state.world_lab_status = Some(format!(
                "Saved {name} internally. External export was cancelled."
            ));
            refresh_world_lab_fixtures(state);
            return;
        };
        match std::fs::copy(&saved, &destination) {
            Ok(_) => {
                state.world_lab_status = Some(format!(
                    "Saved {name} internally and exported it to {}.",
                    destination.display()
                ));
            }
            Err(error) => {
                state.world_lab_status = Some(format!(
                    "Saved internally, but could not export the fixture: {error}"
                ));
            }
        }
    }
    #[cfg(target_os = "android")]
    {
        if crate::bevy_options::android_request_storage_export(&saved, &file_name, "application/json") {
            state.world_lab_status =
                Some(format!("Saved {name} internally. Choose a destination for the JSON export."));
        } else {
            state.world_lab_status =
                Some(format!("Saved {name} internally, but the Android document picker is unavailable."));
        }
    }
    refresh_world_lab_fixtures(state);
}

fn import_world_lab_fixture(
    state: &mut DeveloperToolsState,
    spatial: &mut SpatialExplorationState,
    navigation: &mut NavigationState,
) {
    #[cfg(not(target_os = "android"))]
    {
        let Some(source) = rfd::FileDialog::new()
            .set_title("Import a World Lab fixture")
            .add_filter("TEUS spatial map fixtures", &["json"])
            .pick_file()
        else {
            state.world_lab_status = Some("Fixture import cancelled.".into());
            state.dirty = true;
            return;
        };
        match load_spatial_fixture(&source) {
            Ok(fixture) => {
                let paths = GamePaths::get();
                match save_spatial_fixture(
                    &paths.developer_fixtures_dir,
                    &fixture.name,
                    &fixture.map,
                ) {
                    Ok(saved) => {
                        state.world_lab_status = Some(format!(
                            "Imported {} as {}.",
                            source.display(),
                            saved.display()
                        ));
                        activate_world_lab_map(
                            state,
                            spatial,
                            navigation,
                            fixture.map,
                            &fixture.name,
                        );
                        refresh_world_lab_fixtures(state);
                    }
                    Err(error) => {
                        state.world_lab_status =
                            Some(format!("Could not keep the imported fixture: {error}"));
                        state.dirty = true;
                    }
                }
            }
            Err(error) => {
                state.world_lab_status =
                    Some(format!("Could not import World Lab fixture: {error}"));
                state.dirty = true;
            }
        }
    }
    #[cfg(target_os = "android")]
    {
        if crate::bevy_options::android_request_storage_import("spatial_fixtures") {
            state.world_lab_status = Some(
                "Choose a World Lab .json fixture. Return here and choose Refresh to inspect it."
                    .into(),
            );
        } else {
            state.world_lab_status = Some("The Android document picker is unavailable.".into());
        }
        state.dirty = true;
    }
}

fn load_world_lab_fixture(
    state: &mut DeveloperToolsState,
    spatial: &mut SpatialExplorationState,
    navigation: &mut NavigationState,
    index: usize,
) {
    let Some(entry) = state.world_lab_fixtures.get(index).cloned() else {
        return;
    };
    let Some(map) = entry.map else {
        state.world_lab_status = Some(format!(
            "Cannot load {}: {}",
            entry.name,
            entry.error.unwrap_or_else(|| "fixture is invalid".into())
        ));
        state.dirty = true;
        return;
    };

    let paths = GamePaths::get();
    if entry.path.starts_with(&paths.spatial_fixture_imports_dir) {
        match save_spatial_fixture(&paths.developer_fixtures_dir, &entry.name, &map) {
            Ok(_) => {
                if let Err(error) = std::fs::remove_file(&entry.path) {
                    bevy::log::warn!(
                        "Could not remove imported World Lab fixture {}: {error}",
                        entry.path.display()
                    );
                }
            }
            Err(error) => {
                state.world_lab_status =
                    Some(format!("Could not install imported fixture: {error}"));
                state.dirty = true;
                return;
            }
        }
    }

    refresh_world_lab_fixtures(state);
    activate_world_lab_map(state, spatial, navigation, map, &entry.name);
}

fn is_developer_tool_screen(screen: ScreenId) -> bool {
    matches!(
        screen,
        ScreenId::DeveloperTools
            | ScreenId::CharacterWorkshop
            | ScreenId::WorldLab
            | ScreenId::WorldLabEditor
            | ScreenId::WorldLabFixtures
    )
}

fn screen_action_count(screen: ScreenId, state: &DeveloperToolsState) -> usize {
    match screen {
        ScreenId::DeveloperTools => 3,
        ScreenId::CharacterWorkshop => WORKSHOP_ACTIONS.len(),
        ScreenId::WorldLab => WORLD_LAB_ACTIONS.len(),
        ScreenId::WorldLabEditor => WORLD_LAB_EDITOR_ACTIONS.len(),
        ScreenId::WorldLabFixtures => state.world_lab_fixtures.len() + 3,
        _ => 0,
    }
}

fn return_to_start(
    state: &mut DeveloperToolsState,
    lifecycle: &mut LifecycleState,
    navigation: &mut NavigationState,
) {
    navigation.current_screen = Some(ScreenId::Lifecycle);
    navigation.return_screen = None;
    navigation.selected = 0;
    state.rendered_screen = None;
    state.dirty = false;
    lifecycle.mark_dirty();
}

fn developer_tools_input(
    mut state: ResMut<DeveloperToolsState>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
    mut spatial: ResMut<SpatialExplorationState>,
) {
    let Some(screen) = navigation.current_screen else {
        return;
    };
    if !is_developer_tool_screen(screen) || input_queue.0.is_empty() {
        return;
    }

    for event in std::mem::take(&mut input_queue.0) {
        let count = screen_action_count(screen, &state);
        match event {
            InputEvent::Up => {
                navigation.selected = navigation.selected.saturating_sub(1);
                state.dirty = true;
            }
            InputEvent::Down => {
                navigation.selected = (navigation.selected + 1).min(count.saturating_sub(1));
                state.dirty = true;
            }
            InputEvent::Home => {
                navigation.selected = 0;
                state.dirty = true;
            }
            InputEvent::End => {
                navigation.selected = count.saturating_sub(1);
                state.dirty = true;
            }
            InputEvent::Confirm => {
                match screen {
                    ScreenId::DeveloperTools => match navigation.selected {
                        0 => {
                            navigation.current_screen = Some(ScreenId::CharacterWorkshop);
                            navigation.selected = 0;
                            state.dirty = true;
                        }
                        1 => {
                            if bevy_spatial::sandbox_map(&spatial).is_none() {
                                bevy_spatial::set_sandbox_map(
                                    &mut spatial,
                                    generate_world_lab_map(state.world_lab),
                                );
                                state.world_lab_generated_options = Some(state.world_lab);
                                state.world_lab_status =
                                    Some("A default map is ready to inspect or play-test.".into());
                            }
                            navigation.current_screen = Some(ScreenId::WorldLab);
                            navigation.selected = 0;
                            state.dirty = true;
                        }
                        _ => {
                            return_to_start(&mut state, &mut lifecycle, &mut navigation);
                        }
                    },
                    ScreenId::CharacterWorkshop => match navigation.selected {
                        0 => cycle_current_part(&mut state, -1),
                        1 => cycle_current_part(&mut state, 1),
                        2 => cycle_current_slot(&mut state, -1),
                        3 => cycle_current_slot(&mut state, 1),
                        4 => {
                            let selected_slot = state.selected_slot;
                            if let Some(slot) = state.slots.get_mut(selected_slot) {
                                slot.visible = !slot.visible;
                                state.dirty = true;
                            }
                        }
                        5 => randomize_combination(&mut state),
                        6 => reset_combination(&mut state),
                        7 => {
                            navigation.current_screen = Some(ScreenId::DeveloperTools);
                            navigation.selected = 0;
                            state.dirty = true;
                        }
                        _ => {}
                    },
                    ScreenId::WorldLab => {
                        match navigation.selected {
                            0 => {
                                state.world_lab.seed = state.world_lab.seed.saturating_sub(1);
                                state.world_lab_status = Some(
                                    "Settings changed. Generate a map before play-testing.".into(),
                                );
                            }
                            1 => {
                                state.world_lab.seed = state.world_lab.seed.saturating_add(1);
                                state.world_lab_status = Some(
                                    "Settings changed. Generate a map before play-testing.".into(),
                                );
                            }
                            2 => {
                                state.world_lab.size_index =
                                    (state.world_lab.size_index + 1) % WORLD_LAB_SIZES.len();
                                state.world_lab_status = Some(
                                    "Settings changed. Generate a map before play-testing.".into(),
                                );
                            }
                            3 => {
                                state.world_lab.shrine_layout = !state.world_lab.shrine_layout;
                                state.world_lab_status = Some(
                                    "Settings changed. Generate a map before play-testing.".into(),
                                );
                            }
                            4 => {
                                let map = generate_world_lab_map(state.world_lab);
                                state.world_lab_cursor = Some(map.entry);
                                state.world_lab_held_prop = None;
                                bevy_spatial::set_sandbox_map(&mut spatial, map);
                                state.world_lab_generated_options = Some(state.world_lab);
                                state.world_lab_status =
                                    Some("Map generated with the current settings.".into());
                            }
                            5 => {
                                if let Some(map) = bevy_spatial::sandbox_map(&spatial) {
                                    state.world_lab_cursor = Some(world_lab_cursor(&state, map));
                                    navigation.current_screen = Some(ScreenId::WorldLabEditor);
                                    navigation.selected = 0;
                                } else {
                                    state.world_lab_status =
                                        Some("Generate a map before opening the editor.".into());
                                }
                            }
                            6 => {
                                if state.world_lab_generated_options == Some(state.world_lab)
                                    && bevy_spatial::sandbox_map(&spatial).is_some()
                                {
                                    bevy_spatial::start_sandbox_playtest(
                                        &mut spatial,
                                        &mut navigation,
                                    );
                                } else {
                                    state.world_lab_status = Some(
                                        "Generate a map with the current settings before play-testing."
                                            .into(),
                                    );
                                }
                            }
                            7 => export_world_lab_fixture(&mut state, &spatial),
                            8 => {
                                refresh_world_lab_fixtures(&mut state);
                                navigation.current_screen = Some(ScreenId::WorldLabFixtures);
                                navigation.selected = 0;
                            }
                            9 => {
                                state.world_lab = WorldLabOptions::default();
                                let map = generate_world_lab_map(state.world_lab);
                                state.world_lab_cursor = Some(map.entry);
                                state.world_lab_held_prop = None;
                                bevy_spatial::set_sandbox_map(&mut spatial, map);
                                state.world_lab_generated_options = Some(state.world_lab);
                                state.world_lab_status =
                                    Some("Settings reset and map regenerated.".into());
                            }
                            10 => {
                                navigation.current_screen = Some(ScreenId::DeveloperTools);
                                navigation.selected = 0;
                            }
                            _ => {}
                        }
                        state.dirty = true;
                    }
                    ScreenId::WorldLabFixtures => {
                        let fixture_count = state.world_lab_fixtures.len();
                        if navigation.selected < fixture_count {
                            let selected = navigation.selected;
                            load_world_lab_fixture(
                                &mut state,
                                &mut spatial,
                                &mut navigation,
                                selected,
                            );
                        } else if navigation.selected == fixture_count {
                            import_world_lab_fixture(
                                &mut state,
                                &mut spatial,
                                &mut navigation,
                            );
                        } else if navigation.selected == fixture_count + 1 {
                            refresh_world_lab_fixtures(&mut state);
                            state.world_lab_status = Some(format!(
                                "Refreshed fixture list: {} file(s).",
                                state.world_lab_fixtures.len()
                            ));
                        } else {
                            navigation.current_screen = Some(ScreenId::WorldLab);
                            navigation.selected = 8;
                            state.dirty = true;
                        }
                    }
                    ScreenId::WorldLabEditor => {
                        match navigation.selected {
                            0 => move_world_lab_cursor(&mut state, &spatial, -1, 0),
                            1 => move_world_lab_cursor(&mut state, &spatial, 1, 0),
                            2 => move_world_lab_cursor(&mut state, &spatial, 0, -1),
                            3 => move_world_lab_cursor(&mut state, &spatial, 0, 1),
                            4 => cycle_world_lab_terrain(&mut state, &mut spatial),
                            5 => {
                                state.world_lab_prop_kind = Some(
                                    match state
                                        .world_lab_prop_kind
                                        .unwrap_or(SpatialPropKind::Pillar)
                                    {
                                        SpatialPropKind::Pillar => SpatialPropKind::Table,
                                        SpatialPropKind::Table => SpatialPropKind::Pillar,
                                    },
                                );
                                let kind =
                                    state.world_lab_prop_kind.unwrap_or(SpatialPropKind::Pillar);
                                state.world_lab_status = Some(format!(
                                    "Prop tool set to {}.",
                                    world_lab_prop_name(kind)
                                ));
                            }
                            6 => place_world_lab_prop(&mut state, &mut spatial),
                            7 => pick_up_or_drop_world_lab_prop(&mut state, &mut spatial),
                            8 => remove_world_lab_prop(&mut state, &mut spatial),
                            9 => {
                                navigation.current_screen = Some(ScreenId::WorldLab);
                                navigation.selected = 5;
                            }
                            _ => {}
                        }
                        state.dirty = true;
                    }
                    _ => {}
                }
                break;
            }
            InputEvent::Cancel => {
                match screen {
                    ScreenId::CharacterWorkshop => {
                        navigation.current_screen = Some(ScreenId::DeveloperTools);
                        navigation.selected = 0;
                        state.dirty = true;
                    }
                    ScreenId::DeveloperTools => {
                        return_to_start(&mut state, &mut lifecycle, &mut navigation);
                    }
                    ScreenId::WorldLab => {
                        navigation.current_screen = Some(ScreenId::DeveloperTools);
                        navigation.selected = 0;
                        state.dirty = true;
                    }
                    ScreenId::WorldLabEditor => {
                        navigation.current_screen = Some(ScreenId::WorldLab);
                        navigation.selected = 5;
                        state.dirty = true;
                    }
                    ScreenId::WorldLabFixtures => {
                        navigation.current_screen = Some(ScreenId::WorldLab);
                        navigation.selected = 8;
                        state.dirty = true;
                    }
                    _ => {}
                }
                break;
            }
            _ => {}
        }
    }
}

fn render_hub(commands: &mut Commands, navigation: &NavigationState) {
    let root = bevy_presentation::spawn_screen(commands, "DEVELOPER TOOLS");
    let panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "Visual development workspaces. No campaign or save is required.",
    );

    let workshop = bevy_presentation::spawn_action_button(
        commands,
        panel,
        0,
        "character",
        "Character Workshop",
    );
    if navigation.selected == 0 {
        commands
            .entity(workshop)
            .insert(bevy_presentation::UiSelected);
    }

    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "World Lab creates temporary spaces with the real spatial generator and renderer.",
    );

    let world_lab =
        bevy_presentation::spawn_action_button(commands, panel, 1, "explore", "World Lab");
    if navigation.selected == 1 {
        commands
            .entity(world_lab)
            .insert(bevy_presentation::UiSelected);
    }

    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "Generate deterministic spaces and play-test them in the spatial renderer.",
    );

    let back = bevy_presentation::spawn_action_button(commands, panel, 2, "back", "Back");
    if navigation.selected == 2 {
        commands.entity(back).insert(bevy_presentation::UiSelected);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_character_workshop(
    commands: &mut Commands,
    navigation: &NavigationState,
    state: &DeveloperToolsState,
    images: &mut Assets<Image>,
    cache: &mut PortraitImageCache,
) {
    let root = bevy_presentation::spawn_screen(commands, "CHARACTER WORKSHOP");
    bevy_presentation::spawn_muted_label(
        commands,
        root,
        "Temporary portrait sandbox. Changes never touch campaign characters or content files.",
    );

    let workspace = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            min_width: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(10),
            ..default()
        })
        .id();
    commands.entity(root).add_child(workspace);

    let portrait = build_preview_portrait(&state.slots);
    let detail = format!(
        "Seed {} | {} visible layers",
        state.seed,
        portrait.layers.len()
    );
    bevy_interactions::render_character_portrait(
        commands,
        workspace,
        "ASSEMBLED CHARACTER",
        Some(&detail),
        &portrait,
        Some("No portrait layers are selected."),
        images,
        cache,
    );

    let controls = commands
        .spawn((
            Node {
                width: percent(50),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                padding: UiRect::all(px(8)),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                overflow: Overflow::scroll(),
                ..default()
            },
            BackgroundColor(bevy_presentation::THEME_SURFACE),
            BorderColor::all(bevy_presentation::THEME_BORDER),
        ))
        .id();
    commands.entity(workspace).add_child(controls);

    bevy_presentation::spawn_label(commands, controls, "ASSEMBLY CONTROLS");
    bevy_presentation::spawn_muted_label(
        commands,
        controls,
        "Select a slot, cycle its parts, and toggle its visibility.",
    );

    if let Some(slot) = state.slots.get(state.selected_slot) {
        bevy_presentation::spawn_label(
            commands,
            controls,
            format!(
                "SLOT {}/{}: {}",
                state.selected_slot + 1,
                state.slots.len(),
                slot.name
            ),
        );
        if let Some(part) = slot.parts.get(slot.selected_part) {
            bevy_presentation::spawn_label(commands, controls, format!("PART: {}", part.id));
            bevy_presentation::spawn_muted_label(
                commands,
                controls,
                format!("Asset: {}", part.asset),
            );
            bevy_presentation::spawn_muted_label(
                commands,
                controls,
                format!(
                    "Part {}/{} | Draw layer {} | Weight {}{}",
                    slot.selected_part + 1,
                    slot.parts.len(),
                    part.layer,
                    part.weight,
                    if part.weight == 0 {
                        " (excluded from random generation)"
                    } else {
                        ""
                    }
                ),
            );
            if !part.requires_tags.is_empty() {
                bevy_presentation::spawn_muted_label(
                    commands,
                    controls,
                    format!("Requires tags: {}", part.requires_tags.join(", ")),
                );
            }
            if !part.excludes_tags.is_empty() {
                bevy_presentation::spawn_muted_label(
                    commands,
                    controls,
                    format!("Excludes tags: {}", part.excludes_tags.join(", ")),
                );
            }
        } else {
            bevy_presentation::spawn_muted_label(commands, controls, "This slot has no parts.");
        }
        bevy_presentation::spawn_muted_label(
            commands,
            controls,
            if slot.visible {
                "Layer visibility: visible"
            } else {
                "Layer visibility: hidden"
            },
        );
        bevy_presentation::spawn_label(commands, controls, "AVAILABLE PARTS");
        for (index, part) in slot.parts.iter().enumerate() {
            let marker = if index == slot.selected_part {
                ">"
            } else {
                " "
            };
            bevy_presentation::spawn_muted_label(
                commands,
                controls,
                format!("{marker} {} — {}", part.id, part.asset),
            );
        }
    } else {
        bevy_presentation::spawn_muted_label(
            commands,
            controls,
            "No character parts were found in the loaded base/mod content.",
        );
    }

    bevy_presentation::spawn_label(commands, controls, "CURRENT COMPOSITION");
    let active = build_preview_portrait(&state.slots);
    if active.layers.is_empty() {
        bevy_presentation::spawn_muted_label(commands, controls, "No visible layers.");
    }
    for layer in &active.layers {
        bevy_presentation::spawn_muted_label(
            commands,
            controls,
            format!("{} / {} (layer {})", layer.slot, layer.id, layer.layer),
        );
    }

    let active_assets = active
        .layers
        .iter()
        .map(|layer| layer.asset.as_str())
        .collect::<HashSet<_>>();
    let mut missing = cache
        .missing
        .iter()
        .filter(|asset| active_assets.contains(asset.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    missing.sort();
    if !missing.is_empty() {
        bevy_presentation::spawn_label(commands, controls, "SELECTED ASSET LOAD FAILURES");
        for asset in missing {
            let detail = cache
                .failures
                .get(&asset)
                .map(|error| format!("{asset}: {error}"))
                .unwrap_or(asset);
            bevy_presentation::spawn_muted_label(commands, controls, detail);
        }
    }

    if !state.load_warnings.is_empty() {
        bevy_presentation::spawn_label(commands, controls, "CONTENT DIAGNOSTICS");
        for warning in state.load_warnings.iter().take(3) {
            bevy_presentation::spawn_muted_label(commands, controls, warning);
        }
    }

    for (index, (icon, label)) in WORKSHOP_ACTIONS.iter().enumerate() {
        let button =
            bevy_presentation::spawn_action_button(commands, controls, index, *icon, *label);
        if navigation.selected == index {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }
}

fn render_world_lab(
    commands: &mut Commands,
    navigation: &NavigationState,
    state: &DeveloperToolsState,
    spatial: &SpatialExplorationState,
) {
    let root = bevy_presentation::spawn_screen(commands, "WORLD LAB");
    bevy_presentation::spawn_muted_label(
        commands,
        root,
        "Disposable spatial sandbox. Nothing here is written to campaign worlds or saves.",
    );

    let controls = bevy_presentation::spawn_panel(commands, root);
    let (width, height) =
        WORLD_LAB_SIZES[state.world_lab.size_index.min(WORLD_LAB_SIZES.len() - 1)];
    bevy_presentation::spawn_label(commands, controls, "GENERATION SETTINGS");
    bevy_presentation::spawn_muted_label(
        commands,
        controls,
        format!("World seed: {}", state.world_lab.seed),
    );
    bevy_presentation::spawn_muted_label(
        commands,
        controls,
        format!("Map dimensions: {width} × {height}"),
    );
    bevy_presentation::spawn_muted_label(
        commands,
        controls,
        format!(
            "Spatial layout: {}",
            if state.world_lab.shrine_layout {
                "Old Shrine chamber"
            } else {
                "Procedural"
            }
        ),
    );

    let map = bevy_spatial::sandbox_map(spatial);
    let map_is_current =
        state.world_lab_generated_options == Some(state.world_lab) && map.is_some();
    if let Some(map) = map {
        let tile_count = usize::from(map.width) * usize::from(map.height);
        let walls = map
            .terrain
            .iter()
            .filter(|terrain| **terrain == Terrain::Wall)
            .count();
        bevy_presentation::spawn_muted_label(
            commands,
            controls,
            format!(
                "Generated map seed: {} | {} tiles | {} walls | {} props | {} points of interest",
                map.seed,
                tile_count,
                walls,
                map.props.len(),
                map.points_of_interest.len()
            ),
        );
    } else {
        bevy_presentation::spawn_muted_label(commands, controls, "No map generated yet.");
    }
    if !map_is_current {
        bevy_presentation::spawn_muted_label(
            commands,
            controls,
            "The preview does not match the current settings. Generate a map before play-testing.",
        );
    }
    if let Some(status) = state.world_lab_status.as_deref() {
        bevy_presentation::spawn_muted_label(commands, controls, status);
    }

    for (index, (icon, label)) in WORLD_LAB_ACTIONS.iter().enumerate() {
        let button =
            bevy_presentation::spawn_action_button(commands, controls, index, *icon, *label);
        if navigation.selected == index {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }

    let preview_panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_label(commands, preview_panel, "MAP PREVIEW");
    if let Some(map) = map {
        bevy_presentation::spawn_muted_label(
            commands,
            preview_panel,
            "Legend: @ entry · ? point of interest · P prop · # wall · . floor · : ash · % rubble",
        );
        if map_is_current {
            bevy_presentation::spawn_muted_label(
                commands,
                preview_panel,
                world_lab_map_preview(map),
            );
        } else {
            bevy_presentation::spawn_muted_label(
                commands,
                preview_panel,
                "Previous generated map. Update it with Generate Map after changing settings.",
            );
            bevy_presentation::spawn_muted_label(
                commands,
                preview_panel,
                world_lab_map_preview(map),
            );
        }
    } else {
        bevy_presentation::spawn_muted_label(
            commands,
            preview_panel,
            "Choose Generate Map to create a reproducible spatial layout.",
        );
    }
}

fn render_world_lab_editor(
    commands: &mut Commands,
    navigation: &NavigationState,
    state: &DeveloperToolsState,
    spatial: &SpatialExplorationState,
) {
    let root = bevy_presentation::spawn_screen(commands, "WORLD LAB · MAP EDITOR");
    bevy_presentation::spawn_muted_label(
        commands,
        root,
        "Changes affect only this temporary map. Entry, actor, passage, discovery, and prop cells are protected.",
    );

    let controls = bevy_presentation::spawn_panel(commands, root);
    if let Some(map) = bevy_spatial::sandbox_map(spatial) {
        let cursor = world_lab_cursor(state, map);
        bevy_presentation::spawn_label(commands, controls, "SELECTED CELL");
        bevy_presentation::spawn_muted_label(
            commands,
            controls,
            world_lab_cell_details(map, cursor),
        );
        let prop_kind = state.world_lab_prop_kind.unwrap_or(SpatialPropKind::Pillar);
        bevy_presentation::spawn_muted_label(
            commands,
            controls,
            format!("Prop tool: {}", world_lab_prop_name(prop_kind)),
        );
        if let Some(held_id) = state.world_lab_held_prop.as_deref() {
            if let Some(prop) = map.props.iter().find(|prop| prop.id == held_id) {
                bevy_presentation::spawn_muted_label(
                    commands,
                    controls,
                    format!(
                        "Moving {} from ({}, {}). Move the cursor, then choose Pick up / drop prop.",
                        world_lab_prop_name(prop.kind), prop.position.x, prop.position.y
                    ),
                );
            }
        }
    } else {
        bevy_presentation::spawn_muted_label(commands, controls, "No temporary map is available.");
    }
    if let Some(status) = state.world_lab_status.as_deref() {
        bevy_presentation::spawn_muted_label(commands, controls, status);
    }
    for (index, (icon, label)) in WORLD_LAB_EDITOR_ACTIONS.iter().enumerate() {
        let button =
            bevy_presentation::spawn_action_button(commands, controls, index, *icon, *label);
        if navigation.selected == index {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }

    let preview_panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_label(commands, preview_panel, "EDIT PREVIEW");
    if let Some(map) = bevy_spatial::sandbox_map(spatial) {
        let cursor = world_lab_cursor(state, map);
        bevy_presentation::spawn_muted_label(
            commands, preview_panel,
            "Legend: X cursor · H moving prop · @ entry · ? discovery · P prop · # wall · . floor · : ash · % rubble",
        );
        bevy_presentation::spawn_muted_label(
            commands,
            preview_panel,
            world_lab_map_preview_with_cursor(
                map,
                Some(cursor),
                state.world_lab_held_prop.as_deref(),
            ),
        );
    } else {
        bevy_presentation::spawn_muted_label(commands, preview_panel, "Generate a map first.");
    }
}

fn render_world_lab_fixtures(
    commands: &mut Commands,
    navigation: &NavigationState,
    state: &DeveloperToolsState,
) {
    let root = bevy_presentation::spawn_screen(commands, "WORLD LAB · FIXTURES");
    bevy_presentation::spawn_muted_label(
        commands,
        root,
        "Reusable spatial maps are stored separately from campaign saves. Import validates a fixture before it replaces the temporary sandbox.",
    );
    let panel = bevy_presentation::spawn_panel(commands, root);
    if state.world_lab_fixtures.is_empty() {
        bevy_presentation::spawn_muted_label(
            commands,
            panel,
            "No fixtures found. Save the current map or import a .json fixture.",
        );
    }
    for (index, fixture) in state.world_lab_fixtures.iter().enumerate() {
        let button = bevy_presentation::spawn_action_button_with_detail(
            commands,
            panel,
            index,
            if fixture.error.is_some() { "cancel" } else { "folder" },
            &fixture.name,
            &fixture.details,
        );
        if navigation.selected == index {
            commands
                .entity(button)
                .insert(bevy_presentation::UiSelected);
        }
    }

    let import_index = state.world_lab_fixtures.len();
    let import = bevy_presentation::spawn_action_button(
        commands,
        panel,
        import_index,
        "folder",
        "Import fixture",
    );
    if navigation.selected == import_index {
        commands.entity(import).insert(bevy_presentation::UiSelected);
    }
    let refresh_index = import_index + 1;
    let refresh = bevy_presentation::spawn_action_button(
        commands,
        panel,
        refresh_index,
        "continue",
        "Refresh fixture list",
    );
    if navigation.selected == refresh_index {
        commands.entity(refresh).insert(bevy_presentation::UiSelected);
    }
    let back_index = refresh_index + 1;
    let back = bevy_presentation::spawn_action_button(
        commands,
        panel,
        back_index,
        "back",
        "Back to World Lab",
    );
    if navigation.selected == back_index {
        commands.entity(back).insert(bevy_presentation::UiSelected);
    }
    if let Some(status) = state.world_lab_status.as_deref() {
        bevy_presentation::spawn_muted_label(commands, panel, status);
    }
}

fn render_if_active(
    mut commands: Commands,
    navigation: Res<NavigationState>,
    mut state: ResMut<DeveloperToolsState>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<PortraitImageCache>,
    spatial: Res<SpatialExplorationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    let Some(screen) = navigation
        .current_screen
        .filter(|screen| is_developer_tool_screen(*screen))
    else {
        state.rendered_screen = None;
        return;
    };
    if state.rendered_screen == Some(screen) && !state.dirty && !spatial.is_changed() {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }
    match screen {
        ScreenId::DeveloperTools => render_hub(&mut commands, &navigation),
        ScreenId::CharacterWorkshop => {
            render_character_workshop(&mut commands, &navigation, &state, &mut images, &mut cache)
        }
        ScreenId::WorldLab => render_world_lab(&mut commands, &navigation, &state, &spatial),
        ScreenId::WorldLabEditor => {
            render_world_lab_editor(&mut commands, &navigation, &state, &spatial)
        }
        ScreenId::WorldLabFixtures => {
            render_world_lab_fixtures(&mut commands, &navigation, &state)
        }
        _ => {}
    }
    state.rendered_screen = Some(screen);
    state.dirty = false;
}

#[cfg(test)]
#[path = "../tests/unit/bevy_dev_tools.rs"]
mod tests;
