//! Deterministic, renderer-independent surface climate and biome sampling.
//!
//! Temperature and moisture are regional fields sampled from the global macro
//! lattice. Elevation cools exposed high ground, while nearby ocean samples
//! increase moisture. Biome classification is a compact environmental layer,
//! not a planetary climate-zone model.

use super::{coast, terrain, MacroCoordinate};

const Q16_ONE: i64 = 1 << 16;
const TEMPERATURE_BROAD_CELL: i64 = 16_384;
const TEMPERATURE_REGIONAL_CELL: i64 = 5_120;
const MOISTURE_BROAD_CELL: i64 = 12_288;
const MOISTURE_REGIONAL_CELL: i64 = 4_096;

const TEMPERATURE_BROAD_CHANNEL: u32 = 0x5445_4D42;
const TEMPERATURE_REGIONAL_CHANNEL: u32 = 0x5445_4D52;
const MOISTURE_BROAD_CHANNEL: u32 = 0x4D4F_4953;
const MOISTURE_REGIONAL_CHANNEL: u32 = 0x4D4F_4952;

/// Surface biome classification derived from elevation and local conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Ocean,
    Coast,
    Desert,
    Grassland,
    TemperateForest,
    Taiga,
    Tundra,
    Alpine,
}

/// Deterministic environmental values for one surface macro-coordinate.
///
/// Temperature is signed Q16 in the range -1.0..=1.0. Moisture and
/// precipitation potential are unsigned Q16 in the range 0.0..=1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateSample {
    temperature_q16: i32,
    moisture_q16: u32,
    precipitation_q16: u32,
    biome: Biome,
}

impl ClimateSample {
    pub const fn temperature_q16(self) -> i32 {
        self.temperature_q16
    }

    pub const fn moisture_q16(self) -> u32 {
        self.moisture_q16
    }

    pub const fn precipitation_q16(self) -> u32 {
        self.precipitation_q16
    }

    pub const fn biome(self) -> Biome {
        self.biome
    }

    pub fn temperature(self) -> f32 {
        self.temperature_q16 as f32 / Q16_ONE as f32
    }

    pub fn moisture(self) -> f32 {
        self.moisture_q16 as f32 / Q16_ONE as f32
    }

    pub fn precipitation_potential(self) -> f32 {
        self.precipitation_q16 as f32 / Q16_ONE as f32
    }
}

/// Sample surface climate and biome data at a global macro-coordinate.
///
/// Returns None for underground strata. All authoritative calculations use
/// fixed-point integer arithmetic and depend only on the world seed, ruleset,
/// and global coordinate.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<ClimateSample> {
    if coordinate.stratum != 0 {
        return None;
    }

    let terrain = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;
    let temperature_broad = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        TEMPERATURE_BROAD_CELL,
        TEMPERATURE_BROAD_CHANNEL,
    );
    let temperature_regional = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        TEMPERATURE_REGIONAL_CELL,
        TEMPERATURE_REGIONAL_CHANNEL,
    );
    let base_temperature = (temperature_broad * 70 + temperature_regional * 30) / 100;
    let temperature = apply_elevation_cooling(base_temperature, terrain.elevation_q16());

    let moisture_broad = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        MOISTURE_BROAD_CELL,
        MOISTURE_BROAD_CHANNEL,
    );
    let moisture_regional = terrain::value_noise(
        world_seed,
        ruleset_version,
        coordinate,
        MOISTURE_REGIONAL_CELL,
        MOISTURE_REGIONAL_CHANNEL,
    );
    let moisture_field = (moisture_broad * 65 + moisture_regional * 35) / 100;
    let base_moisture = ((moisture_field + Q16_ONE) / 2).clamp(0, Q16_ONE);
    let ocean_influence = ocean_moisture_influence(world_seed, ruleset_version, coordinate);
    let moisture = apply_ocean_moisture(base_moisture, ocean_influence);

    let precipitation = precipitation_potential(moisture, temperature);
    let coast = coast::sample_surface(world_seed, ruleset_version, coordinate)
        .is_some_and(|sample| sample.is_coastal_land());
    let biome = classify_biome(terrain.elevation_q16(), temperature, moisture, coast);

    Some(ClimateSample {
        temperature_q16: temperature as i32,
        moisture_q16: moisture as u32,
        precipitation_q16: precipitation as u32,
        biome,
    })
}

fn apply_elevation_cooling(base_temperature: i64, elevation_q16: i32) -> i64 {
    let exposed_elevation = i64::from(elevation_q16).max(0);
    (base_temperature - exposed_elevation * 35 / 100).clamp(-Q16_ONE, Q16_ONE)
}

fn apply_ocean_moisture(base_moisture: i64, ocean_influence: i64) -> i64 {
    (base_moisture + ocean_influence * 45 / 100).clamp(0, Q16_ONE)
}

fn precipitation_potential(moisture: i64, temperature: i64) -> i64 {
    let warmth_factor = (3 * Q16_ONE / 4 + temperature.max(0) / 4).clamp(0, Q16_ONE);
    (moisture * warmth_factor / Q16_ONE).clamp(0, Q16_ONE)
}

fn classify_biome(elevation: i32, temperature: i64, moisture: i64, coast: bool) -> Biome {
    if elevation <= 0 {
        Biome::Ocean
    } else if i64::from(elevation) >= Q16_ONE * 72 / 100 {
        Biome::Alpine
    } else if coast {
        Biome::Coast
    } else if temperature <= -Q16_ONE * 58 / 100 {
        Biome::Tundra
    } else if temperature <= -Q16_ONE * 18 / 100 && moisture >= Q16_ONE / 4 {
        Biome::Taiga
    } else if temperature >= Q16_ONE / 5 && moisture <= Q16_ONE * 22 / 100 {
        Biome::Desert
    } else if temperature >= -Q16_ONE * 15 / 100 && moisture >= Q16_ONE * 62 / 100 {
        Biome::TemperateForest
    } else {
        Biome::Grassland
    }
}

fn ocean_moisture_influence(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> i64 {
    // Cardinal probes at increasing distances provide a smooth, bounded
    // approximation of maritime influence without a world-sized distance map.
    const PROBES: [(i64, i64); 4] = [
        (512, Q16_ONE * 5 / 8),
        (1_536, Q16_ONE * 3 / 8),
        (3_072, Q16_ONE / 4),
        (6_144, Q16_ONE / 8),
    ];
    let mut influence = 0;
    for (distance, strength) in PROBES {
        for (dx, dy) in [(distance, 0), (-distance, 0), (0, distance), (0, -distance)] {
            let Some(x) = coordinate.x.checked_add(dx) else {
                continue;
            };
            let Some(y) = coordinate.y.checked_add(dy) else {
                continue;
            };
            if terrain::sample_surface(world_seed, ruleset_version, MacroCoordinate::new(x, y))
                .is_some_and(|sample| !sample.is_land())
            {
                influence = influence.max(strength);
            }
        }
    }
    influence
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_reduces_temperature_without_affecting_ocean_level_samples() {
        let base = 20_000;
        assert!(apply_elevation_cooling(base, 40_000) < apply_elevation_cooling(base, 0));
        assert_eq!(apply_elevation_cooling(base, -20_000), base);
    }

    #[test]
    fn nearby_ocean_increases_moisture_without_exceeding_q16_range() {
        let base = Q16_ONE / 3;
        let influenced = apply_ocean_moisture(base, Q16_ONE / 2);
        assert!(influenced > base);
        assert!(influenced <= Q16_ONE);
        assert_eq!(apply_ocean_moisture(Q16_ONE, Q16_ONE), Q16_ONE);
    }

    #[test]
    fn biome_thresholds_are_explicit() {
        assert_eq!(classify_biome(0, 0, 0, false), Biome::Ocean);
        assert_eq!(classify_biome(1, 0, 0, true), Biome::Coast);
        assert_eq!(classify_biome(50_000, 0, 0, false), Biome::Alpine);
        assert_eq!(
            classify_biome(10_000, -40_000, 30_000, false),
            Biome::Tundra
        );
        assert_eq!(classify_biome(10_000, -20_000, 30_000, false), Biome::Taiga);
        assert_eq!(classify_biome(10_000, 20_000, 10_000, false), Biome::Desert);
        assert_eq!(
            classify_biome(10_000, 0, 50_000, false),
            Biome::TemperateForest
        );
        assert_eq!(classify_biome(10_000, 0, 20_000, false), Biome::Grassland);
    }

    #[test]
    fn precipitation_potential_stays_within_q16_range() {
        for moisture in [0, Q16_ONE / 2, Q16_ONE] {
            for temperature in [-Q16_ONE, 0, Q16_ONE] {
                let precipitation = precipitation_potential(moisture, temperature);
                assert!((0..=Q16_ONE).contains(&precipitation));
            }
        }
    }
}
