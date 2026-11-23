//! Deterministic, bounded terrain-aware planning for persistent world routes.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap};

use serde::{Deserialize, Serialize};

use super::climate::{self, Biome};
use super::hydrology::SurfaceWaterKind;
use super::river_geometry::{RiverBankIntersection, RiverGeometryCache, RiverPosition};
use super::rivers::{self, RiverFlowDirection, RiverId, RiverWidthClass};
use super::{hydrology, terrain, waterbodies, watershed, MacroCoordinate};
use crate::model::World;

pub const MAX_CORRIDOR_SAMPLES: usize = 16;
/// Maximum number of representative geography samples in a physical route scene.
pub const MAX_ROUTE_LANDSCAPE_SAMPLES: usize = 16;
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

/// Broad environment categories used only by physical-travel presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteLandscapeKind {
    Water,
    Mountain,
    Forest,
    Valley,
    Desert,
    Coast,
    Tundra,
    Grassland,
}

/// One bounded, direction-aware sample of the persisted macro route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteLandscapeSample {
    pub coordinate: MacroCoordinate,
    pub kind: RouteLandscapeKind,
    pub elevation_q16: i32,
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

/// The two shores where a planned route enters and leaves an inland lake.
/// Positions follow route travel order and use the shared Q16 macro lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LakeBankIntersection {
    pub entry: RiverPosition,
    pub exit: RiverPosition,
}

impl LakeBankIntersection {
    pub fn midpoint(self) -> Option<RiverPosition> {
        self.entry.midpoint(self.exit)
    }
}

/// A macro-scale route/water crossing candidate.
///
/// Confirmed river and lake crossings retain exact bank intersections when
/// available. `coordinate` is their rounded midpoint; older and unresolved
/// records retain their sampled macro-geography coordinate.
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
    /// Sub-sample entry and exit points where the route meets the generated banks.
    /// Absent on legacy records and on non-river crossings.
    #[serde(default)]
    pub river_banks: Option<RiverBankIntersection>,
    /// Stable generated identity for an inland-lake crossing.
    #[serde(default)]
    pub lake_id: Option<String>,
    /// Sub-sample entry and exit shores for a confirmed lake crossing.
    #[serde(default)]
    pub lake_banks: Option<LakeBankIntersection>,
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

/// Route-cost units required for one two-hour world-time portion.
pub const ROUTE_COST_PER_TIME_POINT: u64 = 50;
const MIN_ROUTE_DISTANCE_COST: u64 = 100;

/// A deterministic, renderer-independent estimate for travelling one world route.
///
/// Distance and terrain costs use normalized route-cost units, not metres or
/// kilometres. One hundred units represent one ordinary 32-sample route-grid
/// step and advance two world-time points (four in-game hours).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteTravelEstimate {
    pub distance_cost: u64,
    pub terrain_cost: u64,
    pub crossing_cost: u64,
    pub total_cost: u64,
    pub time_points: u32,
}

impl RouteGeography {
    /// Estimate travel duration from persisted route cost, bounded biome samples,
    /// and the explicit crossings already required by this route.
    ///
    /// Missing world seeds omit only the biome adjustment; persisted route
    /// distance, planner terrain cost, and crossing requirements remain usable.
    pub fn travel_estimate(
        &self,
        world_seed: Option<u64>,
        ruleset_version: u32,
    ) -> RouteTravelEstimate {
        let has_planned_path = self.status == RoutePlanningStatus::Planned;
        let distance_cost = if has_planned_path {
            route_distance_cost(&self.path)
        } else {
            MIN_ROUTE_DISTANCE_COST
        };
        let planner_terrain_cost = if has_planned_path {
            self.total_cost.saturating_sub(distance_cost)
        } else {
            0
        };
        let biome_penalty_percent = if has_planned_path {
            world_seed
                .map(|seed| {
                    let samples = self.landscape_samples(seed, ruleset_version, true);
                    if samples.is_empty() {
                        0
                    } else {
                        let total = samples
                            .iter()
                            .map(|sample| landscape_travel_penalty_percent(sample.kind))
                            .sum::<u64>();
                        total.div_ceil(samples.len() as u64)
                    }
                })
                .unwrap_or(0)
        } else {
            0
        };
        let biome_cost = distance_cost
            .saturating_mul(biome_penalty_percent)
            .div_ceil(100);
        let terrain_cost = planner_terrain_cost.saturating_add(biome_cost);
        let crossing_cost = self.crossings.iter().fold(0_u64, |total, crossing| {
            total.saturating_add(match crossing.kind {
                RouteCrossingKind::MountainPass => 50,
                RouteCrossingKind::River => match crossing.infrastructure {
                    RouteInfrastructure::Bridge => 0,
                    RouteInfrastructure::Ford | RouteInfrastructure::FordOrBridge => 50,
                    _ => 0,
                },
                RouteCrossingKind::Lake => 100,
                RouteCrossingKind::Ocean | RouteCrossingKind::UnresolvedWater => 0,
            })
        });
        let total_cost = distance_cost
            .saturating_add(terrain_cost)
            .saturating_add(crossing_cost);
        let time_points = total_cost
            .div_ceil(ROUTE_COST_PER_TIME_POINT)
            .max(2)
            .min(u64::from(u32::MAX)) as u32;

        RouteTravelEstimate {
            distance_cost,
            terrain_cost,
            crossing_cost,
            total_cost,
            time_points,
        }
    }
}

fn route_distance_cost(path: &[MacroCoordinate]) -> u64 {
    let numerator = path.windows(2).fold(0_u128, |total, pair| {
        let dx = u128::from(pair[0].x.abs_diff(pair[1].x));
        let dy = u128::from(pair[0].y.abs_diff(pair[1].y));
        let major = dx.max(dy);
        let minor = dx.min(dy);
        total.saturating_add(major.saturating_mul(100).saturating_add(minor.saturating_mul(41)))
    });
    let cost = numerator.div_ceil(ROUTE_GRID_SPACING as u128);
    u64::try_from(cost)
        .unwrap_or(u64::MAX)
        .max(MIN_ROUTE_DISTANCE_COST)
}

fn landscape_travel_penalty_percent(kind: RouteLandscapeKind) -> u64 {
    match kind {
        RouteLandscapeKind::Grassland | RouteLandscapeKind::Valley | RouteLandscapeKind::Water => {
            0
        }
        RouteLandscapeKind::Coast => 10,
        RouteLandscapeKind::Forest => 15,
        RouteLandscapeKind::Desert => 20,
        RouteLandscapeKind::Tundra => 25,
        RouteLandscapeKind::Mountain => 35,
    }
}

impl RouteGeography {
    /// Return resolved inland-lake crossings in the order they are encountered
    /// from the selected route endpoint. Crossings without two verified shores
    /// remain requirements, but cannot yet drive a physical ferry scene.
    pub fn resolved_lake_crossings(&self, from_a: bool) -> Vec<&RouteCrossing> {
        let crossings = self.crossings.iter().filter(|crossing| {
            crossing.kind == RouteCrossingKind::Lake
                && crossing.lake_id.is_some()
                && crossing.lake_banks.is_some()
        });
        if from_a {
            crossings.collect()
        } else {
            crossings.rev().collect()
        }
    }

    pub fn has_resolved_lake_crossing(&self) -> bool {
        self.crossings.iter().any(|crossing| {
            crossing.kind == RouteCrossingKind::Lake
                && crossing.lake_id.is_some()
                && crossing.lake_banks.is_some()
        })
    }

    pub fn has_resolved_river_crossing(&self) -> bool {
        self.crossings.iter().any(|crossing| {
            crossing.kind == RouteCrossingKind::River
                && crossing.river_banks.is_some()
                && matches!(
                    crossing.infrastructure,
                    RouteInfrastructure::Ford | RouteInfrastructure::Bridge
                )
        })
    }

    pub fn resolved_travel_crossings(&self, from_a: bool) -> Vec<&RouteCrossing> {
        let crossings = self
            .crossings
            .iter()
            .filter(|crossing| match crossing.kind {
                RouteCrossingKind::River => {
                    crossing.river_banks.is_some()
                        && matches!(
                            crossing.infrastructure,
                            RouteInfrastructure::Ford | RouteInfrastructure::Bridge
                        )
                }
                RouteCrossingKind::Lake => {
                    crossing.lake_id.is_some() && crossing.lake_banks.is_some()
                }
                _ => false,
            });
        if from_a {
            crossings.collect()
        } else {
            crossings.rev().collect()
        }
    }

    /// Sample the persisted route at evenly spaced positions for a short physical-travel scene.
    ///
    /// The returned samples follow the selected travel direction and are capped independently
    /// of route length. They are transient presentation data; they are never persisted.
    pub fn landscape_samples(
        &self,
        world_seed: u64,
        ruleset_version: u32,
        from_a: bool,
    ) -> Vec<RouteLandscapeSample> {
        if self.status != RoutePlanningStatus::Planned || self.path.len() < 2 {
            return Vec::new();
        }

        let path_len = self.path.len();
        let sample_count = path_len.min(MAX_ROUTE_LANDSCAPE_SAMPLES);
        let mut samples = Vec::with_capacity(sample_count);
        for sample_index in 0..sample_count {
            let intervals = sample_count - 1;
            let forward_index = (sample_index * (path_len - 1) + intervals / 2) / intervals;
            let path_index = if from_a {
                forward_index
            } else {
                path_len - 1 - forward_index
            };
            let coordinate = self.path[path_index];
            let Some(terrain_sample) =
                terrain::sample_surface(world_seed, ruleset_version, coordinate)
            else {
                continue;
            };

            let kind = match hydrology::sample_surface(world_seed, ruleset_version, coordinate)
                .map(|sample| sample.kind())
            {
                Some(
                    SurfaceWaterKind::Ocean
                    | SurfaceWaterKind::Lake(_)
                    | SurfaceWaterKind::River(_)
                    | SurfaceWaterKind::UnresolvedRiver(_),
                ) => RouteLandscapeKind::Water,
                _ => {
                    let biome = climate::sample_surface(world_seed, ruleset_version, coordinate)
                        .map(|sample| sample.biome())
                        .unwrap_or(Biome::Grassland);
                    if terrain_sample.is_mountain_barrier() || biome == Biome::Alpine {
                        RouteLandscapeKind::Mountain
                    } else {
                        match biome {
                            Biome::TemperateForest | Biome::Taiga => RouteLandscapeKind::Forest,
                            Biome::Desert => RouteLandscapeKind::Desert,
                            Biome::Coast => RouteLandscapeKind::Coast,
                            Biome::Tundra => RouteLandscapeKind::Tundra,
                            Biome::Ocean => RouteLandscapeKind::Water,
                            Biome::Grassland => RouteLandscapeKind::Grassland,
                            Biome::Alpine => RouteLandscapeKind::Mountain,
                        }
                    }
                }
            };
            samples.push(RouteLandscapeSample {
                coordinate,
                kind,
                elevation_q16: terrain_sample.elevation_q16(),
            });
        }

        // A local low point between two higher open-land samples is presented as a valley.
        // This is a visual cue only; it does not change route cost or geography.
        let valley_indices = samples
            .windows(3)
            .enumerate()
            .filter_map(|(index, window)| {
                let previous = window[0];
                let current = window[1];
                let next = window[2];
                (current.kind == RouteLandscapeKind::Grassland
                    && previous.kind == RouteLandscapeKind::Grassland
                    && next.kind == RouteLandscapeKind::Grassland
                    && current.elevation_q16.saturating_add(4_096) < previous.elevation_q16
                    && current.elevation_q16.saturating_add(4_096) < next.elevation_q16)
                    .then_some(index + 1)
            })
            .collect::<Vec<_>>();
        for index in valley_indices {
            samples[index].kind = RouteLandscapeKind::Valley;
        }

        samples
    }

    /// Return a reason when explicit route geography cannot currently be crossed.
    /// Legacy FordOrBridge river records remain traversable.
    pub fn travel_block_reason(&self) -> Option<&'static str> {
        for crossing in &self.crossings {
            if crossing.infrastructure == RouteInfrastructure::Impassable
                || crossing.kind == RouteCrossingKind::Ocean
            {
                return Some("Open ocean blocks this overland route.");
            }
            if crossing.infrastructure == RouteInfrastructure::Survey
                || crossing.kind == RouteCrossingKind::UnresolvedWater
            {
                return Some(
                    "This water crossing requires a survey before the route can be travelled.",
                );
            }

            match crossing.kind {
                RouteCrossingKind::River => match crossing.infrastructure {
                    RouteInfrastructure::Ford | RouteInfrastructure::Bridge
                        if crossing.river_banks.is_some() => {}
                    RouteInfrastructure::FordOrBridge => {}
                    _ => {
                        return Some(
                            "The river crossing has no verified banks or suitable ford/bridge.",
                        );
                    }
                },
                RouteCrossingKind::Lake => {
                    if crossing.infrastructure != RouteInfrastructure::Ferry
                        || crossing.lake_id.is_none()
                        || crossing.lake_banks.is_none()
                    {
                        return Some(
                            "The lake crossing needs verified shores and a ferry landing.",
                        );
                    }
                }
                RouteCrossingKind::Ocean | RouteCrossingKind::UnresolvedWater => unreachable!(),
                RouteCrossingKind::MountainPass => {}
            }
        }
        None
    }

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
    let mut river_geometry = RiverGeometryCache::default();
    // Keep shore checks independent from route-segment queries, whose bounded
    // cache may already be near capacity on a long route.
    let mut lake_shore_river_geometry = RiverGeometryCache::default();
    let mut previous_water_crossing: Option<(RouteCrossingKind, Option<RiverId>)> = None;
    let mut active_lake: Option<ActiveLakeCrossing> = None;

    for (index, coordinate) in path.iter().enumerate() {
        let Some(surface) = terrain::sample_surface(world_seed, ruleset_version, *coordinate)
        else {
            if let Some(active) = active_lake.take() {
                finish_lake_crossing(
                    world_seed,
                    ruleset_version,
                    &mut crossings,
                    active,
                    Some(*coordinate),
                    &mut lake_shore_river_geometry,
                );
                previous_water_crossing = None;
            }
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

        if index > 0 {
            let route_start = path[index - 1];
            let route_end = *coordinate;
            if let (Some(start), Some(end)) = (
                RiverPosition::from_macro_coordinate(route_start),
                RiverPosition::from_macro_coordinate(route_end),
            ) {
                if let Some(midpoint) = start.midpoint(end) {
                    for segment in
                        river_geometry.segments_near(world_seed, ruleset_version, midpoint)
                    {
                        if let Some(banks) = segment.intersect_route(start, end) {
                            push_river_crossing(&mut crossings, banks, segment);
                        }
                    }
                }
            }
        }

        // Lake coverage is checked at every bounded route node so a shore
        // transition cannot disappear between the coarser river/ocean probes.
        let lake = if surface.is_land() {
            waterbodies::sample_surface(world_seed, ruleset_version, *coordinate)
        } else {
            None
        };
        if let Some(lake) = lake {
            let lake_id = lake.id(world_seed, ruleset_version);
            if !active_lake
                .as_ref()
                .is_some_and(|active| active.lake_id == lake_id)
            {
                if let Some(active) = active_lake.take() {
                    finish_lake_crossing(
                        world_seed,
                        ruleset_version,
                        &mut crossings,
                        active,
                        Some(*coordinate),
                        &mut lake_shore_river_geometry,
                    );
                }

                let entry = index.checked_sub(1).and_then(|previous_index| {
                    find_lake_bank_boundary(
                        world_seed,
                        ruleset_version,
                        &lake_id,
                        *coordinate,
                        path[previous_index],
                        &mut lake_shore_river_geometry,
                    )
                });
                let crossing_index = crossings.len();
                crossings.push(RouteCrossing {
                    coordinate: *coordinate,
                    kind: RouteCrossingKind::Lake,
                    infrastructure: RouteInfrastructure::Ferry,
                    river_id: None,
                    river_width: None,
                    river_flow_direction: None,
                    river_banks: None,
                    lake_id: Some(lake_id.clone()),
                    lake_banks: None,
                });
                active_lake = Some(ActiveLakeCrossing {
                    lake_id,
                    crossing_index,
                    entry,
                    last_inside: *coordinate,
                });
            } else if let Some(active) = active_lake.as_mut() {
                active.last_inside = *coordinate;
            }
            previous_water_crossing = None;
            continue;
        }

        if let Some(active) = active_lake.take() {
            finish_lake_crossing(
                world_seed,
                ruleset_version,
                &mut crossings,
                active,
                Some(*coordinate),
                &mut lake_shore_river_geometry,
            );
            previous_water_crossing = None;
        }

        // River accumulation is cached by coarse watershed cell. Keep these
        // broader water queries at every other node; lake shore metadata above
        // uses the full, still-bounded route path.
        if path.len() > 3 && index % 2 != 0 && index + 1 != path.len() {
            continue;
        }
        let Some(water) =
            sample_route_water(world_seed, ruleset_version, *coordinate, &mut river_samples)
        else {
            previous_water_crossing = None;
            continue;
        };
        match water {
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
                // Lake samples are handled above at every path node.
                previous_water_crossing = None;
            }
            SurfaceWaterKind::River(_) => {
                // A channel-bearing coarse watershed cell is no longer enough
                // to claim a road crossing. Only intersected local bank geometry
                // above produces a confirmed river crossing record.
                previous_water_crossing = None;
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

    if let Some(active) = active_lake {
        finish_lake_crossing(
            world_seed,
            ruleset_version,
            &mut crossings,
            active,
            None,
            &mut lake_shore_river_geometry,
        );
    }
    crossings
}

#[derive(Debug)]
struct ActiveLakeCrossing {
    lake_id: String,
    crossing_index: usize,
    entry: Option<RiverPosition>,
    last_inside: MacroCoordinate,
}

fn finish_lake_crossing(
    world_seed: u64,
    ruleset_version: u32,
    crossings: &mut [RouteCrossing],
    active: ActiveLakeCrossing,
    next_coordinate: Option<MacroCoordinate>,
    river_geometry: &mut RiverGeometryCache,
) {
    let Some(next_coordinate) = next_coordinate else {
        return;
    };
    let Some(entry) = active.entry else {
        return;
    };
    let Some(exit) = find_lake_bank_boundary(
        world_seed,
        ruleset_version,
        &active.lake_id,
        active.last_inside,
        next_coordinate,
        river_geometry,
    ) else {
        return;
    };
    let banks = LakeBankIntersection { entry, exit };
    let Some(coordinate) = banks
        .midpoint()
        .and_then(RiverPosition::rounded_macro_coordinate)
    else {
        return;
    };
    if let Some(crossing) = crossings.get_mut(active.crossing_index) {
        crossing.coordinate = coordinate;
        crossing.lake_banks = Some(banks);
    }
}

/// Resolve the shore between a known point in one lake and a known dry-land
/// point. The bounded integer search narrows the transition to adjacent macro
/// samples, then stores their half-sample midpoint in the shared Q16 lattice.
fn find_lake_bank_boundary(
    world_seed: u64,
    ruleset_version: u32,
    lake_id: &str,
    wet_coordinate: MacroCoordinate,
    dry_coordinate: MacroCoordinate,
    river_geometry: &mut RiverGeometryCache,
) -> Option<RiverPosition> {
    let wet_sample = waterbodies::sample_surface(world_seed, ruleset_version, wet_coordinate)?;
    if wet_sample.id(world_seed, ruleset_version).as_str() != lake_id {
        return None;
    }
    if !terrain::sample_surface(world_seed, ruleset_version, dry_coordinate)?.is_land()
        || waterbodies::sample_surface(world_seed, ruleset_version, dry_coordinate).is_some()
    {
        return None;
    }
    let dry_position = RiverPosition::from_macro_coordinate(dry_coordinate)?;
    if river_geometry
        .segments_near(world_seed, ruleset_version, dry_position)
        .iter()
        .any(|segment| segment.contains_position(dry_position))
    {
        return None;
    }

    let mut wet = wet_coordinate;
    let mut dry = dry_coordinate;
    for _ in 0..64 {
        let dx = (i128::from(wet.x) - i128::from(dry.x)).abs();
        let dy = (i128::from(wet.y) - i128::from(dry.y)).abs();
        if dx.max(dy) <= 1 {
            break;
        }
        let midpoint = MacroCoordinate::new(
            i64::try_from((i128::from(wet.x) + i128::from(dry.x)) / 2).ok()?,
            i64::try_from((i128::from(wet.y) + i128::from(dry.y)) / 2).ok()?,
        );
        if midpoint == wet || midpoint == dry {
            break;
        }
        match waterbodies::sample_surface(world_seed, ruleset_version, midpoint) {
            Some(sample) if sample.id(world_seed, ruleset_version).as_str() == lake_id => {
                wet = midpoint;
            }
            Some(_) => return None,
            None => {
                if !terrain::sample_surface(world_seed, ruleset_version, midpoint)?.is_land() {
                    return None;
                }
                let midpoint_position = RiverPosition::from_macro_coordinate(midpoint)?;
                if river_geometry
                    .segments_near(world_seed, ruleset_version, midpoint_position)
                    .iter()
                    .any(|segment| segment.contains_position(midpoint_position))
                {
                    return None;
                }
                dry = midpoint;
            }
        }
    }

    RiverPosition::from_macro_coordinate(wet)?.midpoint(RiverPosition::from_macro_coordinate(dry)?)
}

fn push_river_crossing(
    crossings: &mut Vec<RouteCrossing>,
    banks: RiverBankIntersection,
    segment: &super::river_geometry::RiverChannelSegment,
) {
    let Some(midpoint) = banks.midpoint() else {
        return;
    };
    if let Some(last) = crossings.last() {
        if last.kind == RouteCrossingKind::River && last.river_id == Some(segment.river_id()) {
            if let Some(previous) = last.river_banks {
                if let Some(previous_midpoint) = previous.midpoint() {
                    let dx = i128::from(midpoint.x_q16) - i128::from(previous_midpoint.x_q16);
                    let dy = i128::from(midpoint.y_q16) - i128::from(previous_midpoint.y_q16);
                    let tolerance = i128::from(super::river_geometry::RIVER_POSITION_SCALE) * 2;
                    if dx * dx + dy * dy <= tolerance * tolerance {
                        return;
                    }
                }
            }
        }
    }
    let Some(coordinate) = midpoint.rounded_macro_coordinate() else {
        return;
    };
    let width = segment.width_class();
    push_crossing(
        crossings,
        coordinate,
        RouteCrossingKind::River,
        infrastructure_for_water(RouteCrossingKind::River, Some(width)),
        Some(segment.river_id()),
        Some(width),
        Some(segment.flow_direction()),
    );
    if let Some(crossing) = crossings.last_mut() {
        crossing.river_banks = Some(banks);
    }
}

/// Classify non-lake surface water while reusing bounded river accumulation for
/// each coarse watershed cell. Confirmed river channels are only used as a broad
/// candidate signal here; exact route crossings come from local bank geometry.
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

    let coarse_cell =
        watershed::sample_surface(world_seed, ruleset_version, coordinate)?.coarse_cell();
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
    let same_crossing = crossings
        .last()
        .is_some_and(|last| kind != RouteCrossingKind::River && last.kind == kind);
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
        river_banks: None,
        lake_id: None,
        lake_banks: None,
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
