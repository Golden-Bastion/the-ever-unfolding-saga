//! Renderer-independent macro-geography coordinates and deterministic sampling keys.
//!
//! A region is a fixed 256 × 256 chunk in the global macro-sample lattice. Global
//! sample coordinates are signed so expansion works in every direction from the
//! world origin. Vertical strata keep underground geography addressable without
//! changing the shared horizontal coordinate system. The terrain submodule
//! builds deterministic surface elevation fields on this coordinate contract.

use serde::{Deserialize, Serialize};

pub mod terrain;

/// Number of macro-geography samples along each side of a region chunk.
pub const REGION_SAMPLE_RESOLUTION: u16 = 256;

/// Version of the deterministic macro-geography coordinate/sampling contract.
pub const GEOGRAPHY_RULESET_VERSION: u32 = 1;

/// A signed coordinate identifying one region-sized chunk and vertical stratum.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RegionCoordinate {
    pub x: i64,
    pub y: i64,
    /// Zero is the surface; negative values identify underground strata.
    pub stratum: i32,
}

impl RegionCoordinate {
    pub const ORIGIN: Self = Self {
        x: 0,
        y: 0,
        stratum: 0,
    };

    /// Construct a coordinate on the surface stratum.
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y, stratum: 0 }
    }

    /// Construct a coordinate on an explicit vertical geography stratum.
    pub const fn in_stratum(x: i64, y: i64, stratum: i32) -> Self {
        Self { x, y, stratum }
    }

    /// Convert a region-local sample position into the global sample lattice.
    ///
    /// Returns no coordinate for local positions outside the 256 × 256 chunk
    /// or when the global coordinate would overflow the signed coordinate type.
    pub fn global_sample(self, local_x: u16, local_y: u16) -> Option<MacroCoordinate> {
        if local_x >= REGION_SAMPLE_RESOLUTION || local_y >= REGION_SAMPLE_RESOLUTION {
            return None;
        }

        let resolution = i64::from(REGION_SAMPLE_RESOLUTION);
        Some(MacroCoordinate {
            x: self
                .x
                .checked_mul(resolution)?
                .checked_add(i64::from(local_x))?,
            y: self
                .y
                .checked_mul(resolution)?
                .checked_add(i64::from(local_y))?,
            stratum: self.stratum,
        })
    }
}

/// A signed coordinate in the shared global macro-geography sample lattice.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MacroCoordinate {
    pub x: i64,
    pub y: i64,
    /// Vertical geography layer, matching the owning region's stratum.
    pub stratum: i32,
}

impl MacroCoordinate {
    pub const ORIGIN: Self = Self {
        x: 0,
        y: 0,
        stratum: 0,
    };

    /// Construct a coordinate on the surface stratum.
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y, stratum: 0 }
    }

    /// Construct a coordinate on an explicit vertical geography stratum.
    pub const fn in_stratum(x: i64, y: i64, stratum: i32) -> Self {
        Self { x, y, stratum }
    }

    /// Resolve this global sample into its owning region chunk and local index.
    ///
    /// Euclidean division is intentional: negative coordinates map to negative
    /// region chunks with non-negative local sample indices.
    pub fn region_and_local(self) -> (RegionCoordinate, u16, u16) {
        let resolution = i64::from(REGION_SAMPLE_RESOLUTION);
        let region = RegionCoordinate {
            x: self.x.div_euclid(resolution),
            y: self.y.div_euclid(resolution),
            stratum: self.stratum,
        };
        let local_x = self.x.rem_euclid(resolution) as u16;
        let local_y = self.y.rem_euclid(resolution) as u16;
        (region, local_x, local_y)
    }
}

/// Derive a stable, order-independent seed for a macro-geography sample.
///
/// The seed depends only on the world seed, geography ruleset version, and
/// signed global sample coordinate including stratum. It does not depend on
/// generation order, runtime entity IDs, or mutable display names. This is a
/// deterministic non-cryptographic mixer, not a security primitive.
pub fn coordinate_seed(world_seed: u64, ruleset_version: u32, coordinate: MacroCoordinate) -> u64 {
    let mut value = mix64(world_seed ^ 0x6D61_6372_6F5F_6765);
    value = mix64(value ^ u64::from(ruleset_version).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    value = mix64(value ^ (coordinate.x as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    value = mix64(value ^ (coordinate.y as u64).wrapping_mul(0x94D0_49BB_1331_11EB));
    mix64(value ^ (coordinate.stratum as u32 as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93))
}

fn mix64(value: u64) -> u64 {
    let mut value = value;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
#[path = "../tests/unit/geography.rs"]
mod tests;
