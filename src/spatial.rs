//! Renderer-neutral data and deterministic generation for local spatial maps.
//!
//! This module intentionally contains no Bevy types so map generation and state
//! can be tested independently from presentation.

use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use crate::rng::DeterministicRng;

pub const LOCAL_MAP_GENERATOR_VERSION: u32 = 3;
pub const DEFAULT_MAP_WIDTH: u16 = 24;
pub const DEFAULT_MAP_HEIGHT: u16 = 16;
const MIN_MAP_DIMENSION: u16 = 8;
const MAX_MAP_DIMENSION: u16 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    Floor,
    Ash,
    Rubble,
    Wall,
}

impl Terrain {
    pub fn is_walkable(self) -> bool {
        !matches!(self, Self::Wall)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: u16,
    pub y: u16,
}

/// The kind of persistent world actor occupying a local-map interaction point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialEntityKind {
    Npc,
    Corpse,
}

/// Stable placement for a world-owned actor in a character's local environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialEntityPlacement {
    pub entity_id: u64,
    pub kind: SpatialEntityKind,
    pub position: GridPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointOfInterest {
    /// Stable content identifier, independent of display text.
    pub id: String,
    pub position: GridPosition,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub discovered: bool,
    #[serde(default)]
    pub interacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainChange {
    pub position: GridPosition,
    pub terrain: Terrain,
}

/// Static environmental props supported by the first spatial-rendering slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialPropKind {
    Pillar,
    Table,
}

impl SpatialPropKind {
    /// Footprint width measured in local-map cells; props occupy one grid row.
    pub fn footprint_width(self) -> u16 {
        match self {
            Self::Pillar => 1,
            Self::Table => 2,
        }
    }
}

/// Persistent prop anchor. A table's position is the left-most cell of its
/// two-cell footprint; a pillar occupies its single anchor cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialProp {
    pub id: String,
    pub kind: SpatialPropKind,
    pub position: GridPosition,
}

impl SpatialProp {
    pub fn occupies(&self, position: GridPosition) -> bool {
        position.y == self.position.y
            && position.x >= self.position.x
            && position.x < self.position.x.saturating_add(self.kind.footprint_width())
    }

    pub fn footprint_contains(&self, position: GridPosition, map_width: u16) -> bool {
        self.occupies(position) && position.x < map_width
    }
}

/// A physical passage at the edge of a local map. The destination uses the
/// persistent world location ID so it can later connect directly to the world
/// graph without relying on display names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapExit {
    /// Stable within this local map and independent of destination display text.
    pub id: String,
    pub position: GridPosition,
    pub target_location_id: u64,
}

/// Persistent local-map state. Once generated, the stored terrain is the source
/// of truth; loading a map must not regenerate it with a newer generator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMap {
    /// Stable campaign/content location ID, not its display name.
    pub location_id: String,
    pub generator_version: u32,
    /// Built-in layout used when the persistent map was first generated.
    /// Missing in older saves; the saved geometry itself remains authoritative.
    #[serde(default)]
    pub layout_id: Option<String>,
    pub seed: u64,
    pub width: u16,
    pub height: u16,
    /// Row-major terrain, with exactly width * height entries.
    pub terrain: Vec<Terrain>,
    pub entry: GridPosition,
    pub points_of_interest: Vec<PointOfInterest>,
    /// Connected world locations reachable through physical map-edge passages.
    /// Defaults empty so maps in older saves remain loadable.
    #[serde(default)]
    pub exits: Vec<MapExit>,
    #[serde(default)]
    pub discovered_tiles: Vec<bool>,
    #[serde(default)]
    pub terrain_changes: Vec<TerrainChange>,
    /// Persistent static props and their collision footprints. Missing in older saves.
    #[serde(default)]
    pub props: Vec<SpatialProp>,
    /// Persistent positions of world-owned NPCs and searchable remains.
    /// Missing in older saves; placements are added deterministically on entry.
    #[serde(default)]
    pub interactables: Vec<SpatialEntityPlacement>,
}

impl LocalMap {
    pub fn contains(&self, position: GridPosition) -> bool {
        position.x < self.width && position.y < self.height
    }

    pub fn terrain_at(&self, position: GridPosition) -> Option<Terrain> {
        if !self.contains(position) {
            return None;
        }
        self.terrain
            .get(usize::from(position.y) * usize::from(self.width) + usize::from(position.x))
            .copied()
    }

    pub fn is_walkable(&self, position: GridPosition) -> bool {
        self.terrain_at(position)
            .map(Terrain::is_walkable)
            .unwrap_or(false)
            && !self
                .props
                .iter()
                .any(|prop| prop.footprint_contains(position, self.width))
    }

    pub fn set_terrain(&mut self, position: GridPosition, terrain: Terrain) -> bool {
        if !self.contains(position) {
            return false;
        }
        let index = usize::from(position.y) * usize::from(self.width) + usize::from(position.x);
        let Some(cell) = self.terrain.get_mut(index) else {
            return false;
        };
        *cell = terrain;
        if let Some(change) = self
            .terrain_changes
            .iter_mut()
            .find(|change| change.position == position)
        {
            change.terrain = terrain;
        } else {
            self.terrain_changes
                .push(TerrainChange { position, terrain });
        }
        true
    }

    pub fn mark_tile_discovered(&mut self, position: GridPosition) -> bool {
        if !self.contains(position) {
            return false;
        }
        let expected_len = usize::from(self.width) * usize::from(self.height);
        if self.discovered_tiles.len() != expected_len {
            self.discovered_tiles.resize(expected_len, false);
        }
        let index = usize::from(position.y) * usize::from(self.width) + usize::from(position.x);
        self.discovered_tiles[index] = true;
        true
    }

    pub fn is_tile_discovered(&self, position: GridPosition) -> bool {
        if !self.contains(position) {
            return false;
        }
        let index = usize::from(position.y) * usize::from(self.width) + usize::from(position.x);
        self.discovered_tiles.get(index).copied().unwrap_or(false)
    }

    pub fn point_of_interest(&self, id: &str) -> Option<&PointOfInterest> {
        self.points_of_interest.iter().find(|poi| poi.id == id)
    }

    pub fn point_of_interest_mut(&mut self, id: &str) -> Option<&mut PointOfInterest> {
        self.points_of_interest.iter_mut().find(|poi| poi.id == id)
    }
}

/// Character-specific state for the currently active local exploration session.
/// This is deliberately separate from LocalMap, which belongs to the world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplorationSession {
    pub location_id: String,
    pub position: GridPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalMapConfig {
    pub width: u16,
    pub height: u16,
}

impl Default for LocalMapConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_MAP_WIDTH,
            height: DEFAULT_MAP_HEIGHT,
        }
    }
}

impl LocalMapConfig {
    fn normalized(self) -> Self {
        Self {
            width: self.width.clamp(MIN_MAP_DIMENSION, MAX_MAP_DIMENSION),
            height: self.height.clamp(MIN_MAP_DIMENSION, MAX_MAP_DIMENSION),
        }
    }
}

/// Generate a deterministic bounded map from a world seed and stable location
/// identity. A guaranteed corridor connects the entry and the initial point of
/// interest, even when random terrain would otherwise block the route.
pub fn generate_local_map(
    world_seed: u64,
    location_id: impl Into<String>,
    config: LocalMapConfig,
) -> LocalMap {
    generate_local_map_with_layout(world_seed, location_id, config, None)
}

/// Generate a local map using an optional content-selected built-in layout.
/// Unknown layout identifiers safely fall back to procedural terrain and props.
pub fn generate_local_map_with_layout(
    world_seed: u64,
    location_id: impl Into<String>,
    config: LocalMapConfig,
    layout: Option<&str>,
) -> LocalMap {
    let location_id = location_id.into();
    let config = config.normalized();
    let seed = local_map_seed(world_seed, &location_id);
    let mut rng = DeterministicRng::new(seed);
    let width = config.width;
    let height = config.height;
    let layout_id =
        if layout == Some("shrine_chamber") && width >= DEFAULT_MAP_WIDTH && height >= 12 {
            Some("shrine_chamber".to_string())
        } else {
            None
        };
    let entry = GridPosition {
        x: 1,
        y: height / 2,
    };
    let poi_position = GridPosition {
        x: width - 2,
        y: 1 + rng.gen_range(0..usize::from(height - 2)) as u16,
    };
    let mut terrain = Vec::with_capacity(usize::from(width) * usize::from(height));

    for y in 0..height {
        for x in 0..width {
            let position = GridPosition { x, y };
            let tile = if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                Terrain::Wall
            } else {
                match rng.gen_range(0..12) {
                    0 | 1 => Terrain::Wall,
                    2 | 3 => Terrain::Rubble,
                    4 | 5 => Terrain::Ash,
                    _ => Terrain::Floor,
                }
            };
            let index = usize::from(y) * usize::from(width) + usize::from(x);
            debug_assert_eq!(terrain.len(), index);
            terrain.push(if position == entry || position == poi_position {
                Terrain::Floor
            } else {
                tile
            });
        }
    }

    // Carve an L-shaped corridor, making the first discovery reachable for all
    // generated dimensions and seeds.
    let corridor_y = entry.y;
    for x in entry.x..=poi_position.x {
        let index = usize::from(corridor_y) * usize::from(width) + usize::from(x);
        terrain[index] = Terrain::Floor;
    }
    let (from_y, to_y) = if corridor_y <= poi_position.y {
        (corridor_y, poi_position.y)
    } else {
        (poi_position.y, corridor_y)
    };
    for y in from_y..=to_y {
        let index = usize::from(y) * usize::from(width) + usize::from(poi_position.x);
        terrain[index] = Terrain::Floor;
    }

    let discovered_tiles = vec![false; terrain.len()];
    let mut map = LocalMap {
        location_id: location_id.clone(),
        generator_version: LOCAL_MAP_GENERATOR_VERSION,
        layout_id,
        seed,
        width,
        height,
        terrain,
        entry,
        points_of_interest: vec![PointOfInterest {
            id: format!("{location_id}.poi.initial_discovery"),
            position: poi_position,
            title: "Unsettling Remains".to_string(),
            description: "Something here resists the passage of time.".to_string(),
            discovered: false,
            interacted: false,
        }],
        exits: Vec::new(),
        discovered_tiles,
        terrain_changes: Vec::new(),
        props: Vec::new(),
        interactables: Vec::new(),
    };
    populate_generated_props(&mut map, &mut rng);
    if map.layout_id.as_deref() == Some("shrine_chamber") {
        apply_shrine_chamber(&mut map);
    }
    map
}

/// Overlay the first authored room template. The chamber is intentionally
/// compact and sits above the guaranteed east-west corridor, so entry-to-POI
/// and edge-passage routes remain clear. Layout geometry is renderer-neutral.
fn apply_shrine_chamber(map: &mut LocalMap) {
    // The initial template is designed for the standard 24x16 prototype map.
    if map.width < DEFAULT_MAP_WIDTH || map.height < 12 {
        return;
    }

    let left = 2_u16;
    let right = 7_u16;
    let top = 2_u16;
    let bottom = 7_u16;
    let doorway_x = 5_u16;

    // Remove random props that overlap any part of the authored room.
    map.props.retain(|prop| {
        let prop_right_exclusive = prop.position.x.saturating_add(prop.kind.footprint_width());
        !(prop.position.y >= top
            && prop.position.y <= bottom
            && prop.position.x <= right
            && prop_right_exclusive > left)
    });

    for y in top..=bottom {
        for x in left..=right {
            let position = GridPosition { x, y };
            let doorway = position.x == doorway_x && position.y == bottom;
            let boundary = x == left || x == right || y == top || y == bottom;
            let terrain = if doorway {
                Terrain::Floor
            } else if boundary {
                Terrain::Wall
            } else {
                Terrain::Floor
            };
            let index = usize::from(y) * usize::from(map.width) + usize::from(x);
            map.terrain[index] = terrain;
        }
    }

    map.props.push(SpatialProp {
        id: format!("{}.room.pillar", map.location_id),
        kind: SpatialPropKind::Pillar,
        position: GridPosition { x: 3, y: 5 },
    });
    map.props.push(SpatialProp {
        id: format!("{}.room.table", map.location_id),
        kind: SpatialPropKind::Table,
        position: GridPosition { x: 5, y: 5 },
    });
}

/// Add a sparse deterministic set of props outside the guaranteed
/// entry-to-discovery route. Props are persisted and are not regenerated for
/// an already established local map.
fn populate_generated_props(map: &mut LocalMap, rng: &mut DeterministicRng) {
    let target_count = (usize::from(map.width) * usize::from(map.height) / 96).clamp(1, 5);

    for index in 0..target_count {
        let mut kind = if index % 2 == 0 {
            SpatialPropKind::Pillar
        } else {
            SpatialPropKind::Table
        };
        let mut candidates = prop_candidates(map, kind);
        if candidates.is_empty() && kind == SpatialPropKind::Table {
            kind = SpatialPropKind::Pillar;
            candidates = prop_candidates(map, kind);
        }
        if candidates.is_empty() {
            break;
        }

        let position = candidates[rng.gen_range(0..candidates.len())];
        map.props.push(SpatialProp {
            id: format!("{}.prop.{index}", map.location_id),
            kind,
            position,
        });
    }
}

fn reachable_walkable_tiles(map: &LocalMap) -> HashSet<GridPosition> {
    let mut reachable = HashSet::new();
    if !map.is_walkable(map.entry) {
        return reachable;
    }

    let mut pending = VecDeque::from([map.entry]);
    reachable.insert(map.entry);
    while let Some(position) = pending.pop_front() {
        for (dx, dy) in [(0_i32, -1_i32), (1, 0), (0, 1), (-1, 0)] {
            let x = i32::from(position.x) + dx;
            let y = i32::from(position.y) + dy;
            if x < 0 || y < 0 || x >= i32::from(map.width) || y >= i32::from(map.height) {
                continue;
            }
            let neighbour = GridPosition {
                x: x as u16,
                y: y as u16,
            };
            if map.is_walkable(neighbour) && reachable.insert(neighbour) {
                pending.push_back(neighbour);
            }
        }
    }
    reachable
}

/// Add missing NPC/corpse placements without moving any placement already
/// persisted on the local map. Candidate selection is stable for a map seed
/// and entity ID, and avoids terrain, props, exits, points of interest, and
/// other placed entities. Only cells reachable from the entry are considered.
pub fn ensure_spatial_entity_placements(map: &mut LocalMap, entities: &[(u64, SpatialEntityKind)]) {
    let mut entities = entities.to_vec();
    entities.sort_by_key(|(entity_id, kind)| {
        (*entity_id, matches!(*kind, SpatialEntityKind::Corpse) as u8)
    });

    let reachable = reachable_walkable_tiles(map);
    for (entity_id, kind) in entities {
        if map
            .interactables
            .iter()
            .any(|placement| placement.entity_id == entity_id && placement.kind == kind)
        {
            continue;
        }

        let candidates = (1..map.height.saturating_sub(1))
            .flat_map(|y| (1..map.width.saturating_sub(1)).map(move |x| GridPosition { x, y }))
            .filter(|position| reachable.contains(position))
            .filter(|position| {
                position.x.abs_diff(map.entry.x) + position.y.abs_diff(map.entry.y) >= 2
            })
            .filter(|position| {
                !map.points_of_interest
                    .iter()
                    .any(|poi| poi.position == *position)
            })
            .filter(|position| !map.exits.iter().any(|exit| exit.position == *position))
            .filter(|position| {
                !map.interactables
                    .iter()
                    .any(|placement| placement.position == *position)
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            continue;
        }
        let start = (map.seed.wrapping_add(entity_id.wrapping_mul(0x9E37_79B9))
            % candidates.len() as u64) as usize;
        let position = candidates[start];
        map.interactables.push(SpatialEntityPlacement {
            entity_id,
            kind,
            position,
        });
    }
}

fn prop_candidates(map: &LocalMap, kind: SpatialPropKind) -> Vec<GridPosition> {
    let poi = map.points_of_interest.first().map(|poi| poi.position);
    let mut candidates = Vec::new();

    for y in 1..map.height.saturating_sub(1) {
        for x in 1..map.width.saturating_sub(1) {
            let last_x = x.saturating_add(kind.footprint_width().saturating_sub(1));
            if last_x >= map.width.saturating_sub(1) {
                continue;
            }

            let footprint = (x..=last_x)
                .map(|cell_x| GridPosition { x: cell_x, y })
                .collect::<Vec<_>>();
            let conflicts_with_route = footprint
                .iter()
                .any(|cell| reserved_discovery_corridor(map.entry, poi, *cell));
            let overlaps_prop = footprint.iter().any(|cell| {
                map.props
                    .iter()
                    .any(|prop| prop.footprint_contains(*cell, map.width))
            });
            let terrain_walkable = footprint
                .iter()
                .all(|cell| map.terrain_at(*cell).is_some_and(Terrain::is_walkable));

            if terrain_walkable && !conflicts_with_route && !overlaps_prop {
                candidates.push(GridPosition { x, y });
            }
        }
    }
    candidates
}

fn reserved_discovery_corridor(
    entry: GridPosition,
    poi: Option<GridPosition>,
    position: GridPosition,
) -> bool {
    let Some(poi) = poi else {
        return false;
    };
    let horizontal = position.y == entry.y && position.x >= entry.x && position.x <= poi.x;
    let (first_y, last_y) = if entry.y <= poi.y {
        (entry.y, poi.y)
    } else {
        (poi.y, entry.y)
    };
    let vertical = position.x == poi.x && position.y >= first_y && position.y <= last_y;
    horizontal || vertical
}

/// Place a physical map-edge passage for each connected world location.
///
/// Passages are assigned deterministically to east, west, north, and south
/// edges. Each gets a clear walkable corridor to the entry area. Exit targets
/// are world IDs, so destination danger state does not affect passage identity.
pub fn configure_map_exits(map: &mut LocalMap, target_location_ids: &[u64]) {
    let mut targets = Vec::with_capacity(target_location_ids.len());
    for target in target_location_ids {
        if !targets.contains(target) {
            targets.push(*target);
        }
    }

    map.exits.clear();
    for (index, target_location_id) in targets.iter().copied().enumerate() {
        let side = index % 4;
        let slot = index / 4;
        let side_count = targets.iter().skip(side).step_by(4).count();
        let (x, y) = match side {
            // East and west exits are spaced along the vertical edge.
            0 => (
                map.width - 1,
                spaced_edge_coordinate(slot, side_count, map.height),
            ),
            1 => (0, spaced_edge_coordinate(slot, side_count, map.height)),
            // North and south exits are spaced along the horizontal edge.
            2 => (spaced_edge_coordinate(slot, side_count, map.width), 0),
            _ => (
                spaced_edge_coordinate(slot, side_count, map.width),
                map.height - 1,
            ),
        };
        let position = GridPosition { x, y };
        carve_exit_corridor(map, position, side);
        map.exits.push(MapExit {
            id: format!("{}.exit.{}", map.location_id, target_location_id),
            position,
            target_location_id,
        });
    }
}

fn spaced_edge_coordinate(slot: usize, count: usize, dimension: u16) -> u16 {
    let usable = usize::from(dimension.saturating_sub(2));
    (1 + (slot + 1) * usable / (count + 1)) as u16
}

/// Route passages around the authored chamber so extra graph connections do
/// not punch arbitrary holes through its walls or remove its furniture.
fn carve_shrine_chamber_exit_corridor(map: &mut LocalMap, exit: GridPosition, side: usize) -> bool {
    let (left, right, top, bottom) = (2_u16, 7_u16, 2_u16, 7_u16);
    let bypass_x = right + 1;
    let entry = map.entry;
    let mut position = exit;
    carve_generated_floor(map, position);

    match side {
        // East passages aligned with the chamber detour around its east wall.
        0 if (top..=bottom).contains(&exit.y) => {
            let row = position.y;
            carve_toward(
                map,
                &mut position,
                GridPosition {
                    x: bypass_x,
                    y: row,
                },
            );
            carve_toward(
                map,
                &mut position,
                GridPosition {
                    x: bypass_x,
                    y: entry.y,
                },
            );
            carve_toward(map, &mut position, entry);
            true
        }
        // North/south passages aligned with the room bypass its right side.
        2 if (left..=right).contains(&exit.x) => {
            let column = position.x;
            carve_toward(map, &mut position, GridPosition { x: column, y: 1 });
            carve_toward(map, &mut position, GridPosition { x: bypass_x, y: 1 });
            carve_toward(
                map,
                &mut position,
                GridPosition {
                    x: bypass_x,
                    y: entry.y,
                },
            );
            carve_toward(map, &mut position, entry);
            true
        }
        3 if (left..=right).contains(&exit.x) => {
            let column = position.x;
            let bypass_y = map.height - 2;
            carve_toward(
                map,
                &mut position,
                GridPosition {
                    x: column,
                    y: bypass_y,
                },
            );
            carve_toward(
                map,
                &mut position,
                GridPosition {
                    x: bypass_x,
                    y: bypass_y,
                },
            );
            carve_toward(
                map,
                &mut position,
                GridPosition {
                    x: bypass_x,
                    y: entry.y,
                },
            );
            carve_toward(map, &mut position, entry);
            true
        }
        _ => false,
    }
}

fn carve_toward(map: &mut LocalMap, position: &mut GridPosition, target: GridPosition) {
    while position.x != target.x {
        position.x = if position.x > target.x {
            position.x - 1
        } else {
            position.x + 1
        };
        carve_generated_floor(map, *position);
    }
    while position.y != target.y {
        position.y = if position.y > target.y {
            position.y - 1
        } else {
            position.y + 1
        };
        carve_generated_floor(map, *position);
    }
}

fn carve_exit_corridor(map: &mut LocalMap, exit: GridPosition, side: usize) {
    if map.layout_id.as_deref() == Some("shrine_chamber")
        && carve_shrine_chamber_exit_corridor(map, exit, side)
    {
        return;
    }

    let mut position = exit;
    carve_generated_floor(map, position);
    match side {
        // Reach the entry column before joining the central entry corridor.
        0 | 1 => {
            while position.x != map.entry.x {
                position.x = if position.x > map.entry.x {
                    position.x - 1
                } else {
                    position.x + 1
                };
                carve_generated_floor(map, position);
            }
            while position.y != map.entry.y {
                position.y = if position.y > map.entry.y {
                    position.y - 1
                } else {
                    position.y + 1
                };
                carve_generated_floor(map, position);
            }
        }
        // North/south passages first move inward, then join the entry corridor.
        _ => {
            while position.y != map.entry.y {
                position.y = if position.y > map.entry.y {
                    position.y - 1
                } else {
                    position.y + 1
                };
                carve_generated_floor(map, position);
            }
            while position.x != map.entry.x {
                position.x = if position.x > map.entry.x {
                    position.x - 1
                } else {
                    position.x + 1
                };
                carve_generated_floor(map, position);
            }
        }
    }
}

fn carve_generated_floor(map: &mut LocalMap, position: GridPosition) {
    // Generated exit corridors are authoritative; remove any obstacle whose
    // footprint would overlap a required corridor cell.
    let map_width = map.width;
    map.props
        .retain(|prop| !prop.footprint_contains(position, map_width));
    let index = usize::from(position.y) * usize::from(map.width) + usize::from(position.x);
    if let Some(terrain) = map.terrain.get_mut(index) {
        *terrain = Terrain::Floor;
    }
}

fn local_map_seed(world_seed: u64, location_id: &str) -> u64 {
    // FNV-1a over the stable location ID, mixed with the campaign seed. This
    // avoids dependence on process-randomized hashers or display names.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in location_id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash ^ world_seed.rotate_left(23)
}

/// Return whether a walkable path exists between two map positions.
pub fn has_walkable_path(map: &LocalMap, start: GridPosition, goal: GridPosition) -> bool {
    if !map.is_walkable(start) || !map.is_walkable(goal) {
        return false;
    }
    let mut visited = vec![false; usize::from(map.width) * usize::from(map.height)];
    let index_of = |position: GridPosition| {
        usize::from(position.y) * usize::from(map.width) + usize::from(position.x)
    };
    let mut queue = VecDeque::new();
    visited[index_of(start)] = true;
    queue.push_back(start);

    while let Some(position) = queue.pop_front() {
        if position == goal {
            return true;
        }
        let neighbors = [
            position
                .x
                .checked_sub(1)
                .map(|x| GridPosition { x, y: position.y }),
            position
                .x
                .checked_add(1)
                .filter(|x| *x < map.width)
                .map(|x| GridPosition { x, y: position.y }),
            position
                .y
                .checked_sub(1)
                .map(|y| GridPosition { x: position.x, y }),
            position
                .y
                .checked_add(1)
                .filter(|y| *y < map.height)
                .map(|y| GridPosition { x: position.x, y }),
        ];
        for neighbor in neighbors.into_iter().flatten() {
            let index = index_of(neighbor);
            if !visited[index] && map.is_walkable(neighbor) {
                visited[index] = true;
                queue.push_back(neighbor);
            }
        }
    }
    false
}

#[cfg(test)]
#[path = "../tests/unit/spatial.rs"]
mod tests;
