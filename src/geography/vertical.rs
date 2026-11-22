//! Deterministic vertical links between adjacent macro-geography strata.
//!
//! A vertical connection is derived only where the same global horizontal
//! coordinate is physically open on both connected layers. Surface entrances
//! additionally require exposed land. The sampler is bounded to one cave
//! network cell and never scans or mutates world state.

use super::{coordinate_seed, terrain, underground, MacroCoordinate};

/// Maximum half-width of the deterministic candidate grid around a cave cell.
const CANDIDATE_RADIUS: i64 = 12;
/// Spacing between candidate coordinates.
const CANDIDATE_STEP: i64 = 3;

/// The physical kind of a vertical cave connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalConnectionKind {
    /// An exposed surface opening leading into the first underground stratum.
    SurfaceEntrance,
    /// A shaft or natural drop connecting two adjacent underground strata.
    InterStratum,
}

/// A deterministic vertical opening at one shared horizontal coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalConnection {
    /// Stable identity derived from the connected strata and global position.
    pub id: u64,
    /// Upper endpoint, on the surface or a shallower underground stratum.
    pub upper: MacroCoordinate,
    /// Lower endpoint, exactly one stratum below `upper`.
    pub lower: MacroCoordinate,
    pub kind: VerticalConnectionKind,
}

/// Find the deterministic vertical opening associated with one underground
/// network cell and its immediately upper layer.
///
/// `upper_stratum` is zero for a surface entrance, or a negative underground
/// stratum for an inter-stratum connection. Positive strata are unsupported.
/// At most one connection is returned per cell/layer pair.
pub fn connection_for_cell(
    world_seed: u64,
    ruleset_version: u32,
    cell_x: i64,
    cell_y: i64,
    upper_stratum: i32,
) -> Option<VerticalConnection> {
    if upper_stratum > 0 || upper_stratum == i32::MIN {
        return None;
    }
    let lower_stratum = upper_stratum.checked_sub(1)?;
    let base_x = cell_x
        .checked_mul(underground::NETWORK_CELL_SAMPLES)?
        .checked_add(underground::NETWORK_CELL_SAMPLES / 2)?;
    let base_y = cell_y
        .checked_mul(underground::NETWORK_CELL_SAMPLES)?
        .checked_add(underground::NETWORK_CELL_SAMPLES / 2)?;
    let kind = if upper_stratum == 0 {
        VerticalConnectionKind::SurfaceEntrance
    } else {
        VerticalConnectionKind::InterStratum
    };

    let mut best: Option<(u64, MacroCoordinate)> = None;
    let mut offset_y = -CANDIDATE_RADIUS;
    while offset_y <= CANDIDATE_RADIUS {
        let mut offset_x = -CANDIDATE_RADIUS;
        while offset_x <= CANDIDATE_RADIUS {
            let Some(x) = base_x.checked_add(offset_x) else {
                offset_x += CANDIDATE_STEP;
                continue;
            };
            let Some(y) = base_y.checked_add(offset_y) else {
                offset_x += CANDIDATE_STEP;
                continue;
            };
            let upper = MacroCoordinate::in_stratum(x, y, upper_stratum);
            let lower = MacroCoordinate::in_stratum(x, y, lower_stratum);
            let lower_open = underground::sample_underground(world_seed, ruleset_version, lower)
                .is_some_and(|sample| sample.is_open());
            let upper_open = if upper_stratum == 0 {
                terrain::sample_surface(world_seed, ruleset_version, upper)
                    .is_some_and(|sample| sample.is_land())
            } else {
                underground::sample_underground(world_seed, ruleset_version, upper)
                    .is_some_and(|sample| sample.is_open())
            };

            if upper_open && lower_open {
                let score = coordinate_seed(
                    world_seed,
                    ruleset_version ^ 0x5645_5254,
                    MacroCoordinate::in_stratum(x, y, upper_stratum),
                );
                if best.map_or(true, |(best_score, _)| score < best_score) {
                    best = Some((score, lower));
                }
            }
            offset_x += CANDIDATE_STEP;
        }
        offset_y += CANDIDATE_STEP;
    }

    let (_, lower) = best?;
    let upper = MacroCoordinate::in_stratum(lower.x, lower.y, upper_stratum);
    let id = coordinate_seed(world_seed, ruleset_version ^ 0x5348_4146, upper);
    Some(VerticalConnection {
        id,
        upper,
        lower,
        kind,
    })
}

/// Find vertical connections adjacent to the queried layer in a bounded
/// square of cave network cells. Underground queries include both the link
/// descending from the current layer and the link ascending to the layer above.
/// The radius is clamped to two cells (25 cells per layer pair).
pub fn connections_near(
    world_seed: u64,
    ruleset_version: u32,
    coordinate: MacroCoordinate,
    radius_cells: u8,
) -> Vec<VerticalConnection> {
    if coordinate.stratum > 0 || coordinate.stratum == i32::MIN {
        return Vec::new();
    }
    let radius = i64::from(radius_cells.min(2));
    let cell_x = coordinate.x.div_euclid(underground::NETWORK_CELL_SAMPLES);
    let cell_y = coordinate.y.div_euclid(underground::NETWORK_CELL_SAMPLES);
    let upper_strata = if coordinate.stratum == 0 {
        vec![0]
    } else {
        vec![coordinate.stratum + 1, coordinate.stratum]
    };
    let mut connections = Vec::new();
    for upper_stratum in upper_strata {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let (Some(x), Some(y)) = (cell_x.checked_add(dx), cell_y.checked_add(dy)) else {
                    continue;
                };
                if let Some(connection) =
                    connection_for_cell(world_seed, ruleset_version, x, y, upper_stratum)
                {
                    connections.push(connection);
                }
            }
        }
    }
    connections.sort_by_key(|connection| {
        (
            connection.upper.stratum,
            connection.upper.x,
            connection.upper.y,
        )
    });
    connections
}

#[cfg(test)]
#[path = "../../tests/unit/vertical_connections.rs"]
mod tests;
