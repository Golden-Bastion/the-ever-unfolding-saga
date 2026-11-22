//! Persistent, on-demand topology for individually addressable cave chambers.
//!
//! Cave chamber identities come from the deterministic underground network
//! lattice. This module materializes only immediate horizontal neighbors and
//! nearby valid vertical links; it never scans an unbounded stratum.

use serde::{Deserialize, Serialize};

use super::{coordinate_seed, underground, vertical, MacroCoordinate};
use crate::model::{Location, LocationGeographyRecord, Region, RouteDirection, World, WorldRoute};

/// Physical relationship represented by one cave route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaveConnectionKind {
    HorizontalPassage,
    SurfaceEntrance,
    VerticalShaft,
}

/// A pending, deterministic connection from one location to a cave chamber.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaveConnectionTarget {
    /// Stable identity independent of world entity allocation order.
    pub id: String,
    pub destination_anchor: MacroCoordinate,
    /// Existing endpoint when a reverse vertical link leads to an already materialized location.
    #[serde(default)]
    pub destination_location_id: Option<u64>,
    pub kind: CaveConnectionKind,
}

impl CaveConnectionTarget {
    pub fn route_id(&self) -> String {
        format!("route.cave.{}", self.id)
    }

    pub fn location_key(&self) -> String {
        if self.destination_anchor.stratum == 0 {
            format!(
                "location.cave_entrance.{}.{}",
                self.destination_anchor.x, self.destination_anchor.y
            )
        } else {
            cave_location_key(self.destination_anchor)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveMaterialization {
    pub region_id: u64,
    pub location_id: u64,
    pub route_id: String,
    pub created_region: bool,
    pub created_location: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveMaterializationError {
    MissingGeography,
    MissingSource,
    InvalidConnection,
    IdentityConflict,
}

fn cave_location_key(anchor: MacroCoordinate) -> String {
    format!("location.cave.{}.{}.{}", anchor.stratum, anchor.x, anchor.y)
}

fn neighboring_chamber_center(
    seed: u64,
    ruleset: u32,
    cell_x: i64,
    cell_y: i64,
    stratum: i32,
) -> Option<MacroCoordinate> {
    const RADIUS: i64 = 12;
    const STEP: i64 = 3;
    let base_x = cell_x
        .checked_mul(underground::NETWORK_CELL_SAMPLES)?
        .checked_add(underground::NETWORK_CELL_SAMPLES / 2)?;
    let base_y = cell_y
        .checked_mul(underground::NETWORK_CELL_SAMPLES)?
        .checked_add(underground::NETWORK_CELL_SAMPLES / 2)?;
    let mut offset_y = -RADIUS;
    while offset_y <= RADIUS {
        let mut offset_x = -RADIUS;
        while offset_x <= RADIUS {
            let coordinate = MacroCoordinate::in_stratum(
                base_x.checked_add(offset_x)?,
                base_y.checked_add(offset_y)?,
                stratum,
            );
            if let Some(center) = underground::sample_underground(seed, ruleset, coordinate)
                .and_then(|sample| sample.network_cell())
            {
                if center.x.div_euclid(underground::NETWORK_CELL_SAMPLES) == cell_x
                    && center.y.div_euclid(underground::NETWORK_CELL_SAMPLES) == cell_y
                    && center.stratum == stratum
                {
                    return Some(center);
                }
            }
            offset_x += STEP;
        }
        offset_y += STEP;
    }
    None
}

fn horizontal_target_id(first: MacroCoordinate, second: MacroCoordinate) -> String {
    let (first, second) =
        if (first.stratum, first.x, first.y) <= (second.stratum, second.x, second.y) {
            (first, second)
        } else {
            (second, first)
        };
    format!(
        "horizontal.{}.{}.{}.{}.{}",
        first.stratum, first.x, first.y, second.x, second.y
    )
}

/// Return the immediate cave connections available from one persisted location.
/// Surface locations expose nearby shafts; cave locations expose their actual
/// reciprocal horizontal passages and shafts attached to that exact chamber.
pub fn connection_targets_for_location(
    world: &World,
    source_location_id: u64,
) -> Vec<CaveConnectionTarget> {
    let Some(seed) = world.geography_seed else {
        return Vec::new();
    };
    let Some(anchor) = world
        .location_geography
        .iter()
        .find(|record| record.location_id == source_location_id)
        .map(|record| record.anchor)
    else {
        return Vec::new();
    };
    if world.location_by_id(source_location_id).is_none() || anchor.stratum > 0 {
        return Vec::new();
    }

    let ruleset = world.geography_ruleset_version;
    let mut targets = Vec::new();

    if anchor.stratum == 0 {
        for connection in vertical::connections_near(seed, ruleset, anchor, 2) {
            if connection.kind != vertical::VerticalConnectionKind::SurfaceEntrance {
                continue;
            }
            let Some(chamber) = underground::sample_underground(seed, ruleset, connection.lower)
                .and_then(|sample| sample.network_cell())
            else {
                continue;
            };
            targets.push(CaveConnectionTarget {
                id: format!("vertical.{:016x}", connection.id),
                destination_anchor: chamber,
                destination_location_id: None,
                kind: CaveConnectionKind::SurfaceEntrance,
            });
        }
    } else {
        let Some(sample) = underground::sample_underground(seed, ruleset, anchor) else {
            return Vec::new();
        };
        let Some(chamber) = sample.network_cell() else {
            return Vec::new();
        };
        if chamber != anchor {
            return Vec::new();
        }

        let directions = [
            (underground::direction::NORTH, 0_i64, -1_i64),
            (underground::direction::EAST, 1, 0),
            (underground::direction::SOUTH, 0, 1),
            (underground::direction::WEST, -1, 0),
        ];
        for (bit, dx, dy) in directions {
            if sample.connections() & bit == 0 {
                continue;
            }
            let Some(cell_x) = chamber
                .x
                .div_euclid(underground::NETWORK_CELL_SAMPLES)
                .checked_add(dx)
            else {
                continue;
            };
            let Some(cell_y) = chamber
                .y
                .div_euclid(underground::NETWORK_CELL_SAMPLES)
                .checked_add(dy)
            else {
                continue;
            };
            let Some(neighbor) =
                neighboring_chamber_center(seed, ruleset, cell_x, cell_y, chamber.stratum)
            else {
                continue;
            };
            targets.push(CaveConnectionTarget {
                id: horizontal_target_id(chamber, neighbor),
                destination_anchor: neighbor,
                destination_location_id: None,
                kind: CaveConnectionKind::HorizontalPassage,
            });
        }

        for connection in vertical::connections_near(seed, ruleset, chamber, 2) {
            let (source_endpoint, destination_endpoint) =
                if connection.upper.stratum == chamber.stratum {
                    (connection.upper, connection.lower)
                } else if connection.lower.stratum == chamber.stratum {
                    (connection.lower, connection.upper)
                } else {
                    continue;
                };
            let Some(source_chamber) =
                underground::sample_underground(seed, ruleset, source_endpoint)
                    .and_then(|value| value.network_cell())
            else {
                continue;
            };
            if source_chamber != chamber {
                continue;
            }
            let (destination, destination_location_id, kind) = if destination_endpoint.stratum == 0
            {
                let route_id = format!("route.cave.vertical.{:016x}", connection.id);
                let existing = world
                    .route_by_id(&route_id)
                    .and_then(|route| route.destination_from(source_location_id));
                if let Some(destination_id) = existing {
                    let Some(surface_anchor) = world
                        .location_geography
                        .iter()
                        .find(|record| record.location_id == destination_id)
                        .map(|record| record.anchor)
                    else {
                        continue;
                    };
                    (
                        surface_anchor,
                        Some(destination_id),
                        CaveConnectionKind::SurfaceEntrance,
                    )
                } else {
                    (
                        destination_endpoint,
                        None,
                        CaveConnectionKind::SurfaceEntrance,
                    )
                }
            } else {
                let Some(destination) =
                    underground::sample_underground(seed, ruleset, destination_endpoint)
                        .and_then(|value| value.network_cell())
                else {
                    continue;
                };
                (destination, None, CaveConnectionKind::VerticalShaft)
            };
            targets.push(CaveConnectionTarget {
                id: format!("vertical.{:016x}", connection.id),
                destination_anchor: destination,
                destination_location_id,
                kind,
            });
        }
    }

    if anchor.stratum == 0 {
        targets.sort_by_key(|target| {
            let dx =
                (i128::from(target.destination_anchor.x) - i128::from(anchor.x)).unsigned_abs();
            let dy =
                (i128::from(target.destination_anchor.y) - i128::from(anchor.y)).unsigned_abs();
            (
                dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)),
                target.id.clone(),
            )
        });
        targets.dedup_by(|left, right| left.id == right.id);
        targets.truncate(3);
    } else {
        targets.sort_by(|left, right| left.id.cmp(&right.id));
        targets.dedup_by(|left, right| left.id == right.id);
    }
    targets
}

/// Materialize one validated cave connection. The destination chamber and its
/// region are created at most once; the authoritative route is bidirectional.
pub fn materialize_connection(
    world: &mut World,
    source_location_id: u64,
    target: &CaveConnectionTarget,
) -> Result<CaveMaterialization, CaveMaterializationError> {
    if !connection_targets_for_location(world, source_location_id).contains(target) {
        return Err(CaveMaterializationError::InvalidConnection);
    }
    if world.location_by_id(source_location_id).is_none() {
        return Err(CaveMaterializationError::MissingSource);
    }
    let seed = world
        .geography_seed
        .ok_or(CaveMaterializationError::MissingGeography)?;
    let route_id = target.route_id();
    let stable_key = target.location_key();

    let existing_location_id = target
        .destination_location_id
        .or_else(|| {
            world
                .location_geography
                .iter()
                .find(|record| record.stable_key == stable_key)
                .map(|record| record.location_id)
        })
        .or_else(|| {
            if target.destination_anchor.stratum == 0 {
                world
                    .location_geography
                    .iter()
                    .find(|record| record.anchor == target.destination_anchor)
                    .map(|record| record.location_id)
            } else {
                None
            }
        });

    if let Some(route) = world.route_by_id(&route_id) {
        let Some(location_id) = existing_location_id else {
            return Err(CaveMaterializationError::IdentityConflict);
        };
        if !route.connects(source_location_id, location_id)
            || !route.allows_travel(source_location_id, location_id)
        {
            return Err(CaveMaterializationError::IdentityConflict);
        }
        let region_id = world
            .location_by_id(location_id)
            .ok_or(CaveMaterializationError::IdentityConflict)?
            .region_id;
        return Ok(CaveMaterialization {
            region_id,
            location_id,
            route_id,
            created_region: false,
            created_location: false,
        });
    }
    if world.routes.iter().any(|route| route.id == route_id) {
        return Err(CaveMaterializationError::IdentityConflict);
    }

    let destination_region = target.destination_anchor.region_and_local().0;
    let existing_region_id = world
        .regions
        .iter()
        .find(|region| region.coordinate == Some(destination_region))
        .map(|region| region.id);
    if let Some(location_id) = existing_location_id {
        let Some(location) = world.location_by_id(location_id) else {
            return Err(CaveMaterializationError::IdentityConflict);
        };
        if world
            .regions
            .iter()
            .find(|region| region.id == location.region_id)
            .and_then(|region| region.coordinate)
            != Some(destination_region)
        {
            return Err(CaveMaterializationError::IdentityConflict);
        }
    }

    let next_id_before = world.next_id;
    let created_region = existing_region_id.is_none();
    let region_id = existing_region_id.unwrap_or_else(|| {
        let id = world.allocate_id();
        world.regions.push(Region {
            id,
            name: format!(
                "Cave Region ({}, {}, {})",
                destination_region.x, destination_region.y, destination_region.stratum
            ),
            description: "A region of the underground cave network.".to_string(),
            location_ids: Vec::new(),
            coordinate: Some(destination_region),
        });
        id
    });

    let created_location = existing_location_id.is_none();
    let location_id = existing_location_id.unwrap_or_else(|| {
        let id = world.allocate_id();
        let anchor = target.destination_anchor;
        world.locations.push(Location {
            id,
            name: if anchor.stratum == 0 {
                format!("Hidden Cave Entrance ({}, {})", anchor.x, anchor.y)
            } else {
                format!(
                    "Cavern Chamber ({}, {}, layer {})",
                    anchor.x, anchor.y, anchor.stratum
                )
            },
            description: if anchor.stratum == 0 {
                "A concealed opening in the earth leads into the cave network.".to_string()
            } else {
                format!(
                    "A natural chamber deep beneath the surface (stratum {}).",
                    anchor.stratum
                )
            },
            region_id,
            dangerous: anchor.stratum != 0
                && (coordinate_seed(seed, world.geography_ruleset_version, anchor) & 3) != 0,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
            spatial_layout: None,
        });
        world
            .regions
            .iter_mut()
            .find(|region| region.id == region_id)
            .expect("cave region was just resolved")
            .location_ids
            .push(id);
        world.location_geography.push(LocationGeographyRecord {
            location_id: id,
            stable_key: stable_key.clone(),
            anchor,
            footprint_radius_samples: 0,
        });
        world
            .location_geography
            .sort_by_key(|record| record.location_id);
        id
    });

    if !world.add_route(WorldRoute::new(
        route_id.clone(),
        source_location_id,
        location_id,
        RouteDirection::Bidirectional,
    )) {
        if created_location {
            world
                .locations
                .retain(|location| location.id != location_id);
            world
                .location_geography
                .retain(|record| record.location_id != location_id);
            if let Some(region) = world
                .regions
                .iter_mut()
                .find(|region| region.id == region_id)
            {
                region.location_ids.retain(|id| *id != location_id);
            }
        }
        if created_region {
            world.regions.retain(|region| region.id != region_id);
        }
        world.next_id = next_id_before;
        return Err(CaveMaterializationError::IdentityConflict);
    }

    Ok(CaveMaterialization {
        region_id,
        location_id,
        route_id,
        created_region,
        created_location,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/cave_topology.rs"]
mod tests;
