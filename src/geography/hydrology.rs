//! Renderer-independent surface-water classification and local flow connections.
//!
//! This module composes the existing terrain, watershed, river, and lake samplers.
//! It does not simulate water movement or claim physically calibrated discharge.

use super::{rivers, terrain, waterbodies, watershed, MacroCoordinate};

/// The surface-water feature occupying a sampled coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceWaterKind {
    /// Ocean terrain, including coastal water.
    Ocean,
    /// A cell covered by a generated inland lake.
    Lake(waterbodies::LakeSample),
    /// A cell classified as part of a river channel.
    River(rivers::RiverSample),
}

/// The immediate destination of a river channel's drainage link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowDestination {
    /// The next coarse watershed cell is ocean.
    Ocean,
    /// The next coarse watershed cell is covered by this inland lake.
    Lake { outlet_cell: MacroCoordinate },
    /// The next coarse watershed cell is also classified as a river channel.
    River { coarse_cell: MacroCoordinate },
    /// The drainage link continues over land below the current channel threshold.
    Land { coarse_cell: MacroCoordinate },
    /// The channel cell is a closed watershed sink with no downstream link.
    ClosedBasin,
}

/// A derived surface-water sample and, for rivers, its immediate drainage link.
///
/// River links follow the existing strictly downhill coarse watershed graph.
/// They are local graph connections, not a complete discharge simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceWaterSample {
    coordinate: MacroCoordinate,
    kind: SurfaceWaterKind,
    downstream_cell: Option<MacroCoordinate>,
    downstream: Option<FlowDestination>,
}

impl SurfaceWaterSample {
    /// The queried global coordinate.
    pub const fn coordinate(self) -> MacroCoordinate {
        self.coordinate
    }

    /// The surface-water feature at the queried coordinate.
    pub const fn kind(self) -> SurfaceWaterKind {
        self.kind
    }

    /// The next coarse drainage cell for a river, if one exists.
    ///
    /// This is None for ocean and lake samples, and for closed-basin river sinks.
    pub const fn downstream_cell(self) -> Option<MacroCoordinate> {
        self.downstream_cell
    }

    /// The classified immediate destination of a river's downstream link.
    ///
    /// Ocean and lake samples have no outgoing drainage destination. A closed
    /// basin river sink reports Some(ClosedBasin) to distinguish it from those
    /// terminal water features.
    pub const fn downstream(self) -> Option<FlowDestination> {
        self.downstream
    }
}

/// Sample surface water and the immediate destination of any river channel.
///
/// Ocean and lake coverage take precedence over river-channel classification:
/// a river terminating inside a lake is represented as a lake cell, while
/// upstream river cells resolve their next link to that lake's stable outlet.
/// Land without a classified river channel returns None. Underground strata
/// are never interpreted as surface water.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<SurfaceWaterSample> {
    if coordinate.stratum != 0 {
        return None;
    }

    let local_terrain = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;
    if !local_terrain.is_land() {
        return Some(SurfaceWaterSample {
            coordinate,
            kind: SurfaceWaterKind::Ocean,
            downstream_cell: None,
            downstream: None,
        });
    }

    if let Some(lake) = waterbodies::sample_surface(world_seed, ruleset_version, coordinate) {
        return Some(SurfaceWaterSample {
            coordinate,
            kind: SurfaceWaterKind::Lake(lake),
            downstream_cell: None,
            downstream: None,
        });
    }

    let river = rivers::sample_surface(world_seed, ruleset_version, coordinate)?;
    if !river.is_channel() {
        return None;
    }

    let watershed = watershed::sample_surface(world_seed, ruleset_version, coordinate)?;
    let downstream_cell = watershed.coarse_downstream();
    let downstream = match downstream_cell {
        Some(next) => Some(classify_destination(world_seed, ruleset_version, next)?),
        None => Some(FlowDestination::ClosedBasin),
    };

    Some(SurfaceWaterSample {
        coordinate,
        kind: SurfaceWaterKind::River(river),
        downstream_cell,
        downstream,
    })
}

fn classify_destination(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<FlowDestination> {
    let elevation = terrain::sample_surface(world_seed, ruleset_version, coordinate)?;
    if !elevation.is_land() {
        return Some(FlowDestination::Ocean);
    }

    if let Some(lake) = waterbodies::sample_surface(world_seed, ruleset_version, coordinate) {
        return Some(FlowDestination::Lake {
            outlet_cell: lake.outlet_cell(),
        });
    }

    let river = rivers::sample_surface(world_seed, ruleset_version, coordinate)?;
    if river.is_channel() {
        Some(FlowDestination::River {
            coarse_cell: river.coarse_cell(),
        })
    } else {
        Some(FlowDestination::Land {
            coarse_cell: river.coarse_cell(),
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/hydrology.rs"]
mod tests;
