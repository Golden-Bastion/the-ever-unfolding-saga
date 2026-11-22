//! Deterministic, bounded terrain-aware planning for persistent world routes.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap};

use serde::{Deserialize, Serialize};

use super::hydrology::SurfaceWaterKind;
use super::rivers::{self, RiverFlowDirection, RiverId, RiverWidthClass};
use super::{hydrology, terrain, waterbodies, watershed, MacroCoordinate};
use crate::model::World;

pub const MAX_CORRIDOR_SAMPLES: usize = 16;
pub const ROUTE_GRID_SPACING: i64 = 32;
pub const MAX_ROUTE_SEARCH_NODES: usize = 4_096;
pub const MAX_ROUTE_GRID_DISTANCE: i32 = 96;
pub const MAX_ROUTE_PATH_POINTS: usize = 256;
const SEARCH_MARGIN_CELLS: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RouteCorridorAssessment {
    pub sampled_points: u8,
    pub undersampled: bool,
    pub ocean_samples: u8,
    pub lake_samples: u8,
    pub river_samples: u8,
    pub unresolved_water_samples: u8,
    pub mountain_barrier_samples: u8,
    pub max_mountain_ridge_strength_q16: u32,
}

impl RouteCorridorAssessment {
    pub const fn has_confirmed_water_crossing(self) -> bool {
        self.ocean_samples > 0 || self.lake_samples > 0 || self.river_samples > 0
    }

    pub const fn has_unresolved_water(self) -> bool {
        self.unresolved_water_samples > 0
    }
}

/// Broad semantic classification attached to a route and its local-map exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RouteProfile {
    #[default]
    Unavailable,
    Trail,
    MountainPass,
    RiverCrossing,
    WaterCrossing,
    Mixed,
    Inconclusive,
}

/// Result of trying to construct a land-based route between two surface anchors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoutePlanningStatus {
    #[default]
    MissingEndpoint,
    Planned,
    InvalidEndpoint,
    UnsupportedStratum,
    NoLandPathInSearchWindow,
    SearchLimit,
}

/// Physical features a planned route encounters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteCrossingKind {
    MountainPass,
    River,
    Lake,
    Ocean,
    UnresolvedWater,
}

/// Infrastructure or follow-up required before treating a crossing as traversable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteInfrastructure {
    None,
    /// Legacy combined value retained for existing persisted route records.
    FordOrBridge,
    Ford,
    Bridge,
    Ferry,
    Survey,
    Impassable,
}

/// A macro-scale route/water crossing candidate.
///
/// River coordinates identify the sampled route point inside a channel-bearing
/// watershed cell. Until higher-resolution river centre-lines are introduced,
/// they are not guaranteed to be the exact point where the route meets a bank.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCrossing {
    pub coordinate: MacroCoordinate,
    pub kind: RouteCrossingKind,
    pub infrastructure: RouteInfrastructure,
    /// Stable basin/outlet identity for confirmed river crossings.
    #[serde(default)]
    pub river_id: Option<RiverId>,
    /// Relative channel width, derived from bounded contributing area.
    #[serde(default)]
    pub river_width: Option<RiverWidthClass>,
    /// Flow direction at the crossing, useful when orienting future bridges.
    #[serde(default)]
    pub river_flow_direction: Option<RiverFlowDirection>,
}

/// Persisted, deterministic route geometry and the physical features it crosses.
/// The path follows a 32-sample search grid with midpoint validation; it is not local-map geometry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteGeography {
    #[serde(default)]
    pub status: RoutePlanningStatus,
    #[serde(default)]
    pub profile: RouteProfile,
    #[serde(default)]
    pub path: Vec<MacroCoordinate>,
    #[serde(default)]
    pub crossings: Vec<RouteCrossing>,
    #[serde(default)]
    pub total_cost: u64,
}

impl RouteGeography {
    pub fn unavailable(status: RoutePlanningStatus) -> Self {
        Self {
            status,
            profile: RouteProfile::Unavailable,
            path: Vec::new(),
            crossings: Vec::new(),
            total_cost: 0,
        }
    }

    /// Return the dominant compass heading leaving one endpoint of this route.
    /// Route paths are stored in canonical endpoint-A to endpoint-B order.
    pub fn heading_from(&self, from_a: bool) -> Option<RouteHeading> {
        if self.status != RoutePlanningStatus::Planned || self.path.len() < 2 {
            return None;
        }

        let pair = if from_a {
            self.path.windows(2).find(|pair| pair[0] != pair[1])
        } else {
            self.path.windows(2).rev().find(|pair| pair[0] != pair[1])
        }?;

        let (start, end) = if from_a {
            (pair[0], pair[1])
        } else {
            (pair[1], pair[0])
        };
        let dx = i128::from(end.x) - i128::from(start.x);
        let dy = i128::from(end.y) - i128::from(start.y);
        if dx.abs() >= dy.abs() {
            Some(if dx >= 0 {
                RouteHeading::East
            } else {
                RouteHeading::West
            })
        } else {
            Some(if dy >= 0 {
                RouteHeading::South
            } else {
                RouteHeading::North
            })
        }
    }
}

/// Compass heading for local maps, where increasing macro y maps to south.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteHeading {
    North,
    East,
    South,
    West,
}

/// Sample a bounded set of points along the direct corridor between two anchors.
///
/// Retained as a lightweight diagnostic for callers that specifically need a
/// direct-line estimate. Route creation uses `plan_route` instead.
pub fn assess_world_route(world: &World, route_id: &str) -> Option<RouteCorridorAssessment> {
    let route = world.route_by_id(route_id)?;
    let start = world
        .location_geography
        .iter()
        .find(|record| record.location_id == route.location_a_id)?
        .anchor;
    let end = world
        .location_geography
        .iter()
        .find(|record| record.location_id == route.location_b_id)?
        .anchor;
    assess_corridor(
        world.geography_seed?,
        world.geography_ruleset_version,
        start,
        end,
    )
}

pub fn assess_corridor(
    world_seed: u64,
    ruleset_version: u32,
    start: MacroCoordinate,
    end: MacroCoordinate,
) -> Option<RouteCorridorAssessment> {
    if start.stratum != 0 || end.stratum != 0 {
        return None;
    }

    let dx = i128::from(end.x) - i128::from(start.x);
    let dy = i128::from(end.y) - i128::from(start.y);
    let distance = dx.unsigned_abs().max(dy.unsigned_abs());
    let required = if distance == 0 {
        0
    } else {
        distance.div_ceil(ROUTE_GRID_SPACING as u128)
    };
    let steps = required.min((MAX_CORRIDOR_SAMPLES - 1) as u128) as usize;
    let mut result = RouteCorridorAssessment {
        undersampled: required > steps as u128,
        ..Default::default()
    };

    for index in 0..=steps {
        let fraction = index as i128;
        let denominator = (steps as i128).max(1);
        let x = i128::from(start.x) + dx * fraction / denominator;
        let y = i128::from(start.y) + dy * fraction / denominator;
        let coordinate = MacroCoordinate::new(i64::try_from(x).ok()?, i64::try_from(y).ok()?);
        let surface = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;

        result.sampled_points += 1;
        result.max_mountain_ridge_strength_q16 = result
            .max_mountain_ridge_strength_q16
            .max(surface.mountain_ridge_strength_q16());
        if surface.is_mountain_barrier() {
            result.mountain_barrier_samples += 1;
        }

        if let Some(water) = hydrology::sample_surface(world_seed, ruleset_version, coordinate) {
            match water.kind() {
                SurfaceWaterKind::Ocean => result.ocean_samples += 1,
                SurfaceWaterKind::Lake(_) => result.lake_samples += 1,
                SurfaceWaterKind::River(_) => result.river_samples += 1,
                SurfaceWaterKind::UnresolvedRiver(_) => result.unresolved_water_samples += 1,
            }
        }
    }

    Some(result)
}

/// Plan a deterministic land route using bounded A* over the shared macro lattice.
///
/// Ocean cells are impassable for this overland planner. Elevation changes and
/// mountain ridges add cost rather than becoming absolute barriers. Water
/// features on the resulting path are recorded as explicit crossing requirements.
pub fn plan_route(
    world_seed: u64,
    ruleset_version: u32,
    start: MacroCoordinate,
    end: MacroCoordinate,
) -> RouteGeography {
    if start.stratum != 0 || end.stratum != 0 {
        return RouteGeography::unavailable(RoutePlanningStatus::UnsupportedStratum);
    }

    let Some(start_terrain) = terrain::sample_surface(world_seed, ruleset_version, start) else {
        return RouteGeography::unavailable(RoutePlanningStatus::InvalidEndpoint);
    };
    let Some(end_terrain) = terrain::sample_surface(world_seed, ruleset_version, end) else {
        return RouteGeography::unavailable(RoutePlanningStatus::InvalidEndpoint);
    };
    if !start_terrain.is_land() || !end_terrain.is_land() {
        return RouteGeography::unavailable(RoutePlanningStatus::InvalidEndpoint);
    }
    if start == end {
        return planned_geography(world_seed, ruleset_version, vec![start], 0);
    }

    let dx = i128::from(end.x) - i128::from(start.x);
    let dy = i128::from(end.y) - i128::from(start.y);
    let Some(goal_x) = nearest_grid_offset(dx) else {
        return RouteGeography::unavailable(RoutePlanningStatus::SearchLimit);
    };
    let Some(goal_y) = nearest_grid_offset(dy) else {
        return RouteGeography::unavailable(RoutePlanningStatus::SearchLimit);
    };
    let goal = (goal_x, goal_y);

    if goal_x.unsigned_abs().max(goal_y.unsigned_abs()) > MAX_ROUTE_GRID_DISTANCE as u32 {
        return RouteGeography::unavailable(RoutePlanningStatus::SearchLimit);
    }
    if goal == (0, 0) {
        let Some(middle) = midpoint(start, end) else {
            return RouteGeography::unavailable(RoutePlanningStatus::InvalidEndpoint);
        };
        let Some(middle_terrain) = terrain::sample_surface(world_seed, ruleset_version, middle)
        else {
            return RouteGeography::unavailable(RoutePlanningStatus::InvalidEndpoint);
        };
        if !middle_terrain.is_land() {
            return RouteGeography::unavailable(RoutePlanningStatus::NoLandPathInSearchWindow);
        }
        let movement_cost = if dx == 0 || dy == 0 { 100_u64 } else { 141_u64 };
        let slope_cost = u64::from(
            start_terrain
                .elevation_q16()
                .abs_diff(end_terrain.elevation_q16()),
        ) / 128;
        let ridge_cost = u64::from(
            middle_terrain
                .mountain_ridge_strength_q16()
                .saturating_add(end_terrain.mountain_ridge_strength_q16()),
        ) / 128;
        let total_cost = movement_cost + slope_cost + ridge_cost;
        return planned_geography(
            world_seed,
            ruleset_version,
            vec![start, middle, end],
            total_cost,
        );
    }

    let min_x = goal_x.min(0).saturating_sub(SEARCH_MARGIN_CELLS);
    let max_x = goal_x.max(0).saturating_add(SEARCH_MARGIN_CELLS);
    let min_y = goal_y.min(0).saturating_sub(SEARCH_MARGIN_CELLS);
    let max_y = goal_y.max(0).saturating_add(SEARCH_MARGIN_CELLS);

    let mut terrain_cache = HashMap::new();
    terrain_cache.insert((0, 0), start_terrain);
    terrain_cache.insert(goal, end_terrain);

    let mut open = BinaryHeap::new();
    let mut costs = HashMap::<(i32, i32), u64>::new();
    let mut previous = HashMap::<(i32, i32), (i32, i32)>::new();
    costs.insert((0, 0), 0);
    open.push(Reverse((heuristic((0, 0), goal), 0_u64, 0_i32, 0_i32)));

    let neighbors = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let mut expanded = 0_usize;
    let mut reached = false;

    while let Some(Reverse((_estimated_total, current_cost, x, y))) = open.pop() {
        let current = (x, y);
        if costs.get(&current).copied() != Some(current_cost) {
            continue;
        }
        if current == goal {
            reached = true;
            break;
        }
        if expanded >= MAX_ROUTE_SEARCH_NODES {
            break;
        }
        expanded += 1;

        let Some(current_terrain) = terrain_cache.get(&current).copied() else {
            continue;
        };

        for (step_x, step_y) in neighbors {
            let next = (x + step_x, y + step_y);
            if next.0 < min_x || next.0 > max_x || next.1 < min_y || next.1 > max_y {
                continue;
            }

            let next_terrain = if let Some(sample) = terrain_cache.get(&next).copied() {
                sample
            } else {
                let Some(coordinate) = route_node_coordinate(start, end, goal, next) else {
                    continue;
                };
                let Some(sample) = terrain::sample_surface(world_seed, ruleset_version, coordinate)
                else {
                    continue;
                };
                if !sample.is_land() {
                    continue;
                }
                terrain_cache.insert(next, sample);
                sample
            };

            if !next_terrain.is_land() {
                continue;
            }

            let Some(current_coordinate) = route_node_coordinate(start, end, goal, current) else {
                continue;
            };
            let Some(next_coordinate) = route_node_coordinate(start, end, goal, next) else {
                continue;
            };
            let Some(midpoint) = midpoint(current_coordinate, next_coordinate) else {
                continue;
            };
            let Some(midpoint_terrain) =
                terrain::sample_surface(world_seed, ruleset_version, midpoint)
            else {
                continue;
            };
            if !midpoint_terrain.is_land() {
                continue;
            }

            let movement_cost = if step_x == 0 || step_y == 0 {
                100_u64
            } else {
                141_u64
            };
            let elevation_delta = current_terrain
                .elevation_q16()
                .abs_diff(next_terrain.elevation_q16());
            let slope_cost = u64::from(elevation_delta) / 128;
            let ridge_cost = u64::from(
                midpoint_terrain
                    .mountain_ridge_strength_q16()
                    .saturating_add(next_terrain.mountain_ridge_strength_q16()),
            ) / 128;
            let next_cost = current_cost
                .saturating_add(movement_cost)
                .saturating_add(slope_cost)
                .saturating_add(ridge_cost);

            if costs.get(&next).is_some_and(|known| *known <= next_cost) {
                continue;
            }

            costs.insert(next, next_cost);
            previous.insert(next, current);
            let estimate = next_cost.saturating_add(heuristic(next, goal));
            open.push(Reverse((estimate, next_cost, next.0, next.1)));
        }
    }

    if !reached {
        return RouteGeography::unavailable(if expanded >= MAX_ROUTE_SEARCH_NODES {
            RoutePlanningStatus::SearchLimit
        } else {
            RoutePlanningStatus::NoLandPathInSearchWindow
        });
    }

    let mut nodes = vec![goal];
    let mut cursor = goal;
    while cursor != (0, 0) {
        let Some(parent) = previous.get(&cursor).copied() else {
            return RouteGeography::unavailable(RoutePlanningStatus::NoLandPathInSearchWindow);
        };
        nodes.push(parent);
        cursor = parent;
    }
    nodes.reverse();

    let mut node_coordinates = Vec::with_capacity(nodes.len());
    for node in nodes {
        let Some(coordinate) = route_node_coordinate(start, end, goal, node) else {
            return RouteGeography::unavailable(RoutePlanningStatus::InvalidEndpoint);
        };
        node_coordinates.push(coordinate);
    }

    let mut path = Vec::with_capacity(node_coordinates.len().saturating_mul(2));
    if let Some(first) = node_coordinates.first().copied() {
        path.push(first);
    }
    for pair in node_coordinates.windows(2) {
        let Some(middle) = midpoint(pair[0], pair[1]) else {
            return RouteGeography::unavailable(RoutePlanningStatus::InvalidEndpoint);
        };
        path.push(middle);
        path.push(pair[1]);
    }
    if path.len() > MAX_ROUTE_PATH_POINTS {
        return RouteGeography::unavailable(RoutePlanningStatus::SearchLimit);
    }

    let total_cost = costs.get(&goal).copied().unwrap_or_default();
    planned_geography(world_seed, ruleset_version, path, total_cost)
}

/// Compute and persist route geography for routes that do not have it yet.
/// Existing route plans are never silently rerolled by later geography changes.
pub fn ensure_world_route_geography(world: &mut World) -> usize {
    let Some(seed) = world.geography_seed else {
        return 0;
    };
    let ruleset = world.geography_ruleset_version;
    let anchors = world
        .location_geography
        .iter()
        .map(|record| (record.location_id, record.anchor))
        .collect::<BTreeMap<_, _>>();

    let mut generated = 0;
    for route in &mut world.routes {
        if route.geography.is_some() {
            continue;
        }
        let (Some(start), Some(end)) = (
            anchors.get(&route.location_a_id).copied(),
            anchors.get(&route.location_b_id).copied(),
        ) else {
            // A route may be added before location embedding. Leave it pending
            // so a later embedding pass can plan it instead of persisting failure.
            continue;
        };
        route.geography = Some(plan_route(seed, ruleset, start, end));
        generated += 1;
    }
    generated
}

fn midpoint(start: MacroCoordinate, end: MacroCoordinate) -> Option<MacroCoordinate> {
    let x = (i128::from(start.x) + i128::from(end.x)) / 2;
    let y = (i128::from(start.y) + i128::from(end.y)) / 2;
    Some(MacroCoordinate::new(
        i64::try_from(x).ok()?,
        i64::try_from(y).ok()?,
    ))
}

fn nearest_grid_offset(delta: i128) -> Option<i32> {
    let spacing = i128::from(ROUTE_GRID_SPACING);
    let quotient = delta / spacing;
    let remainder = delta % spacing;
    let rounded = if remainder.abs() * 2 >= spacing {
        quotient + if delta >= 0 { 1 } else { -1 }
    } else {
        quotient
    };
    i32::try_from(rounded).ok()
}

fn route_node_coordinate(
    start: MacroCoordinate,
    end: MacroCoordinate,
    goal: (i32, i32),
    node: (i32, i32),
) -> Option<MacroCoordinate> {
    if node == (0, 0) {
        return Some(start);
    }
    if node == goal {
        return Some(end);
    }
    let x = i128::from(start.x) + i128::from(node.0) * i128::from(ROUTE_GRID_SPACING);
    let y = i128::from(start.y) + i128::from(node.1) * i128::from(ROUTE_GRID_SPACING);
    Some(MacroCoordinate::new(
        i64::try_from(x).ok()?,
        i64::try_from(y).ok()?,
    ))
}

fn heuristic(from: (i32, i32), to: (i32, i32)) -> u64 {
    let dx = (i64::from(to.0) - i64::from(from.0)).unsigned_abs();
    let dy = (i64::from(to.1) - i64::from(from.1)).unsigned_abs();
    let diagonal = dx.min(dy);
    let straight = dx.max(dy) - diagonal;
    diagonal * 141 + straight * 100
}

fn planned_geography(
    world_seed: u64,
    ruleset_version: u32,
    path: Vec<MacroCoordinate>,
    total_cost: u64,
) -> RouteGeography {
    let crossings = classify_crossings(world_seed, ruleset_version, &path);
    let profile = classify_profile(&crossings);
    RouteGeography {
        status: RoutePlanningStatus::Planned,
        profile,
        path,
        crossings,
        total_cost,
    }
}

fn classify_crossings(
    world_seed: u64,
    ruleset_version: u32,
    path: &[MacroCoordinate],
) -> Vec<RouteCrossing> {
    let mut crossings = Vec::new();
    let mut river_samples = HashMap::new();
    let mut previous_water_crossing: Option<(RouteCrossingKind, Option<RiverId>)> = None;
    for (index, coordinate) in path.iter().enumerate() {
        let Some(surface) = terrain::sample_surface(world_seed, ruleset_version, *coordinate)
        else {
            continue;
        };
        if surface.is_mountain_barrier() {
            push_crossing(
                &mut crossings,
                *coordinate,
                RouteCrossingKind::MountainPass,
                RouteInfrastructure::None,
                None,
                None,
                None,
            );
        }

        // Water classification is more expensive than elevation sampling.
        // Probe each 32-sample grid node and always include the final endpoint.
        if path.len() > 3 && index % 2 != 0 && index + 1 != path.len() {
            continue;
        }
        let Some(water) = sample_route_water(
            world_seed,
            ruleset_version,
            *coordinate,
            &mut river_samples,
        ) else {
            previous_water_crossing = None;
            continue;
        };
        match water.kind() {
            SurfaceWaterKind::Ocean => {
                let key = (RouteCrossingKind::Ocean, None);
                if previous_water_crossing != Some(key) {
                    push_crossing(
                        &mut crossings,
                        *coordinate,
                        key.0,
                        infrastructure_for_water(key.0, None),
                        None,
                        None,
                        None,
                    );
                }
                previous_water_crossing = Some(key);
            }
            SurfaceWaterKind::Lake(_) => {
                let key = (RouteCrossingKind::Lake, None);
                if previous_water_crossing != Some(key) {
                    push_crossing(
                        &mut crossings,
                        *coordinate,
                        key.0,
                        infrastructure_for_water(key.0, None),
                        None,
                        None,
                        None,
                    );
                }
                previous_water_crossing = Some(key);
            }
            SurfaceWaterKind::River(river) => {
                // A continuous water stretch is one physical crossing.
                // Mountain markers and same-run river-cell identity queries do
                // not interrupt continuity or repeat the bounded outlet trace.
                let already_crossing_river = previous_water_crossing
                    .is_some_and(|(kind, _)| kind == RouteCrossingKind::River);
                if !already_crossing_river {
                    let river_width = river.width_class();
                    let river_id = river.network_id(world_seed, ruleset_version);
                    push_crossing(
                        &mut crossings,
                        *coordinate,
                        RouteCrossingKind::River,
                        infrastructure_for_water(RouteCrossingKind::River, Some(river_width)),
                        river_id,
                        Some(river_width),
                        river.flow_direction(world_seed, ruleset_version),
                    );
                    previous_water_crossing =
                        Some((RouteCrossingKind::River, river_id));
                }
            }
            SurfaceWaterKind::UnresolvedRiver(_) => {
                let key = (RouteCrossingKind::UnresolvedWater, None);
                if previous_water_crossing != Some(key) {
                    push_crossing(
                        &mut crossings,
                        *coordinate,
                        key.0,
                        infrastructure_for_water(key.0, None),
                        None,
                        None,
                        None,
                    );
                }
                previous_water_crossing = Some(key);
            }
        }
    }
    crossings
}

/// Classify route water while reusing river accumulation for every point
/// that belongs to the same coarse watershed cell.
///
/// River presence is still a coarse-cell candidate, not a precise local-channel
/// intersection. The finer river geometry needed to identify exact bank crossings
/// remains a separate presentation/topology refinement.
fn sample_route_water(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
    river_samples: &mut HashMap<MacroCoordinate, Option<rivers::RiverSample>>,
) -> Option<SurfaceWaterKind> {
    let local_terrain = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;
    if !local_terrain.is_land() {
        return Some(SurfaceWaterKind::Ocean);
    }

    if let Some(lake) = waterbodies::sample_surface(world_seed, ruleset_version, coordinate) {
        return Some(SurfaceWaterKind::Lake(lake));
    }

    let coarse_cell = watershed::sample_surface(world_seed, ruleset_version, coordinate)?
        .coarse_cell();
    let river = *river_samples
        .entry(coarse_cell)
        .or_insert_with(|| rivers::sample_surface(world_seed, ruleset_version, coarse_cell));
    let river = river?;

    if river.is_channel() {
        Some(SurfaceWaterKind::River(river))
    } else if river.accumulation().is_truncated() {
        Some(SurfaceWaterKind::UnresolvedRiver(river))
    } else {
        None
    }
}

fn infrastructure_for_water(
    kind: RouteCrossingKind,
    river_width: Option<RiverWidthClass>,
) -> RouteInfrastructure {
    match kind {
        RouteCrossingKind::River => river_width
            .map(river_infrastructure)
            .unwrap_or(RouteInfrastructure::Survey),
        RouteCrossingKind::Lake => RouteInfrastructure::Ferry,
        RouteCrossingKind::Ocean => RouteInfrastructure::Impassable,
        RouteCrossingKind::UnresolvedWater => RouteInfrastructure::Survey,
        RouteCrossingKind::MountainPass => RouteInfrastructure::None,
    }
}

fn river_infrastructure(width: RiverWidthClass) -> RouteInfrastructure {
    if width.is_fordable() {
        RouteInfrastructure::Ford
    } else {
        RouteInfrastructure::Bridge
    }
}

fn push_crossing(
    crossings: &mut Vec<RouteCrossing>,
    coordinate: MacroCoordinate,
    kind: RouteCrossingKind,
    infrastructure: RouteInfrastructure,
    river_id: Option<RiverId>,
    river_width: Option<RiverWidthClass>,
    river_flow_direction: Option<RiverFlowDirection>,
) {
    let same_crossing = crossings.last().is_some_and(|last| {
        last.kind == kind
            && (kind != RouteCrossingKind::River || last.river_id == river_id)
    });
    if same_crossing {
        return;
    }
    crossings.push(RouteCrossing {
        coordinate,
        kind,
        infrastructure,
        river_id,
        river_width,
        river_flow_direction,
    });
}

fn classify_profile(crossings: &[RouteCrossing]) -> RouteProfile {
    if crossings
        .iter()
        .any(|crossing| crossing.kind == RouteCrossingKind::UnresolvedWater)
    {
        return RouteProfile::Inconclusive;
    }
    let has_mountain = crossings
        .iter()
        .any(|crossing| crossing.kind == RouteCrossingKind::MountainPass);
    let has_river = crossings
        .iter()
        .any(|crossing| crossing.kind == RouteCrossingKind::River);
    let has_water = crossings.iter().any(|crossing| {
        matches!(
            crossing.kind,
            RouteCrossingKind::Lake | RouteCrossingKind::Ocean
        )
    });
    let kinds = has_mountain as usize + has_river as usize + has_water as usize;
    if kinds > 1 {
        RouteProfile::Mixed
    } else if has_water {
        RouteProfile::WaterCrossing
    } else if has_river {
        RouteProfile::RiverCrossing
    } else if has_mountain {
        RouteProfile::MountainPass
    } else {
        RouteProfile::Trail
    }
}

#[cfg(test)]
#[path = "../../tests/unit/route_geography.rs"]
mod tests;
