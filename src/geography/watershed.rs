//! Deterministic, renderer-independent hierarchical watershed sampling.
//!
//! A coarse drainage lattice establishes continental-scale downstream links.
//! Local D8 drainage is sampled within each coarse cell, while an explicit
//! outlet trace follows the coarser network when callers need basin identity.
//! All decisions derive from the global elevation field and signed coordinates.

use super::{terrain, MacroCoordinate};

/// Width of one hierarchical drainage cell in global macro samples.
pub const WATERSHED_CELL_SAMPLES: i64 = 4_096;

/// Maximum number of coarse links followed by one outlet trace.
pub const MAX_OUTLET_TRACE_STEPS: usize = 128;

const NEIGHBORS: [(i64, i64, DrainageDirection); 8] = [
    (0, -1, DrainageDirection::North),
    (1, -1, DrainageDirection::NorthEast),
    (1, 0, DrainageDirection::East),
    (1, 1, DrainageDirection::SouthEast),
    (0, 1, DrainageDirection::South),
    (-1, 1, DrainageDirection::SouthWest),
    (-1, 0, DrainageDirection::West),
    (-1, -1, DrainageDirection::NorthWest),
];

/// One of the eight neighboring drainage directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrainageDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl DrainageDirection {
    /// Return the signed global-coordinate offset for this direction.
    pub const fn offset(self) -> (i64, i64) {
        match self {
            Self::North => (0, -1),
            Self::NorthEast => (1, -1),
            Self::East => (1, 0),
            Self::SouthEast => (1, 1),
            Self::South => (0, 1),
            Self::SouthWest => (-1, 1),
            Self::West => (-1, 0),
            Self::NorthWest => (-1, -1),
        }
    }
}

/// The coarse drainage state at a surface coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatershedSample {
    coarse_cell: MacroCoordinate,
    elevation_q16: i32,
    coarse_downstream: Option<MacroCoordinate>,
    local_direction: Option<DrainageDirection>,
}

impl WatershedSample {
    /// Global coordinate at the center of the containing coarse drainage cell.
    pub const fn coarse_cell(self) -> MacroCoordinate {
        self.coarse_cell
    }

    /// Elevation at the coarse cell's representative center, in signed Q16.
    pub const fn elevation_q16(self) -> i32 {
        self.elevation_q16
    }

    /// The next lower coarse cell, or None for ocean cells and local sinks.
    pub const fn coarse_downstream(self) -> Option<MacroCoordinate> {
        self.coarse_downstream
    }

    /// The steepest lower immediate neighbor inside this coarse cell.
    ///
    /// None means no lower same-cell neighbor exists. The coarse downstream
    /// link remains available separately, so local minima do not erase the
    /// larger drainage network.
    pub const fn local_direction(self) -> Option<DrainageDirection> {
        self.local_direction
    }

    pub const fn is_ocean(self) -> bool {
        self.elevation_q16 <= 0
    }

    pub const fn is_local_sink(self) -> bool {
        self.local_direction.is_none()
    }
}

/// The kind of terminal node reached by tracing the coarse drainage graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutletKind {
    /// The drainage path reached a coarse cell at or below sea level.
    Ocean,
    /// The path reached a land cell with no lower neighboring coarse cell.
    ClosedBasin,
    /// The bounded trace ended before a terminal node was found.
    SearchLimit,
}

/// A deterministic coarse-network outlet or bounded continuation point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatershedOutlet {
    coordinate: MacroCoordinate,
    kind: OutletKind,
    steps: u16,
}

impl WatershedOutlet {
    /// Global coordinate at the center of the terminal/continuation cell.
    pub const fn coordinate(self) -> MacroCoordinate {
        self.coordinate
    }

    pub const fn kind(self) -> OutletKind {
        self.kind
    }

    /// Number of coarse links traversed from the starting cell.
    pub const fn steps(self) -> u16 {
        self.steps
    }

    pub const fn is_terminal(self) -> bool {
        !matches!(self.kind, OutletKind::SearchLimit)
    }
}

/// Sample the hierarchical drainage field at a global surface coordinate.
///
/// The coarse network is evaluated on a 4,096-sample lattice. The local D8
/// direction is restricted to neighbors inside the same coarse cell; the
/// separate coarse link carries flow across watershed-cell boundaries.
/// Returns None for underground strata or coordinates whose cell center
/// cannot be represented without signed-coordinate overflow.
pub fn sample_surface(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<WatershedSample> {
    if coordinate.stratum != 0 {
        return None;
    }

    let coarse_x = coordinate.x.div_euclid(WATERSHED_CELL_SAMPLES);
    let coarse_y = coordinate.y.div_euclid(WATERSHED_CELL_SAMPLES);
    let coarse_cell = cell_center(coarse_x, coarse_y)?;
    let elevation = terrain::sample_surface(world_seed, ruleset_version, coarse_cell)?;
    let coarse_downstream = if elevation.is_land() {
        downstream_cell(world_seed, ruleset_version, coarse_x, coarse_y, elevation.elevation_q16())
    } else {
        None
    };

    let mut local_direction = None;
    let mut lowest_elevation = terrain::sample_surface(world_seed, ruleset_version, coordinate)?
        .elevation_q16();

    for (dx, dy, direction) in NEIGHBORS {
        let Some(x) = coordinate.x.checked_add(dx) else {
            continue;
        };
        let Some(y) = coordinate.y.checked_add(dy) else {
            continue;
        };
        if x.div_euclid(WATERSHED_CELL_SAMPLES) != coarse_x
            || y.div_euclid(WATERSHED_CELL_SAMPLES) != coarse_y
        {
            continue;
        }

        let neighbor = MacroCoordinate::new(x, y);
        let Some(sample) = terrain::sample_surface(world_seed, ruleset_version, neighbor) else {
            continue;
        };
        if sample.elevation_q16() < lowest_elevation {
            lowest_elevation = sample.elevation_q16();
            local_direction = Some(direction);
        }
    }

    Some(WatershedSample {
        coarse_cell,
        elevation_q16: elevation.elevation_q16(),
        coarse_downstream,
        local_direction,
    })
}

/// Follow the coarse drainage network to an ocean or closed-basin outlet.
///
/// Every link strictly decreases representative elevation, so the graph is
/// acyclic. The trace is deliberately bounded to keep an isolated query's
/// worst-case work predictable. SearchLimit is an explicit continuation
/// result, not a fabricated basin identity.
pub fn trace_to_outlet(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
) -> Option<WatershedOutlet> {
    if coordinate.stratum != 0 {
        return None;
    }

    let mut coarse_x = coordinate.x.div_euclid(WATERSHED_CELL_SAMPLES);
    let mut coarse_y = coordinate.y.div_euclid(WATERSHED_CELL_SAMPLES);

    for steps in 0..=MAX_OUTLET_TRACE_STEPS {
        let cell = cell_center(coarse_x, coarse_y)?;
        let elevation = terrain::sample_surface(world_seed, ruleset_version, cell)?;
        if !elevation.is_land() {
            return Some(WatershedOutlet {
                coordinate: cell,
                kind: OutletKind::Ocean,
                steps: steps as u16,
            });
        }

        let Some(next) = downstream_cell(
            world_seed,
            ruleset_version,
            coarse_x,
            coarse_y,
            elevation.elevation_q16(),
        ) else {
            return Some(WatershedOutlet {
                coordinate: cell,
                kind: OutletKind::ClosedBasin,
                steps: steps as u16,
            });
        };

        if steps == MAX_OUTLET_TRACE_STEPS {
            return Some(WatershedOutlet {
                coordinate: cell,
                kind: OutletKind::SearchLimit,
                steps: steps as u16,
            });
        }

        coarse_x = next.x.div_euclid(WATERSHED_CELL_SAMPLES);
        coarse_y = next.y.div_euclid(WATERSHED_CELL_SAMPLES);
    }

    None
}

fn cell_center(coarse_x: i64, coarse_y: i64) -> Option<MacroCoordinate> {
    let half_cell = WATERSHED_CELL_SAMPLES / 2;
    Some(MacroCoordinate::new(
        coarse_x
            .checked_mul(WATERSHED_CELL_SAMPLES)?
            .checked_add(half_cell)?,
        coarse_y
            .checked_mul(WATERSHED_CELL_SAMPLES)?
            .checked_add(half_cell)?,
    ))
}

fn downstream_cell(
    world_seed: u64,
    ruleset_version: u32,
    coarse_x: i64,
    coarse_y: i64,
    current_elevation: i32,
) -> Option<MacroCoordinate> {
    let mut lowest: Option<(i32, i64, i64, MacroCoordinate)> = None;

    for (dx, dy, _) in NEIGHBORS {
        let Some(x) = coarse_x.checked_add(dx) else {
            continue;
        };
        let Some(y) = coarse_y.checked_add(dy) else {
            continue;
        };
        let Some(coordinate) = cell_center(x, y) else {
            continue;
        };
        let Some(sample) = terrain::sample_surface(world_seed, ruleset_version, coordinate) else {
            continue;
        };
        let candidate = sample.elevation_q16();
        if candidate >= current_elevation {
            continue;
        }

        let candidate_key = (candidate, y, x, coordinate);
        if lowest
            .as_ref()
            .is_none_or(|(best, best_y, best_x, _)| {
                (candidate, y, x) < (*best, *best_y, *best_x)
            })
        {
            lowest = Some(candidate_key);
        }
    }

    lowest.map(|(_, _, _, coordinate)| coordinate)
}
