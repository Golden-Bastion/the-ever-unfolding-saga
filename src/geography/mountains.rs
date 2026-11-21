//! Deterministic, renderer-independent mountain-range sampling.
//!
//! A warped ridged field creates broad, connected mountain belts. Mountain
//! uplift is applied to land elevation before climate and drainage sampling.
//! Ruleset 1 remains unchanged; the mountain field is enabled from ruleset 2.

use super::{terrain, MacroCoordinate};

const Q16_ONE: i64 = 1 << 16;
const RANGE_CELL_SAMPLES: i64 = 12_288;
const REGIONAL_CELL_SAMPLES: i64 = 4_096;
const WARP_CELL_SAMPLES: i64 = 16_384;
const WARP_DISTANCE_SAMPLES: i64 = 2_048;
const RIDGE_THRESHOLD_Q16: i64 = 36_000;

/// Ridge-strength threshold used by the shared surface-terrain barrier signal.
pub const MOUNTAIN_BARRIER_THRESHOLD_Q16: u32 = 42_000;
const MAX_UPLIFT_Q16: i64 = 31_457; // approximately 0.48 normalized elevation

const RANGE_CHANNEL: u32 = 0x4D4F_554E;
const REGIONAL_CHANNEL: u32 = 0x4D52_4547;
const WARP_X_CHANNEL: u32 = 0x4D57_4158;
const WARP_Y_CHANNEL: u32 = 0x4D57_4159;

/// Mountain intensity and its contribution to the surface elevation field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountainSample {
    ridge_strength_q16: u32,
    uplift_q16: i32,
}

impl MountainSample {
    /// Normalized ridge strength in unsigned Q16.
    pub const fn ridge_strength_q16(self) -> u32 {
        self.ridge_strength_q16
    }

    /// Positive elevation added by this range, in signed Q16.
    pub const fn uplift_q16(self) -> i32 {
        self.uplift_q16
    }

    /// Whether the sample is classified as part of a mountain range.
    pub const fn is_mountain(self) -> bool {
        self.ridge_strength_q16 >= MOUNTAIN_BARRIER_THRESHOLD_Q16
    }
}

/// Sample mountain ranges at one global surface coordinate.
///
/// The field is active from geography ruleset 2 onward. Older rulesets return
/// zero uplift so existing saved worlds retain their original terrain.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<MountainSample> {
    if coordinate.stratum != 0 {
        return None;
    }
    let base_elevation =
        terrain::base_surface_elevation_q16(world_seed, ruleset_version, coordinate)?;
    sample_with_base(world_seed, ruleset_version, coordinate, base_elevation)
}

pub(crate) fn sample_with_base(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
    base_elevation_q16: i32,
) -> Option<MountainSample> {
    if coordinate.stratum != 0 {
        return None;
    }
    if ruleset_version < 2 || base_elevation_q16 <= 0 {
        return Some(MountainSample {
            ridge_strength_q16: 0,
            uplift_q16: 0,
        });
    }

    let warp_x = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        WARP_CELL_SAMPLES,
        WARP_X_CHANNEL,
    ) * WARP_DISTANCE_SAMPLES
        / Q16_ONE;
    let warp_y = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        WARP_CELL_SAMPLES,
        WARP_Y_CHANNEL,
    ) * WARP_DISTANCE_SAMPLES
        / Q16_ONE;
    let warped = MacroCoordinate::new(
        coordinate.x.checked_add(warp_x).unwrap_or(coordinate.x),
        coordinate.y.checked_add(warp_y).unwrap_or(coordinate.y),
    );

    let broad_ridge = terrain::value_noise(
        world_seed,
        ruleset_version,
        warped,
        RANGE_CELL_SAMPLES,
        RANGE_CHANNEL,
    );
    // Zero crossings of a smooth field form long, branching ridge belts.
    let ridge_distance = Q16_ONE - broad_ridge.abs();
    let ridge_core = ((ridge_distance - RIDGE_THRESHOLD_Q16).max(0) * Q16_ONE
        / (Q16_ONE - RIDGE_THRESHOLD_Q16))
        .clamp(0, Q16_ONE);

    let regional = terrain::value_noise(
        world_seed,
        ruleset_version,
        warped,
        REGIONAL_CELL_SAMPLES,
        REGIONAL_CHANNEL,
    );
    let regional_factor =
        (Q16_ONE * 55 / 100 + (regional + Q16_ONE) * 45 / 200).clamp(Q16_ONE * 55 / 100, Q16_ONE);
    let ridge_strength = ridge_core * ridge_core / Q16_ONE * regional_factor / Q16_ONE;
    let uplift = ridge_strength * ridge_strength / Q16_ONE * MAX_UPLIFT_Q16 / Q16_ONE;

    Some(MountainSample {
        ridge_strength_q16: ridge_strength as u32,
        uplift_q16: uplift as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_ruleset_has_no_mountain_uplift() {
        let sample = sample_surface(42, 1, MacroCoordinate::new(1_024, -2_048)).unwrap();
        assert_eq!(sample.ridge_strength_q16(), 0);
        assert_eq!(sample.uplift_q16(), 0);
        assert!(!sample.is_mountain());
    }

    #[test]
    fn underground_strata_are_rejected() {
        assert!(sample_surface(42, 2, MacroCoordinate::in_stratum(0, 0, -1)).is_none());
    }
}
