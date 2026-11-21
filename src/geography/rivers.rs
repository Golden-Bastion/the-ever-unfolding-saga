//! Deterministic, renderer-independent river-channel extraction.
//!
//! Flow accumulation walks the reverse coarse watershed graph. Each query
//! examines at most MAX_CONTRIBUTING_CELLS accepted cells and sums their
//! climate-derived precipitation potential. The sum is a relative index, not
//! physical discharge or water volume; no generated hydrology state is stored.

use super::{climate, terrain, watershed, MacroCoordinate};
use std::collections::{BTreeSet, VecDeque};

/// Hard upper bound on coarse cells included in one accumulation query.
pub const MAX_CONTRIBUTING_CELLS: usize = 128;

/// Minimum contributing coarse cells required to classify a river channel.
pub const MIN_RIVER_CONTRIBUTING_CELLS: u16 = 8;

/// Bounded estimate of the land area draining through one coarse cell.
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

/// River-channel classification and its contributing-area estimate.
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
        accumulate_upstream(world_seed, ruleset_version, coarse_cell)
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
) -> FlowAccumulation {
    let start_key = (
        start.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES),
        start.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES),
    );
    let mut pending = VecDeque::from([start]);
    let mut accepted = BTreeSet::from([start_key]);
    let mut examined = BTreeSet::new();
    let mut contributing_cells = 1usize;
    let mut precipitation_potential_sum_q16 =
        u64::from(precipitation_potential_q16(world_seed, ruleset_version, start));
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
            let Some(candidate_precipitation_q16) = upstream_precipitation_q16(
                world_seed,
                ruleset_version,
                candidate_x,
                candidate_y,
                candidate,
                downstream,
            ) else {
                continue;
            };

            if contributing_cells == MAX_CONTRIBUTING_CELLS {
                truncated = true;
                break;
            }

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

    FlowAccumulation {
        contributing_cells: contributing_cells as u16,
        precipitation_potential_sum_q16,
        truncated,
    }
}

fn upstream_precipitation_q16(
    world_seed: u64,
    ruleset_version: u32,
    candidate_x: i64,
    candidate_y: i64,
    candidate: MacroCoordinate,
    downstream: MacroCoordinate,
) -> Option<u32> {
    let elevation = terrain::sample_surface(world_seed, ruleset_version, candidate)?;
    if !elevation.is_land()
        || watershed::downstream_cell(
            world_seed,
            ruleset_version,
            candidate_x,
            candidate_y,
            elevation.elevation_q16(),
        ) != Some(downstream)
    {
        return None;
    }

    Some(precipitation_potential_q16(world_seed, ruleset_version, candidate))
}

fn precipitation_potential_q16(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> u32 {
    climate::sample_surface(world_seed, ruleset_version, coordinate)
        .map_or(0, |sample| sample.precipitation_q16())
}

#[cfg(test)]
#[path = "../../tests/unit/rivers.rs"]
mod tests;
