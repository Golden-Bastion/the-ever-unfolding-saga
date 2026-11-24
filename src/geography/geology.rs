//! Deterministic, renderer-independent underground geological materials.
//!
//! Geological formations are sampled independently from cave topology. Every
//! underground coordinate belongs to a material field, including solid rock;
//! chambers and passages do not create, remove, or reshape geological layers.
//! The material families are stylized world-building categories inspired by
//! granite, slate, and basalt rather than literal mineralogical simulations.

use super::{terrain, MacroCoordinate};

const Q16_ONE: i64 = 1 << 16;
const MAX_DEPTH_BIAS: u64 = 16;

const BROAD_CELL_SAMPLES: i64 = 4_096;
const REGIONAL_CELL_SAMPLES: i64 = 1_024;
const DETAIL_CELL_SAMPLES: i64 = 256;

const BROAD_CHANNEL: u32 = 0x4745_4F42;
const REGIONAL_CHANNEL: u32 = 0x4745_4F52;
const DETAIL_CHANNEL: u32 = 0x4745_4F44;

/// Stylized geological material families found in underground strata.
///
/// These names communicate broad material character. They do not imply
/// real-world mineral composition, exact geological formation, or mining rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeologicalMaterial {
    /// Pale, dense stone with a coarse crystalline character.
    PaleGranite,
    /// Dark, layered stone with pronounced planar fractures.
    VeilSlate,
    /// Dense, dark volcanic stone with a fine-grained character.
    CinderBasalt,
}

impl GeologicalMaterial {
    /// Relative hardness for future material-aware systems, in unsigned Q16.
    ///
    /// These are game-design values, not physical measurements.
    pub const fn hardness_q16(self) -> u16 {
        match self {
            Self::PaleGranite => 58_000,
            Self::VeilSlate => 36_000,
            Self::CinderBasalt => 54_000,
        }
    }

    /// Relative tendency to split along fractures, in unsigned Q16.
    ///
    /// These are game-design values, not physical measurements.
    pub const fn fracture_q16(self) -> u16 {
        match self {
            Self::PaleGranite => 18_000,
            Self::VeilSlate => 52_000,
            Self::CinderBasalt => 31_000,
        }
    }
}

/// Geological material sampled at one global underground coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeologySample {
    material: GeologicalMaterial,
}

impl GeologySample {
    pub const fn material(self) -> GeologicalMaterial {
        self.material
    }

    pub const fn hardness_q16(self) -> u16 {
        self.material.hardness_q16()
    }

    pub const fn fracture_q16(self) -> u16 {
        self.material.fracture_q16()
    }
}

/// Sample the underground geological material field at a global coordinate.
///
/// Returns None for the surface and positive strata. The field is continuous
/// and deterministic across region boundaries, varies by horizontal location
/// and depth, and is independent of whether cave space exists at the coordinate.
/// It does not mutate or persist generated geography.
pub fn sample_geology(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<GeologySample> {
    if coordinate.stratum >= 0 {
        return None;
    }

    let broad = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        BROAD_CELL_SAMPLES,
        BROAD_CHANNEL,
    );
    let regional = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        REGIONAL_CELL_SAMPLES,
        REGIONAL_CHANNEL,
    );
    let detail = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        DETAIL_CELL_SAMPLES,
        DETAIL_CHANNEL,
    );

    let horizontal_field = (broad * 55 + regional * 30 + detail * 15) / 100;
    let depth = u64::from(coordinate.stratum.unsigned_abs()).min(MAX_DEPTH_BIAS);
    let depth_field = (depth as i64 * Q16_ONE / MAX_DEPTH_BIAS as i64) - Q16_ONE / 2;
    // Depth gently shifts the material balance toward darker, denser formations
    // without overriding large-scale horizontal variation.
    let material_field = horizontal_field * 3 / 4 + depth_field / 4;

    let material = if material_field < -Q16_ONE / 3 {
        GeologicalMaterial::PaleGranite
    } else if material_field > Q16_ONE / 3 {
        GeologicalMaterial::CinderBasalt
    } else {
        GeologicalMaterial::VeilSlate
    };

    Some(GeologySample { material })
}

#[cfg(test)]
#[path = "../../tests/unit/geology.rs"]
mod tests;
