//! Persistent, deterministic placement of world locations on macro-geography.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::content::CampaignContent;
use crate::geography::frontier::FrontierDirection;
use crate::geography::{
    coordinate_seed, hydrology, terrain, MacroCoordinate, RegionCoordinate,
    GEOGRAPHY_RULESET_VERSION,
};
use crate::model::{LocationGeographyRecord, World};

pub const LOCATION_GEOGRAPHY_EMBEDDING_VERSION: u32 = 2;
pub const DEFAULT_FOOTPRINT_RADIUS_SAMPLES: u16 = 4;
const PLACEMENT_CANDIDATES: u64 = 128;

/// Broad environmental preferences used when selecting a new location anchor.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocationArchetype {
    /// Favors flatter, accessible land with nearby classified freshwater.
    Settlement,
    /// Has no strong environmental preference beyond valid surface land.
    #[default]
    Wilderness,
    /// Favors elevated and geologically distinctive terrain.
    Landmark,
}

/// Attach deterministic geography to every region and location that lacks it.
///
/// Existing coordinates and anchors always win. No IDs, routes, history, or
/// saved local maps are changed by this migration.
pub fn ensure_location_geography(world: &mut World, content: Option<&CampaignContent>) {
    let seed = world
        .geography_seed
        .or_else(|| world.generation.as_ref().map(|generation| generation.seed))
        .unwrap_or_else(|| compatibility_seed(world));
    world.geography_seed = Some(seed);

    let ruleset = if world.geography_ruleset_version != 0 {
        world.geography_ruleset_version
    } else {
        world
            .generation
            .as_ref()
            .map(|generation| generation.geography_ruleset_version)
            .filter(|version| *version != 0)
            .unwrap_or(GEOGRAPHY_RULESET_VERSION)
    };
    world.geography_ruleset_version = ruleset;
    if let Some(generation) = world.generation.as_mut() {
        if generation.geography_ruleset_version == 0 {
            generation.geography_ruleset_version = ruleset;
        }
    }

    embed_missing_region_coordinates(world);

    let authored = content.map(|content| {
        content
            .world
            .locations
            .iter()
            .map(|location| (location.name.as_str(), location))
            .collect::<BTreeMap<_, _>>()
    });

    let mut region_ordinals = BTreeMap::<u64, usize>::new();
    for location in &world.locations {
        let Some(region) = world
            .regions
            .iter()
            .find(|region| region.id == location.region_id)
        else {
            continue;
        };
        let Some(region_coordinate) = region.coordinate else {
            continue;
        };

        let authored_location = authored
            .as_ref()
            .and_then(|entries| entries.get(location.name.as_str()).copied());
        let ordinal = region_ordinals.entry(location.region_id).or_default();
        let generated_key = format!(
            "location.generated.{}.{}.{}",
            region_coordinate.x, region_coordinate.y, *ordinal
        );
        *ordinal += 1;

        let stable_key = authored_location
            .map(|entry| format!("location.content.{}", entry.id))
            .unwrap_or_else(|| {
                if world.generation.is_some() {
                    generated_key
                } else {
                    format!("location.compat.{}", location.id)
                }
            });

        if let Some(existing) = world
            .location_geography
            .iter_mut()
            .find(|record| record.location_id == location.id)
        {
            if existing.stable_key.starts_with("location.compat.") && authored_location.is_some() {
                existing.stable_key = stable_key;
            }
            continue;
        }

        let authored_anchor = authored_location
            .and_then(|entry| entry.geography.as_ref())
            .and_then(|geography| geography.anchor)
            .filter(|anchor| {
                anchor.stratum == 0 && anchor.region_and_local().0 == region_coordinate
            });

        let radius = authored_location
            .and_then(|entry| entry.geography.as_ref())
            .and_then(|geography| geography.footprint_radius_samples)
            .unwrap_or(DEFAULT_FOOTPRINT_RADIUS_SAMPLES)
            .min(32);
        let archetype = authored_location
            .map(|entry| entry.archetype)
            .unwrap_or_default();

        let anchor = authored_anchor.unwrap_or_else(|| {
            choose_anchor(seed, ruleset, region_coordinate, &stable_key, radius, archetype)
        });
        let footprint_radius_samples = if authored_anchor.is_some() {
            radius
        } else {
            validated_footprint_radius(seed, ruleset, anchor, radius)
        };

        world.location_geography.push(LocationGeographyRecord {
            location_id: location.id,
            stable_key,
            anchor,
            footprint_radius_samples,
        });
    }

    world
        .location_geography
        .sort_by_key(|record| record.location_id);
    world.geography_embedding_version = world
        .geography_embedding_version
        .max(LOCATION_GEOGRAPHY_EMBEDDING_VERSION);
    crate::geography::routes::ensure_world_route_geography(world);
}

fn compatibility_seed(world: &World) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    fn feed(hash: &mut u64, value: u64) {
        for byte in value.to_le_bytes() {
            *hash ^= u64::from(byte);
            *hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }

    feed(&mut hash, world.id);

    let mut regions = world
        .regions
        .iter()
        .map(|region| region.id)
        .collect::<Vec<_>>();
    regions.sort_unstable();
    for id in regions {
        feed(&mut hash, id);
    }

    let mut locations = world
        .locations
        .iter()
        .map(|location| (location.id, location.region_id))
        .collect::<Vec<_>>();
    locations.sort_unstable();
    for (id, region_id) in locations {
        feed(&mut hash, id);
        feed(&mut hash, region_id);
    }

    let mut routes = world.routes.iter().collect::<Vec<_>>();
    routes.sort_by(|left, right| left.id.cmp(&right.id));
    for route in routes {
        for byte in route.id.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        feed(&mut hash, route.location_a_id);
        feed(&mut hash, route.location_b_id);
        let direction = match route.direction {
            crate::model::RouteDirection::Bidirectional => 0,
            crate::model::RouteDirection::AtoB => 1,
            crate::model::RouteDirection::BtoA => 2,
        };
        feed(&mut hash, direction);
    }

    hash
}

fn embed_missing_region_coordinates(world: &mut World) {
    let mut region_ids = world
        .regions
        .iter()
        .map(|region| region.id)
        .collect::<Vec<_>>();
    region_ids.sort_unstable();
    let valid_ids = region_ids.iter().copied().collect::<BTreeSet<_>>();

    let mut adjacency = BTreeMap::<u64, BTreeSet<u64>>::new();
    for id in &region_ids {
        adjacency.entry(*id).or_default();
    }

    for route in &world.routes {
        let Some(region_a) = world
            .location_by_id(route.location_a_id)
            .map(|location| location.region_id)
        else {
            continue;
        };
        let Some(region_b) = world
            .location_by_id(route.location_b_id)
            .map(|location| location.region_id)
        else {
            continue;
        };

        if region_a != region_b && valid_ids.contains(&region_a) && valid_ids.contains(&region_b) {
            adjacency.entry(region_a).or_default().insert(region_b);
            adjacency.entry(region_b).or_default().insert(region_a);
        }
    }

    let mut order = Vec::with_capacity(region_ids.len());
    let mut visited = BTreeSet::new();
    for root in &region_ids {
        if !visited.insert(*root) {
            continue;
        }

        let mut queue = VecDeque::from([*root]);
        while let Some(current) = queue.pop_front() {
            order.push(current);
            for neighbor in adjacency.get(&current).into_iter().flatten() {
                if visited.insert(*neighbor) {
                    queue.push_back(*neighbor);
                }
            }
        }
    }

    let mut occupied = world
        .regions
        .iter()
        .filter_map(|region| region.coordinate)
        .map(|coordinate| (coordinate.x, coordinate.y, coordinate.stratum))
        .collect::<BTreeSet<_>>();

    let mut next_x = 0_i64;
    for id in order {
        let Some(region) = world.regions.iter_mut().find(|region| region.id == id) else {
            continue;
        };
        if region.coordinate.is_some() {
            continue;
        }

        while occupied.contains(&(next_x, 0, 0)) {
            next_x = next_x.saturating_add(1);
        }
        region.coordinate = Some(RegionCoordinate::new(next_x, 0));
        occupied.insert((next_x, 0, 0));
        next_x = next_x.saturating_add(1);
    }
}

fn choose_anchor(
    seed: u64,
    ruleset: u32,
    region: RegionCoordinate,
    stable_key: &str,
    radius: u16,
) -> MacroCoordinate {
    let Some(origin) = region.global_sample(0, 0) else {
        return MacroCoordinate::ORIGIN;
    };

    let key_hash = stable_key
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
        });

    let mut best: Option<(i64, MacroCoordinate)> = None;
    for index in 0..PLACEMENT_CANDIDATES {
        let random = coordinate_seed(seed ^ key_hash, ruleset.wrapping_add(index as u32), origin);
        let Some(coordinate) =
            region.global_sample((random & 0xff) as u16, ((random >> 32) & 0xff) as u16)
        else {
            continue;
        };

        let Some(sample) = terrain::sample_surface(seed, ruleset, coordinate) else {
            continue;
        };
        let footprint = validated_footprint_radius(seed, ruleset, coordinate, radius);
        let land_bonus = if sample.is_land() { 1_i64 << 40 } else { 0 };
        let footprint_bonus = if footprint == radius && radius > 0 {
            1_i64 << 24
        } else {
            0
        };
        let score = land_bonus + footprint_bonus + i64::from(sample.elevation_q16())
            - i64::from(sample.mountain_ridge_strength_q16() / 4);

        match best {
            None => best = Some((score, coordinate)),
            Some((best_score, _)) if score > best_score => {
                best = Some((score, coordinate));
            }
            Some(_) => {}
        }
    }

    best.map(|(_, coordinate)| coordinate).unwrap_or(origin)
}

/// Produce deterministic land-anchor candidates close to the edge through
/// which a frontier location will be entered. Route planning later rejects
/// candidates with no continuous land connection to the source.
pub(super) fn frontier_anchor_candidates(
    seed: u64,
    ruleset: u32,
    region: RegionCoordinate,
    stable_key: &str,
    direction: FrontierDirection,
    limit: usize,
) -> Vec<MacroCoordinate> {
    const CANDIDATE_COUNT: u64 = 256;
    const EDGE_INSET: u16 = 8;
    const EDGE_SPAN: u16 = 64;

    let Some(origin) = region.global_sample(0, 0) else {
        return Vec::new();
    };
    let key_hash = stable_key
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
        });

    let mut candidates = Vec::<(i64, MacroCoordinate)>::new();
    for index in 0..CANDIDATE_COUNT {
        let random = coordinate_seed(seed ^ key_hash, ruleset.wrapping_add(index as u32), origin);
        let edge = EDGE_INSET + (random as u16 % EDGE_SPAN);
        let tangent = (random >> 32) as u16 & 0xff;
        let (local_x, local_y) = match direction {
            FrontierDirection::North => (tangent, 256 - edge),
            FrontierDirection::East => (edge, tangent),
            FrontierDirection::South => (tangent, edge),
            FrontierDirection::West => (256 - edge, tangent),
        };
        let Some(coordinate) = region.global_sample(local_x, local_y) else {
            continue;
        };
        let Some(sample) = terrain::sample_surface(seed, ruleset, coordinate) else {
            continue;
        };
        if !sample.is_land() {
            continue;
        }

        let footprint =
            validated_footprint_radius(seed, ruleset, coordinate, DEFAULT_FOOTPRINT_RADIUS_SAMPLES);
        let score = (1_i64 << 40) + i64::from(sample.elevation_q16())
            - i64::from(sample.mountain_ridge_strength_q16() / 4)
            + if footprint == DEFAULT_FOOTPRINT_RADIUS_SAMPLES {
                1_i64 << 20
            } else {
                0
            };
        candidates.push((score, coordinate));
    }

    candidates.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.x.cmp(&right.1.x))
            .then_with(|| left.1.y.cmp(&right.1.y))
    });
    candidates.dedup_by_key(|(_, coordinate)| *coordinate);
    candidates
        .into_iter()
        .take(limit)
        .map(|(_, coordinate)| coordinate)
        .collect()
}

pub(super) fn validated_footprint_radius(
    seed: u64,
    ruleset: u32,
    anchor: MacroCoordinate,
    requested: u16,
) -> u16 {
    if requested == 0 {
        return 0;
    }

    let radius = i64::from(requested);
    let offsets = [
        (radius, 0),
        (-radius, 0),
        (0, radius),
        (0, -radius),
        (radius, radius),
        (radius, -radius),
        (-radius, radius),
        (-radius, -radius),
    ];

    if offsets.iter().all(|(dx, dy)| {
        anchor
            .x
            .checked_add(*dx)
            .zip(anchor.y.checked_add(*dy))
            .and_then(|(x, y)| terrain::sample_surface(seed, ruleset, MacroCoordinate::new(x, y)))
            .is_some_and(|sample| sample.is_land())
    }) {
        requested
    } else {
        0
    }
}

#[cfg(test)]
#[path = "../../tests/unit/location_geography.rs"]
mod tests;
