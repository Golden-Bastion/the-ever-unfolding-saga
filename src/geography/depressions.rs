//! Deterministic, renderer-independent depression and spill-level analysis.
//!
//! A query follows the coarse watershed to its closed inland outlet, then runs
//! a bounded minimax search over a local window of coarse terrain samples. The
//! result describes topographic basin potential; it does not claim that a lake
//! is permanently filled or simulate rainfall, discharge, or water volume.

use super::{terrain, watershed, MacroCoordinate};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// Radius of the local depression window, measured in coarse watershed cells.
pub const DEPRESSION_SEARCH_RADIUS_CELLS: i64 = 8;

const WINDOW_SIDE: usize = (DEPRESSION_SEARCH_RADIUS_CELLS as usize * 2) + 1;
const WINDOW_CELL_COUNT: usize = WINDOW_SIDE * WINDOW_SIDE;
const CENTER_INDEX: usize = WINDOW_CELL_COUNT / 2;

/// A bounded description of a closed inland topographic depression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepressionSample {
    outlet_cell: MacroCoordinate,
    floor_elevation_q16: i32,
    spill_elevation_q16: i32,
    basin_cell_count: u16,
    basin_footprint: [u64; 5],
}

impl DepressionSample {
    /// Coarse watershed cell at the floor of the closed basin.
    pub const fn outlet_cell(self) -> MacroCoordinate {
        self.outlet_cell
    }

    /// Terrain elevation at the basin floor, in signed Q16.
    pub const fn floor_elevation_q16(self) -> i32 {
        self.floor_elevation_q16
    }

    /// Lowest local escape level, in signed Q16.
    pub const fn spill_elevation_q16(self) -> i32 {
        self.spill_elevation_q16
    }

    /// Vertical terrain difference between the basin floor and its spill level.
    pub const fn depth_q16(self) -> i32 {
        self.spill_elevation_q16 - self.floor_elevation_q16
    }

    /// Number of connected coarse cells below the spill level, including the floor.
    pub const fn basin_cell_count(self) -> u16 {
        self.basin_cell_count
    }

    /// Whether a coarse watershed cell belongs to this connected basin footprint.
    pub fn contains_coarse_cell(self, coordinate: MacroCoordinate) -> bool {
        if coordinate.stratum != 0 {
            return false;
        }
        let basin_x = self
            .outlet_cell
            .x
            .div_euclid(watershed::WATERSHED_CELL_SAMPLES);
        let basin_y = self
            .outlet_cell
            .y
            .div_euclid(watershed::WATERSHED_CELL_SAMPLES);
        let cell_x = coordinate.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES);
        let cell_y = coordinate.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES);
        let Some(offset_x) = cell_x
            .checked_sub(basin_x)
            .and_then(|value| value.checked_add(DEPRESSION_SEARCH_RADIUS_CELLS))
        else {
            return false;
        };
        let Some(offset_y) = cell_y
            .checked_sub(basin_y)
            .and_then(|value| value.checked_add(DEPRESSION_SEARCH_RADIUS_CELLS))
        else {
            return false;
        };
        if !(0..WINDOW_SIDE as i64).contains(&offset_x)
            || !(0..WINDOW_SIDE as i64).contains(&offset_y)
        {
            return false;
        }
        let index = window_index(offset_x as usize, offset_y as usize);
        self.basin_footprint[index / 64] & (1_u64 << (index % 64)) != 0
    }
}

/// Resolve the local depression reached from a global surface coordinate.
///
/// Only closed inland watershed outlets produce a result. Ocean outlets and
/// bounded traces that have not reached a terminal outlet return None. For a
/// closed basin, a fixed 17 × 17 coarse-cell window estimates the lowest escape
/// level with a minimax flood and counts the connected terrain below that level.
/// Results are deterministic and independent of region ownership.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<DepressionSample> {
    if coordinate.stratum != 0 {
        return None;
    }

    let outlet = watershed::trace_to_outlet(world_seed, ruleset_version, coordinate)?;
    if outlet.kind() != watershed::OutletKind::ClosedBasin {
        return None;
    }

    let outlet_cell = outlet.coordinate();
    let coarse_x = outlet_cell.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES);
    let coarse_y = outlet_cell.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES);
    let mut elevations = [0_i32; WINDOW_CELL_COUNT];

    for y in 0..WINDOW_SIDE {
        for x in 0..WINDOW_SIDE {
            let offset_x = x as i64 - DEPRESSION_SEARCH_RADIUS_CELLS;
            let offset_y = y as i64 - DEPRESSION_SEARCH_RADIUS_CELLS;
            let sample_x = coarse_x.checked_add(offset_x)?;
            let sample_y = coarse_y.checked_add(offset_y)?;
            let sample_coordinate = watershed::cell_center(sample_x, sample_y)?;
            elevations[window_index(x, y)] =
                terrain::sample_surface(world_seed, ruleset_version, sample_coordinate)?
                    .elevation_q16();
        }
    }

    let floor_elevation_q16 = elevations[CENTER_INDEX];
    let spill_elevation_q16 = minimax_spill_elevation(&elevations)?;
    let depth_q16 = spill_elevation_q16.checked_sub(floor_elevation_q16)?;
    if depth_q16 <= 0 {
        return None;
    }

    let basin_footprint = basin_footprint(&elevations, spill_elevation_q16);
    let basin_cell_count = basin_footprint
        .iter()
        .map(|word| word.count_ones())
        .sum::<u32>() as u16;
    if basin_cell_count == 0 {
        return None;
    }

    Some(DepressionSample {
        outlet_cell,
        floor_elevation_q16,
        spill_elevation_q16,
        basin_cell_count,
        basin_footprint,
    })
}

fn window_index(x: usize, y: usize) -> usize {
    y * WINDOW_SIDE + x
}

/// Find the lowest possible maximum terrain height along a path from the
/// window's center to any perimeter cell (the local spill elevation).
fn minimax_spill_elevation(elevations: &[i32; WINDOW_CELL_COUNT]) -> Option<i32> {
    let mut best = [i32::MAX; WINDOW_CELL_COUNT];
    let mut frontier = BinaryHeap::new();

    for y in 0..WINDOW_SIDE {
        for x in 0..WINDOW_SIDE {
            if x != 0 && y != 0 && x + 1 != WINDOW_SIDE && y + 1 != WINDOW_SIDE {
                continue;
            }

            let index = window_index(x, y);
            best[index] = elevations[index];
            frontier.push(Reverse((elevations[index], index)));
        }
    }

    while let Some(Reverse((spill_level, index))) = frontier.pop() {
        if spill_level != best[index] {
            continue;
        }
        if index == CENTER_INDEX {
            return Some(spill_level);
        }

        let x = index % WINDOW_SIDE;
        let y = index / WINDOW_SIDE;
        for (dx, dy, _) in watershed::NEIGHBORS {
            let next_x = x as i64 + dx;
            let next_y = y as i64 + dy;
            if !(0..WINDOW_SIDE as i64).contains(&next_x)
                || !(0..WINDOW_SIDE as i64).contains(&next_y)
            {
                continue;
            }

            let next_index = window_index(next_x as usize, next_y as usize);
            let candidate_level = spill_level.max(elevations[next_index]);
            if candidate_level < best[next_index] {
                best[next_index] = candidate_level;
                frontier.push(Reverse((candidate_level, next_index)));
            }
        }
    }

    None
}

/// Count the center-connected footprint below the resolved spill level.
fn basin_footprint(elevations: &[i32; WINDOW_CELL_COUNT], spill_level: i32) -> [u64; 5] {
    if elevations[CENTER_INDEX] >= spill_level {
        return [0; 5];
    }

    let mut footprint = [0_u64; 5];
    let mut visited = [false; WINDOW_CELL_COUNT];
    let mut pending = VecDeque::from([CENTER_INDEX]);
    visited[CENTER_INDEX] = true;
    while let Some(index) = pending.pop_front() {
        footprint[index / 64] |= 1_u64 << (index % 64);
        let x = index % WINDOW_SIDE;
        let y = index / WINDOW_SIDE;

        for (dx, dy, _) in watershed::NEIGHBORS {
            let next_x = x as i64 + dx;
            let next_y = y as i64 + dy;
            if !(0..WINDOW_SIDE as i64).contains(&next_x)
                || !(0..WINDOW_SIDE as i64).contains(&next_y)
            {
                continue;
            }

            let next_index = window_index(next_x as usize, next_y as usize);
            if visited[next_index] || elevations[next_index] >= spill_level {
                continue;
            }

            visited[next_index] = true;
            pending.push_back(next_index);
        }
    }

    footprint
}

#[cfg(test)]
fn count_basin_cells(elevations: &[i32; WINDOW_CELL_COUNT], spill_level: i32) -> u16 {
    basin_footprint(elevations, spill_level)
        .iter()
        .map(|word| word.count_ones())
        .sum::<u32>() as u16
}

#[cfg(test)]
#[path = "../../tests/unit/depressions.rs"]
mod tests;
