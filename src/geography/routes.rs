//! Bounded physical-feature assessment for candidate world-route corridors.
//!
//! This is a straight-line probe, not a pathfinder or permission to cross obstacles.

use super::hydrology::SurfaceWaterKind;
use super::{hydrology, terrain, MacroCoordinate};

pub const MAX_CORRIDOR_SAMPLES: usize = 16;
const SAMPLE_SPACING: i128 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RouteCorridorAssessment {
    pub sampled_points: u8,
    pub undersampled: bool,
    pub ocean_samples: u8,
    pub lake_samples: u8,
    pub river_samples: u8,
    pub unresolved_water_samples: u8,
    pub mountain_barrier_samples: u8,
    pub max_mountain_ridge_strength_q16: u32,
}

impl RouteCorridorAssessment {
    pub const fn has_confirmed_water_crossing(self) -> bool {
        self.ocean_samples > 0 || self.lake_samples > 0 || self.river_samples > 0
    }

    pub const fn has_unresolved_water(self) -> bool {
        self.unresolved_water_samples > 0
    }
}

/// Sample a bounded set of points along the direct corridor between two anchors.
///
/// Long corridors are marked undersampled. Water and mountain signals remain
/// distinct so later route selection cannot mistake uncertain terrain for land.
/// Assess an existing world route using its persisted endpoint geography.
pub fn assess_world_route(
    world: &crate::model::World,
    route_id: &str,
) -> Option<RouteCorridorAssessment> {
    let route = world.route_by_id(route_id)?;
    let start = world
        .location_geography
        .iter()
        .find(|record| record.location_id == route.location_a_id)?
        .anchor;
    let end = world
        .location_geography
        .iter()
        .find(|record| record.location_id == route.location_b_id)?
        .anchor;
    assess_corridor(
        world.geography_seed?,
        world.geography_ruleset_version,
        start,
        end,
    )
}

pub fn assess_corridor(
    world_seed: u64,
    ruleset_version: u32,
    start: MacroCoordinate,
    end: MacroCoordinate,
) -> Option<RouteCorridorAssessment> {
    if start.stratum != 0 || end.stratum != 0 {
        return None;
    }

    let dx = i128::from(end.x) - i128::from(start.x);
    let dy = i128::from(end.y) - i128::from(start.y);
    let distance = dx.unsigned_abs().max(dy.unsigned_abs());
    let required = if distance == 0 {
        0
    } else {
        (distance + SAMPLE_SPACING as u128 - 1) / SAMPLE_SPACING as u128
    };
    let steps = required.min((MAX_CORRIDOR_SAMPLES - 1) as u128) as usize;
    let mut result = RouteCorridorAssessment {
        undersampled: required > steps as u128,
        ..Default::default()
    };

    for index in 0..=steps {
        let fraction = index as i128;
        let denominator = (steps as i128).max(1);
        let x = i128::from(start.x) + dx * fraction / denominator;
        let y = i128::from(start.y) + dy * fraction / denominator;
        let coordinate = MacroCoordinate::new(i64::try_from(x).ok()?, i64::try_from(y).ok()?);
        let surface = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;

        result.sampled_points += 1;
        result.max_mountain_ridge_strength_q16 = result
            .max_mountain_ridge_strength_q16
            .max(surface.mountain_ridge_strength_q16());
        if surface.is_mountain_barrier() {
            result.mountain_barrier_samples += 1;
        }

        if let Some(water) = hydrology::sample_surface(world_seed, ruleset_version, coordinate) {
            match water.kind() {
                SurfaceWaterKind::Ocean => result.ocean_samples += 1,
                SurfaceWaterKind::Lake(_) => result.lake_samples += 1,
                SurfaceWaterKind::River(_) => result.river_samples += 1,
                SurfaceWaterKind::UnresolvedRiver(_) => result.unresolved_water_samples += 1,
            }
        }
    }

    Some(result)
}

#[cfg(test)]
#[path = "../../tests/unit/route_geography.rs"]
mod tests;
