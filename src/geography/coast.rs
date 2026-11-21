//! Deterministic, renderer-independent shoreline classification.
//!
//! Shoreline data is derived from the shared global elevation field. Each
//! sample records which of its eight neighboring directions contain the
//! opposite terrain type, allowing land and water sides of a coast to share
//! the same boundary without chunk-local state.

use super::{terrain, MacroCoordinate};

/// Neighbor direction bits, starting at north and proceeding clockwise.
pub mod direction {
    pub const NORTH: u8 = 1 << 0;
    pub const NORTH_EAST: u8 = 1 << 1;
    pub const EAST: u8 = 1 << 2;
    pub const SOUTH_EAST: u8 = 1 << 3;
    pub const SOUTH: u8 = 1 << 4;
    pub const SOUTH_WEST: u8 = 1 << 5;
    pub const WEST: u8 = 1 << 6;
    pub const NORTH_WEST: u8 = 1 << 7;
}

/// Surface elevation and the directions containing the opposite terrain type.
///
/// For land samples, the mask points toward adjacent ocean samples. For ocean
/// samples, it points toward adjacent land samples. A zero mask means the
/// sample is not immediately adjacent to the opposite terrain type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShorelineSample {
    elevation_q16: i32,
    opposite_terrain_mask: u8,
}

impl ShorelineSample {
    pub const fn elevation_q16(self) -> i32 {
        self.elevation_q16
    }

    pub const fn is_land(self) -> bool {
        self.elevation_q16 > 0
    }

    pub const fn is_shoreline(self) -> bool {
        self.opposite_terrain_mask != 0
    }

    pub const fn is_coastal_land(self) -> bool {
        self.is_land() && self.is_shoreline()
    }

    pub const fn is_coastal_water(self) -> bool {
        !self.is_land() && self.is_shoreline()
    }

    /// Return the eight-bit mask of neighboring samples with opposite terrain.
    ///
    /// Bits are defined by the direction module. The mask is independent of
    /// which region chunk owns this global coordinate.
    pub const fn opposite_terrain_mask(self) -> u8 {
        self.opposite_terrain_mask
    }

    pub const fn has_opposite_neighbor(self, direction: u8) -> bool {
        self.opposite_terrain_mask & direction != 0
    }
}

/// Sample shoreline adjacency at a global surface coordinate.
///
/// The eight immediate neighbors are sampled from the same deterministic
/// elevation field. Returns None for underground strata.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<ShorelineSample> {
    let terrain = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;
    let is_land = terrain.is_land();
    let neighbors = [
        (0, -1, direction::NORTH),
        (1, -1, direction::NORTH_EAST),
        (1, 0, direction::EAST),
        (1, 1, direction::SOUTH_EAST),
        (0, 1, direction::SOUTH),
        (-1, 1, direction::SOUTH_WEST),
        (-1, 0, direction::WEST),
        (-1, -1, direction::NORTH_WEST),
    ];

    let mut opposite_terrain_mask = 0;
    for (dx, dy, bit) in neighbors {
        let Some(x) = coordinate.x.checked_add(dx) else {
            continue;
        };
        let Some(y) = coordinate.y.checked_add(dy) else {
            continue;
        };
        let neighbor = MacroCoordinate::new(x, y);
        if terrain::sample_surface(world_seed, ruleset_version, neighbor)
            .is_some_and(|sample| sample.is_land() != is_land)
        {
            opposite_terrain_mask |= bit;
        }
    }

    Some(ShorelineSample {
        elevation_q16: terrain.elevation_q16(),
        opposite_terrain_mask,
    })
}
