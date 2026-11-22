//! Deterministic, renderer-independent river-channel extraction.
//!
//! Flow accumulation walks the reverse coarse watershed graph. Each query
//! examines at most MAX_CONTRIBUTING_CELLS accepted cells and sums their
//! climate-derived precipitation potential. The sum is a relative index, not
//! physical discharge or water volume; no generated hydrology state is stored.

use super::{climate, terrain, waterbodies, watershed, MacroCoordinate};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

/// Hard upper bound on coarse cells included in one accumulation query.
pub const MAX_CONTRIBUTING_CELLS: usize = 128;

/// Minimum contributing coarse cells required to classify a river channel.
pub const MIN_RIVER_CONTRIBUTING_CELLS: u16 = 8;

/// Bounded estimate of the contributing land area and precipitation potential
/// draining through one coarse cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowAccumulation {
    contributing_cells: u16,
    precipitation_potential_sum_q16: u64,
    truncated: bool,
}

impl FlowAccumulation {
    /// Number of coarse watershed cells found to drain through this cell.
    /// The target cell itself is included for land samples.
    pub const fn contributing_cells(self) -> u16 {
        self.contributing_cells
    }

    /// Sum of the contributors' precipitation-potential samples.
    ///
    /// This unsigned Q16 × coarse-cell index is useful for comparing relative
    /// wetness across drainage areas. It is not a physical discharge value.
    /// When the query is truncated, this sum is also a lower bound.
    pub const fn precipitation_potential_sum_q16(self) -> u64 {
        self.precipitation_potential_sum_q16
    }

    /// Whether additional contributors were found beyond the work bound.
    /// When true, both contributing_cells and the precipitation sum are lower bounds.
    pub const fn is_truncated(self) -> bool {
        self.truncated
    }

    pub const fn is_complete(self) -> bool {
        !self.truncated
    }
}

/// River-channel classification and its bounded contributing-area and
/// precipitation-potential estimates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiverSample {
    coarse_cell: MacroCoordinate,
    accumulation: FlowAccumulation,
}

impl RiverSample {
    /// Center of the coarse watershed cell represented by this sample.
    pub const fn coarse_cell(self) -> MacroCoordinate {
        self.coarse_cell
    }

    pub const fn accumulation(self) -> FlowAccumulation {
        self.accumulation
    }

    /// A truncated result confirms a channel only after the threshold is met.
    pub const fn is_channel(self) -> bool {
        self.accumulation.contributing_cells >= MIN_RIVER_CONTRIBUTING_CELLS
    }
}

/// Relative river width inferred from bounded contributing area.
///
/// These are categorical world-generation widths, not calibrated metres or
/// local-map tile counts. The classes grow downstream as more watershed cells
/// contribute to the channel. Truncated accumulation can establish the lower
/// classes, but the largest class should be read as a lower-bound estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiverWidthClass {
    Narrow,
    Medium,
    Wide,
    VeryWide,
}

impl RiverWidthClass {
    fn from_contributing_cells(cells: u16) -> Self {
        match cells {
            0..=15 => Self::Narrow,
            16..=31 => Self::Medium,
            32..=63 => Self::Wide,
            _ => Self::VeryWide,
        }
    }

    /// Whether the current width class is suitable for a natural ford.
    pub const fn is_fordable(self) -> bool {
        matches!(self, Self::Narrow | Self::Medium)
    }
}

/// Terminal or bounded continuation reached by tracing a river downstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiverOutletKind {
    Ocean,
    Lake,
    ClosedBasin,
    Land,
    Unresolved,
    SearchLimit,
}

/// Stable identity shared by river branches that drain to the same outlet.
///
/// The identity is derived from the outlet coordinate and terminal kind. A
/// search-limited trace uses its deterministic continuation coordinate and is
/// therefore explicitly provisional until a longer trace reaches the outlet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RiverId {
    pub outlet_coordinate: MacroCoordinate,
    pub outlet_kind: RiverOutletKind,
}

/// Direction in which a river flows between adjacent coarse watershed cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiverFlowDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

/// One centre point in a traced river course.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiverPathPoint {
    pub coordinate: MacroCoordinate,
    pub width: RiverWidthClass,
    pub contributing_cells: u16,
    pub downstream: Option<MacroCoordinate>,
    pub flow_direction: Option<RiverFlowDirection>,
}

/// The terminal point reached by a bounded downstream river trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiverOutlet {
    pub coordinate: MacroCoordinate,
    pub kind: RiverOutletKind,
}

/// A connected downstream course through the deterministic watershed graph.
///
/// Tributary traces share an identity when they reach the same outlet. The
/// path contains channel-cell centres in flow order; the final point records
/// its next drainage coordinate even when that destination is a lake or ocean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverNetwork {
    pub id: RiverId,
    pub path: Vec<RiverPathPoint>,
    pub outlet: RiverOutlet,
}

impl RiverSample {
    /// Relative width class derived from the bounded contributing-area count.
    pub const fn width_class(self) -> RiverWidthClass {
        RiverWidthClass::from_contributing_cells(self.accumulation.contributing_cells)
    }
}

/// Maximum number of channel cells included in one downstream course query.
pub const MAX_RIVER_NETWORK_STEPS: usize = 128;

/// Trace a confirmed river channel downstream through connected watershed cells.
///
/// The query is deterministic and bounded. It follows the existing strictly
/// downhill coarse graph, records channel centres and relative width changes,
/// and stops at an ocean, lake, closed basin, unresolved land continuation, or
/// the explicit step limit. It does not simulate water volume or search the
/// whole upstream catchment.
pub fn trace_network(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<RiverNetwork> {
    let initial = sample_surface(world_seed, ruleset_version, coordinate)?;
    if !initial.is_channel() {
        return None;
    }

    let mut current = initial.coarse_cell();
    let mut visited = BTreeSet::new();
    let mut path = Vec::new();
    let mut outlet = None;

    for step in 0..MAX_RIVER_NETWORK_STEPS {
        let key = (
            current.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES),
            current.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES),
        );
        if !visited.insert(key) {
            outlet = Some(RiverOutlet {
                coordinate: current,
                kind: RiverOutletKind::SearchLimit,
            });
            break;
        }

        let elevation = terrain::sample_surface(world_seed, ruleset_version, current)?;
        if !elevation.is_land() {
            outlet = Some(RiverOutlet {
                coordinate: current,
                kind: RiverOutletKind::Ocean,
            });
            break;
        }
        if let Some(lake) = waterbodies::sample_surface(world_seed, ruleset_version, current) {
            outlet = Some(RiverOutlet {
                coordinate: lake.outlet_cell(),
                kind: RiverOutletKind::Lake,
            });
            break;
        }

        let sample = sample_surface(world_seed, ruleset_version, current)?;
        if !sample.is_channel() {
            outlet = Some(RiverOutlet {
                coordinate: current,
                kind: if sample.accumulation().is_truncated() {
                    RiverOutletKind::Unresolved
                } else {
                    RiverOutletKind::Land
                },
            });
            break;
        }

        let drainage = watershed::sample_surface(world_seed, ruleset_version, current)?;
        let downstream = drainage.coarse_downstream();
        path.push(RiverPathPoint {
            coordinate: sample.coarse_cell(),
            width: sample.width_class(),
            contributing_cells: sample.accumulation().contributing_cells(),
            downstream,
            flow_direction: downstream.and_then(|next| flow_direction(current, next)),
        });

        match downstream {
            Some(next) => {
                current = next;
                if step + 1 == MAX_RIVER_NETWORK_STEPS {
                    outlet = Some(RiverOutlet {
                        coordinate: current,
                        kind: RiverOutletKind::SearchLimit,
                    });
                }
            }
            None => {
                outlet = Some(RiverOutlet {
                    coordinate: current,
                    kind: RiverOutletKind::ClosedBasin,
                });
            }
        }

        if outlet.is_some() {
            break;
        }
    }

    if path.is_empty() {
        return None;
    }

    let outlet = outlet.unwrap_or(RiverOutlet {
        coordinate: current,
        kind: RiverOutletKind::SearchLimit,
    });
    let id = RiverId {
        outlet_coordinate: outlet.coordinate,
        outlet_kind: outlet.kind,
    };
    Some(RiverNetwork { id, path, outlet })
}

fn flow_direction(from: MacroCoordinate, to: MacroCoordinate) -> Option<RiverFlowDirection> {
    let cell = watershed::WATERSHED_CELL_SAMPLES;
    let dx = to.x.div_euclid(cell) - from.x.div_euclid(cell);
    let dy = to.y.div_euclid(cell) - from.y.div_euclid(cell);
    match (dx.signum(), dy.signum()) {
        (0, -1) => Some(RiverFlowDirection::North),
        (1, -1) => Some(RiverFlowDirection::NorthEast),
        (1, 0) => Some(RiverFlowDirection::East),
        (1, 1) => Some(RiverFlowDirection::SouthEast),
        (0, 1) => Some(RiverFlowDirection::South),
        (-1, 1) => Some(RiverFlowDirection::SouthWest),
        (-1, 0) => Some(RiverFlowDirection::West),
        (-1, -1) => Some(RiverFlowDirection::NorthWest),
        _ => None,
    }
}

/// Sample river-channel data at a global surface coordinate.
///
/// Evaluate this once per coarse watershed cell. Ocean cells have no channel.
/// Underground strata and unrepresentable coarse-cell centers return None.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<RiverSample> {
    if coordinate.stratum != 0 {
        return None;
    }

    let watershed_sample = watershed::sample_surface(world_seed, ruleset_version, coordinate)?;
    let coarse_cell = watershed_sample.coarse_cell();
    let accumulation = if watershed_sample.is_ocean() {
        FlowAccumulation {
            contributing_cells: 0,
            precipitation_potential_sum_q16: 0,
            truncated: false,
        }
    } else {
        let start_precipitation_q16 =
            climate::sample_surface(world_seed, ruleset_version, coarse_cell)?.precipitation_q16();
        accumulate_upstream(
            world_seed,
            ruleset_version,
            coarse_cell,
            start_precipitation_q16,
        )?
    };

    Some(RiverSample {
        coarse_cell,
        accumulation,
    })
}

fn accumulate_upstream(
    world_seed: u64,
    ruleset_version: u32,
    start: MacroCoordinate,
    start_precipitation_q16: u32,
) -> Option<FlowAccumulation> {
    let start_key = (
        start.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES),
        start.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES),
    );
    let mut pending = VecDeque::from([start]);
    let mut accepted = BTreeSet::from([start_key]);
    let mut examined = BTreeSet::new();
    let mut contributing_cells = 1usize;
    let mut precipitation_potential_sum_q16 = u64::from(start_precipitation_q16);
    let mut truncated = false;

    while let Some(downstream) = pending.pop_front() {
        let downstream_x = downstream.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES);
        let downstream_y = downstream.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES);

        for (dx, dy, _) in watershed::NEIGHBORS {
            let Some(candidate_x) = downstream_x.checked_add(dx) else {
                continue;
            };
            let Some(candidate_y) = downstream_y.checked_add(dy) else {
                continue;
            };
            let candidate_key = (candidate_x, candidate_y);

            if accepted.contains(&candidate_key) || !examined.insert(candidate_key) {
                continue;
            }

            let Some(candidate) = watershed::cell_center(candidate_x, candidate_y) else {
                continue;
            };
            if !drains_into(
                world_seed,
                ruleset_version,
                candidate_x,
                candidate_y,
                candidate,
                downstream,
            ) {
                continue;
            }

            if contributing_cells == MAX_CONTRIBUTING_CELLS {
                truncated = true;
                break;
            }

            let candidate_precipitation_q16 =
                climate::sample_surface(world_seed, ruleset_version, candidate)?
                    .precipitation_q16();
            accepted.insert(candidate_key);
            contributing_cells += 1;
            precipitation_potential_sum_q16 = precipitation_potential_sum_q16
                .saturating_add(u64::from(candidate_precipitation_q16));
            pending.push_back(candidate);
        }

        if truncated {
            break;
        }
    }

    Some(FlowAccumulation {
        contributing_cells: contributing_cells as u16,
        precipitation_potential_sum_q16,
        truncated,
    })
}

fn drains_into(
    world_seed: u64,
    ruleset_version: u32,
    candidate_x: i64,
    candidate_y: i64,
    candidate: MacroCoordinate,
    downstream: MacroCoordinate,
) -> bool {
    let Some(elevation) = terrain::sample_surface(world_seed, ruleset_version, candidate) else {
        return false;
    };
    elevation.is_land()
        && watershed::downstream_cell(
            world_seed,
            ruleset_version,
            candidate_x,
            candidate_y,
            elevation.elevation_q16(),
        ) == Some(downstream)
}

#[cfg(test)]
#[path = "../../tests/unit/rivers.rs"]
mod tests;
