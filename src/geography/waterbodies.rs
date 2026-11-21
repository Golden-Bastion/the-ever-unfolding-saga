//! Deterministic, renderer-independent inland lake sampling.
//!
//! Lakes are derived from connected closed-basin footprints, basin-floor
//! precipitation potential, and coarse-cell elevation. Only explicit changes
//! are persisted by the world model; generated geography remains seed-derived.

use super::rivers;
use super::{climate, coordinate_seed, depressions, terrain, watershed, MacroCoordinate};

/// Minimum connected coarse cells required for a depression to qualify as a lake.
pub const MIN_LAKE_BASIN_CELLS: u16 = 2;
const Q16_ONE: i64 = 1 << 16;

/// A coarse cell covered by a deterministically generated inland lake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LakeSample {
    outlet_cell: MacroCoordinate,
    water_surface_elevation_q16: i32,
    depth_q16: i32,
    basin_cell_count: u16,
    precipitation_potential_q16: u32,
    river_inflow: bool,
}

impl LakeSample {
    /// Stable outlet coordinate shared by every cell belonging to this lake.
    pub const fn outlet_cell(self) -> MacroCoordinate {
        self.outlet_cell
    }

    /// Deterministic lake identity, independent of traversal and region ownership.
    pub fn id(self, world_seed: u64, ruleset_version: u32) -> String {
        lake_id(world_seed, ruleset_version, self.outlet_cell)
    }

    /// Filled water-surface elevation in signed Q16.
    pub const fn water_surface_elevation_q16(self) -> i32 {
        self.water_surface_elevation_q16
    }

    /// Water depth above the local terrain sample, in signed Q16.
    pub const fn depth_q16(self) -> i32 {
        self.depth_q16
    }

    /// Number of connected below-spill coarse cells in the containing basin.
    pub const fn basin_cell_count(self) -> u16 {
        self.basin_cell_count
    }

    /// Precipitation potential at the basin floor, unsigned Q16.
    pub const fn precipitation_potential_q16(self) -> u32 {
        self.precipitation_potential_q16
    }

    /// Whether the basin's outlet receives a classified river channel.
    pub const fn has_river_inflow(self) -> bool {
        self.river_inflow
    }
}

/// Derive a stable lake ID from the world seed, ruleset, and basin outlet.
pub fn lake_id(world_seed: u64, ruleset_version: u32, outlet_cell: MacroCoordinate) -> String {
    let identity = coordinate_seed(world_seed, ruleset_version, outlet_cell);
    format!("lake.{}.{}.{identity:016x}", outlet_cell.x, outlet_cell.y)
}

/// Sample inland lake coverage at a global surface coordinate.
///
/// The query uses the existing bounded 17 × 17 depression footprint. A basin
/// qualifies when at least two connected coarse cells exist and the basin-floor
/// precipitation potential is nonzero. Precipitation scales the filled fraction
/// between the basin floor and its local spill saddle; this is a relative
/// first-pass water-balance proxy, not physical discharge or volume.
///
/// The returned sample exists only for cells at or below the derived water
/// surface. Ocean, coastal water, unresolved watershed traces, disconnected
/// depressions, and underground strata are never classified as inland lakes.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<LakeSample> {
    if coordinate.stratum != 0 {
        return None;
    }

    let coarse_cell =
        watershed::sample_surface(world_seed, ruleset_version, coordinate)?.coarse_cell();
    let local_terrain = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;
    if !local_terrain.is_land() {
        return None;
    }

    let depression = depressions::sample_surface(world_seed, ruleset_version, coarse_cell)?;
    if depression.basin_cell_count() < MIN_LAKE_BASIN_CELLS
        || !depression.contains_coarse_cell(coarse_cell)
    {
        return None;
    }

    let precipitation =
        climate::sample_surface(world_seed, ruleset_version, depression.outlet_cell())?
            .precipitation_q16();
    if precipitation == 0 {
        return None;
    }

    let filled_depth =
        i64::from(depression.depth_q16()).saturating_mul(i64::from(precipitation)) / Q16_ONE;
    let water_surface = i64::from(depression.floor_elevation_q16()) + filled_depth;
    let local_elevation = i64::from(local_terrain.elevation_q16());
    if local_elevation > water_surface {
        return None;
    }

    let depth = (water_surface - local_elevation).clamp(0, i64::from(i32::MAX)) as i32;
    let river_inflow =
        rivers::sample_surface(world_seed, ruleset_version, depression.outlet_cell())
            .is_some_and(|sample| sample.is_channel());

    Some(LakeSample {
        outlet_cell: depression.outlet_cell(),
        water_surface_elevation_q16: water_surface.clamp(i64::from(i32::MIN), i64::from(i32::MAX))
            as i32,
        depth_q16: depth,
        basin_cell_count: depression.basin_cell_count(),
        precipitation_potential_q16: precipitation,
        river_inflow,
    })
}
