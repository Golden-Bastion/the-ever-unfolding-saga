//! Deterministic, renderer-independent surface elevation fields.
//!
//! This is the first terrain layer: broad-scale value noise creates connected
//! continental landmasses with smaller island and coastal variation. All noise
//! is sampled in global macro coordinates, so region generation order and
//! chunk boundaries cannot alter the result. Fixed-point interpolation avoids
//! platform-dependent floating-point decisions in the generated land mask.

use super::{coordinate_seed, MacroCoordinate};

const Q16_ONE: i64 = 1 << 16;
const CONTINENTAL_CELL_SAMPLES: i64 = 8_192;
const REGIONAL_CELL_SAMPLES: i64 = 3_072;
const DETAIL_CELL_SAMPLES: i64 = 768;

const CONTINENTAL_CHANNEL: u32 = 0x434F_4E54;
const REGIONAL_CHANNEL: u32 = 0x5245_4749;
const DETAIL_CHANNEL: u32 = 0x4445_5441;

/// A sampled surface elevation, represented as signed Q16 fixed-point data.
///
/// The normalized elevation range is approximately -1.0 through +1.0.
/// Positive elevations are land; zero and negative elevations are ocean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceTerrainSample {
    elevation_q16: i32,
}

impl SurfaceTerrainSample {
    /// Return the signed Q16 elevation value.
    pub const fn elevation_q16(self) -> i32 {
        self.elevation_q16
    }

    /// Return elevation normalized to approximately -1.0 through +1.0.
    ///
    /// This conversion is for presentation and downstream calculations; the
    /// authoritative land/ocean classification uses the fixed-point value.
    pub fn elevation(self) -> f32 {
        self.elevation_q16 as f32 / Q16_ONE as f32
    }

    /// Whether this sample lies above the generated sea-level threshold.
    pub const fn is_land(self) -> bool {
        self.elevation_q16 > 0
    }
}

/// Sample the deterministic surface elevation field at one global coordinate.
///
/// Returns None for underground strata; those will receive a separate
/// generation model rather than being treated as ordinary surface terrain.
/// The world seed and geography ruleset version are explicit inputs so saved
/// worlds can continue using the rules under which their geography was made.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<SurfaceTerrainSample> {
    if coordinate.stratum != 0 {
        return None;
    }

    let continental = value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        CONTINENTAL_CELL_SAMPLES,
        CONTINENTAL_CHANNEL,
    );
    let regional = value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        REGIONAL_CELL_SAMPLES,
        REGIONAL_CHANNEL,
    );
    let detail = value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        DETAIL_CELL_SAMPLES,
        DETAIL_CHANNEL,
    );

    // The broad field controls continent-scale landmasses. Regional and detail
    // fields break coastlines into peninsulas, bays, and smaller islands without
    // making neighboring samples independent.
    let elevation = (continental * 60 + regional * 27 + detail * 13) / 100;
    Some(SurfaceTerrainSample {
        elevation_q16: elevation.clamp(-Q16_ONE, Q16_ONE) as i32,
    })
}

fn value_noise(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
    cell_size: i64,
    channel: u32,
) -> i64 {
    let lattice_x = coordinate.x.div_euclid(cell_size);
    let lattice_y = coordinate.y.div_euclid(cell_size);
    let next_x = lattice_x + 1;
    let next_y = lattice_y + 1;

    let top_left = lattice_value(
        world_seed,
        ruleset_version,
        coordinate.stratum,
        lattice_x,
        lattice_y,
        channel,
    );
    let top_right = lattice_value(
        world_seed,
        ruleset_version,
        coordinate.stratum,
        next_x,
        lattice_y,
        channel,
    );
    let bottom_left = lattice_value(
        world_seed,
        ruleset_version,
        coordinate.stratum,
        lattice_x,
        next_y,
        channel,
    );
    let bottom_right = lattice_value(
        world_seed,
        ruleset_version,
        coordinate.stratum,
        next_x,
        next_y,
        channel,
    );

    let fraction_x = smooth_fraction(coordinate.x, cell_size);
    let fraction_y = smooth_fraction(coordinate.y, cell_size);
    let top = interpolate(top_left, top_right, fraction_x);
    let bottom = interpolate(bottom_left, bottom_right, fraction_x);
    interpolate(top, bottom, fraction_y)
}

fn lattice_value(
    world_seed: u64,
    ruleset_version: u32,
    stratum: i32,
    x: i64,
    y: i64,
    channel: u32,
) -> i64 {
    let coordinate = MacroCoordinate::in_stratum(x, y, stratum);
    let seed = coordinate_seed(world_seed, ruleset_version ^ channel, coordinate);
    let unit = (seed >> 48) as u16 as i64;
    (unit * (2 * Q16_ONE) / i64::from(u16::MAX)) - Q16_ONE
}

fn smooth_fraction(coordinate: i64, cell_size: i64) -> i64 {
    let linear = coordinate.rem_euclid(cell_size) * Q16_ONE / cell_size;
    linear * linear * (3 * Q16_ONE - 2 * linear) / (Q16_ONE * Q16_ONE)
}

fn interpolate(from: i64, to: i64, fraction: i64) -> i64 {
    from + (to - from) * fraction / Q16_ONE
}
