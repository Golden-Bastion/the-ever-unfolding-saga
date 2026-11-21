//! Deterministic, renderer-independent underground cave-network sampling.
//!
//! Underground strata contain sparse chambers connected by passages on a
//! shared signed global lattice. Each chamber is a stable network node;
//! neighboring nodes connect only when both exist and their shared edge is
//! open. Sampling a point inspects only its containing node and four neighbors.

use super::{coordinate_seed, MacroCoordinate};

/// Width of one underground network cell in global macro samples.
pub const NETWORK_CELL_SAMPLES: i64 = 64;

/// Half-width of a generated chamber, in macro samples.
pub const CHAMBER_RADIUS_SAMPLES: i64 = 8;

/// Half-width of a connecting passage, in macro samples.
pub const PASSAGE_HALF_WIDTH_SAMPLES: i64 = 1;

/// Direction bits in the cave-network connection mask.
pub mod direction {
    pub const NORTH: u8 = 1 << 0;
    pub const EAST: u8 = 1 << 1;
    pub const SOUTH: u8 = 1 << 2;
    pub const WEST: u8 = 1 << 3;
}

const HALF_CELL: i64 = NETWORK_CELL_SAMPLES / 2;
const NODE_CHANNEL: u32 = 0x4341_5645;
const EAST_EDGE_CHANNEL: u32 = 0x4341_5645 ^ 0x4541_5354;
const SOUTH_EDGE_CHANNEL: u32 = 0x4341_5645 ^ 0x534F_5554;
const EDGE_OPEN_PERCENT: u64 = 68;

/// The sampled underground space at one global coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndergroundSample {
    kind: UndergroundKind,
    network_cell: Option<MacroCoordinate>,
    connections: u8,
}

impl UndergroundSample {
    /// Whether this point is solid rock, a chamber, or a connecting passage.
    pub const fn kind(self) -> UndergroundKind {
        self.kind
    }

    /// Stable center coordinate identifying the containing chamber node.
    ///
    /// This is present only when the sampled point lies in generated cave
    /// space. It is derived from the global cell coordinate, not runtime IDs.
    pub const fn network_cell(self) -> Option<MacroCoordinate> {
        self.network_cell
    }

    /// Cardinal links from this chamber to neighboring chamber nodes.
    ///
    /// The mask is reciprocal: an east link on one node is the west link on
    /// its neighbor. Bits are zero for solid-rock samples.
    pub const fn connections(self) -> u8 {
        self.connections
    }

    pub const fn is_open(self) -> bool {
        !matches!(self.kind, UndergroundKind::SolidRock)
    }
}

/// Coarse classification of generated underground space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndergroundKind {
    SolidRock,
    Chamber,
    Passage,
}

/// Sample underground geography at a global coordinate.
///
/// Only negative strata are underground. Strata are independent deterministic
/// layers; greater depth modestly increases chamber frequency. This is a
/// topological cave foundation, not a geological material or fluid simulation.
/// The query is bounded to one chamber and at most four neighboring nodes.
pub fn sample_underground(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<UndergroundSample> {
    if coordinate.stratum >= 0 {
        return None;
    }

    let cell_x = coordinate.x.div_euclid(NETWORK_CELL_SAMPLES);
    let cell_y = coordinate.y.div_euclid(NETWORK_CELL_SAMPLES);
    let center = cell_center(cell_x, cell_y, coordinate.stratum)?;
    if !node_exists(world_seed, ruleset_version, cell_x, cell_y, coordinate.stratum) {
        return Some(solid_sample());
    }

    let connections = connection_mask(
        world_seed,
        ruleset_version,
        cell_x,
        cell_y,
        coordinate.stratum,
    );
    let dx = coordinate.x.checked_sub(center.x)?;
    let dy = coordinate.y.checked_sub(center.y)?;

    let kind = if dx.unsigned_abs() <= CHAMBER_RADIUS_SAMPLES as u64
        && dy.unsigned_abs() <= CHAMBER_RADIUS_SAMPLES as u64
        && (dx * dx + dy * dy)
            <= CHAMBER_RADIUS_SAMPLES * CHAMBER_RADIUS_SAMPLES
    {
        UndergroundKind::Chamber
    } else if lies_on_open_passage(dx, dy, connections) {
        UndergroundKind::Passage
    } else {
        return Some(solid_sample());
    };

    Some(UndergroundSample {
        kind,
        network_cell: Some(center),
        connections,
    })
}

fn solid_sample() -> UndergroundSample {
    UndergroundSample {
        kind: UndergroundKind::SolidRock,
        network_cell: None,
        connections: 0,
    }
}

fn cell_center(cell_x: i64, cell_y: i64, stratum: i32) -> Option<MacroCoordinate> {
    Some(MacroCoordinate::in_stratum(
        cell_x.checked_mul(NETWORK_CELL_SAMPLES)?.checked_add(HALF_CELL)?,
        cell_y.checked_mul(NETWORK_CELL_SAMPLES)?.checked_add(HALF_CELL)?,
        stratum,
    ))
}

fn node_exists(
    world_seed: u64,
    ruleset_version: u32,
    cell_x: i64,
    cell_y: i64,
    stratum: i32,
) -> bool {
    let depth = stratum.unsigned_abs().min(8) as u64;
    let threshold = 52 + depth;
    let key = MacroCoordinate::in_stratum(cell_x, cell_y, stratum);
    coordinate_seed(world_seed, ruleset_version ^ NODE_CHANNEL, key) % 100 < threshold
}

fn connection_mask(
    world_seed: u64,
    ruleset_version: u32,
    cell_x: i64,
    cell_y: i64,
    stratum: i32,
) -> u8 {
    let mut mask = 0;
    for (dx, dy, bit) in [
        (0_i64, -1_i64, direction::NORTH),
        (1, 0, direction::EAST),
        (0, 1, direction::SOUTH),
        (-1, 0, direction::WEST),
    ] {
        let (Some(neighbor_x), Some(neighbor_y)) =
            (cell_x.checked_add(dx), cell_y.checked_add(dy))
        else {
            continue;
        };
        if !node_exists(
            world_seed,
            ruleset_version,
            neighbor_x,
            neighbor_y,
            stratum,
        ) {
            continue;
        }
        if edge_is_open(
            world_seed,
            ruleset_version,
            cell_x,
            cell_y,
            dx,
            dy,
            stratum,
        ) {
            mask |= bit;
        }
    }
    mask
}

fn edge_is_open(
    world_seed: u64,
    ruleset_version: u32,
    cell_x: i64,
    cell_y: i64,
    dx: i64,
    dy: i64,
    stratum: i32,
) -> bool {
    // Every shared edge has one canonical key, whichever side samples it.
    let (edge_x, edge_y, channel) = if dx == 1 {
        (cell_x, cell_y, EAST_EDGE_CHANNEL)
    } else if dx == -1 {
        let Some(x) = cell_x.checked_sub(1) else {
            return false;
        };
        (x, cell_y, EAST_EDGE_CHANNEL)
    } else if dy == 1 {
        (cell_x, cell_y, SOUTH_EDGE_CHANNEL)
    } else {
        let Some(y) = cell_y.checked_sub(1) else {
            return false;
        };
        (cell_x, y, SOUTH_EDGE_CHANNEL)
    };
    let edge_key = MacroCoordinate::in_stratum(edge_x, edge_y, stratum);
    coordinate_seed(world_seed, ruleset_version ^ channel, edge_key) % 100
        < EDGE_OPEN_PERCENT
}

fn lies_on_open_passage(dx: i64, dy: i64, connections: u8) -> bool {
    let half_width = PASSAGE_HALF_WIDTH_SAMPLES as u64;
    let half_length = HALF_CELL as u64;
    (connections & direction::NORTH != 0
        && dx.unsigned_abs() <= half_width
        && dy < 0
        && dy.unsigned_abs() <= half_length)
        || (connections & direction::EAST != 0
            && dy.unsigned_abs() <= half_width
            && dx > 0
            && dx.unsigned_abs() <= half_length)
        || (connections & direction::SOUTH != 0
            && dx.unsigned_abs() <= half_width
            && dy > 0
            && dy.unsigned_abs() <= half_length)
        || (connections & direction::WEST != 0
            && dy.unsigned_abs() <= half_width
            && dx < 0
            && dx.unsigned_abs() <= half_length)
}


#[cfg(test)]
#[path = "../../tests/unit/underground.rs"]
mod tests;
