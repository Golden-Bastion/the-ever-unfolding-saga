//! Persistent, deterministic placement of world locations on macro-geography.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use crate::content::CampaignContent;
use crate::geography::{coordinate_seed, terrain, MacroCoordinate, RegionCoordinate, GEOGRAPHY_RULESET_VERSION};
use crate::model::{LocationGeographyRecord, World};

pub const LOCATION_GEOGRAPHY_EMBEDDING_VERSION: u32 = 1;
pub const DEFAULT_FOOTPRINT_RADIUS_SAMPLES: u16 = 4;
const PLACEMENT_CANDIDATES: u64 = 128;

/// Attach deterministic geography to every region and location that lacks it.
/// Existing coordinates and anchors always win; no IDs, routes, history, or maps are changed.
pub fn ensure_location_geography(world: &mut World, content: Option<&CampaignContent>) {
    let seed = world.geography_seed.unwrap_or_else(|| compatibility_seed(world));
    world.geography_seed = Some(seed);
    let ruleset = if world.geography_ruleset_version != 0 {
        world.geography_ruleset_version
    } else {
        world.generation.as_ref().map(|g| g.geography_ruleset_version)
            .filter(|v| *v != 0).unwrap_or(GEOGRAPHY_RULESET_VERSION)
    };
    world.geography_ruleset_version = ruleset;
    embed_missing_region_coordinates(world);
    let authored = content.map(|c| c.world.locations.iter()
        .map(|location| (location.name.as_str(), location)).collect::<BTreeMap<_, _>>());

    for location in &world.locations {
        let Some(region) = world.regions.iter().find(|r| r.id == location.region_id) else { continue; };
        let Some(region_coordinate) = region.coordinate else { continue; };
        let authored_location = authored.as_ref().and_then(|entries| entries.get(location.name.as_str()).copied());
        let stable_key = authored_location.map(|entry| format!("location.content.{}", entry.id))
            .unwrap_or_else(|| format!("location.compat.{}", location.id));
        if let Some(existing) = world.location_geography.iter_mut().find(|r| r.location_id == location.id) {
            if existing.stable_key.starts_with("location.compat.") && authored_location.is_some() {
                existing.stable_key = stable_key;
            }
            continue;
        }
        let authored_anchor = authored_location.and_then(|e| e.geography.as_ref())
            .and_then(|g| g.anchor)
            .filter(|a| a.stratum == 0 && a.region_and_local().0 == region_coordinate);
        let radius = authored_location.and_then(|e| e.geography.as_ref())
            .and_then(|g| g.footprint_radius_samples)
            .unwrap_or(DEFAULT_FOOTPRINT_RADIUS_SAMPLES).min(32);
        let anchor = authored_anchor.unwrap_or_else(|| choose_anchor(seed, ruleset, region_coordinate, location.id, &stable_key, radius));
        let footprint_radius_samples = if authored_anchor.is_some() { radius } else { validated_footprint_radius(seed, ruleset, anchor, radius) };
        world.location_geography.push(LocationGeographyRecord { location_id: location.id, stable_key, anchor, footprint_radius_samples });
    }
    world.location_geography.sort_by_key(|r| r.location_id);
    world.geography_embedding_version = LOCATION_GEOGRAPHY_EMBEDDING_VERSION;
}

fn compatibility_seed(world: &World) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    fn feed(hash: &mut u64, value: u64) {
        for byte in value.to_le_bytes() { *hash ^= u64::from(byte); *hash = hash.wrapping_mul(0x100_0000_01b3); }
    }
    feed(&mut hash, world.id);
    feed(&mut hash, world.next_id);
    let mut regions = world.regions.iter().map(|r| r.id).collect::<Vec<_>>();
    regions.sort_unstable();
    for id in regions { feed(&mut hash, id); }
    let mut locations = world.locations.iter().map(|l| (l.id, l.region_id)).collect::<Vec<_>>();
    locations.sort_unstable();
    for (id, region) in locations { feed(&mut hash, id); feed(&mut hash, region); }
    let mut routes = world.routes.iter().collect::<Vec<_>>();
    routes.sort_by(|a, b| a.id.cmp(&b.id));
    for route in routes {
        for byte in route.id.as_bytes() { hash ^= u64::from(*byte); hash = hash.wrapping_mul(0x100_0000_01b3); }
        feed(&mut hash, route.location_a_id);
        feed(&mut hash, route.location_b_id);
        feed(&mut hash, route.direction as u64);
    }
    hash
}

fn embed_missing_region_coordinates(world: &mut World) {
    let mut ids = world.regions.iter().map(|r| r.id).collect::<Vec<_>>();
    ids.sort_unstable();
    let valid = ids.iter().copied().collect::<BTreeSet<_>>();
    let mut adjacency = BTreeMap::<u64, BTreeSet<u64>>::new();
    for id in &ids { adjacency.entry(*id).or_default(); }
    for route in &world.routes {
        let Some(a) = world.location_by_id(route.location_a_id).map(|l| l.region_id) else { continue; };
        let Some(b) = world.location_by_id(route.location_b_id).map(|l| l.region_id) else { continue; };
        if a != b && valid.contains(&a) && valid.contains(&b) {
            adjacency.entry(a).or_default().insert(b);
            adjacency.entry(b).or_default().insert(a);
        }
    }
    let mut order = Vec::with_capacity(ids.len());
    let mut visited = BTreeSet::new();
    for root in &ids {
        if !visited.insert(*root) { continue; }
        let mut queue = VecDeque::from([*root]);
        while let Some(current) = queue.pop_front() {
            order.push(current);
            for next in adjacency.get(&current).into_iter().flatten() {
                if visited.insert(*next) { queue.push_back(*next); }
            }
        }
    }
    let mut occupied = world.regions.iter().filter_map(|r| r.coordinate)
        .map(|c| (c.x, c.y, c.stratum)).collect::<BTreeSet<_>>();
    let mut next_x = 0_i64;
    for id in order {
        let Some(region) = world.regions.iter_mut().find(|r| r.id == id) else { continue; };
        if region.coordinate.is_some() { continue; }
        while occupied.contains(&(next_x, 0, 0)) { next_x = next_x.saturating_add(1); }
        region.coordinate = Some(RegionCoordinate::new(next_x, 0));
        occupied.insert((next_x, 0, 0));
        next_x = next_x.saturating_add(1);
    }
}

fn choose_anchor(seed: u64, ruleset: u32, region: RegionCoordinate, location_id: u64, stable_key: &str, radius: u16) -> MacroCoordinate {
    let Some(origin) = region.global_sample(0, 0) else { return MacroCoordinate::ORIGIN; };
    let key_hash = stable_key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)) ^ location_id.rotate_left(19);
    let mut best: Option<(i64, MacroCoordinate)> = None;
    for index in 0..PLACEMENT_CANDIDATES {
        let random = coordinate_seed(seed ^ key_hash, ruleset.wrapping_add(index as u32), origin);
        let coordinate = MacroCoordinate::new(origin.x.saturating_add((random & 0xff) as i64), origin.y.saturating_add(((random >> 32) & 0xff) as i64));
        let Some(sample) = terrain::sample_surface(seed, ruleset, coordinate) else { continue; };
        let footprint = validated_footprint_radius(seed, ruleset, coordinate, radius);
        let land_bonus = if sample.is_land() { 1_i64 << 40 } else { 0 };
        let footprint_bonus = if footprint == radius && radius > 0 { 1_i64 << 24 } else { 0 };
        let score = land_bonus + footprint_bonus + i64::from(sample.elevation_q16()) - i64::from(sample.mountain_ridge_strength_q16() / 4);
        if best.is_none_or(|(s, _)| score > s) { best = Some((score, coordinate)); }
    }
    best.map(|(_, c)| c).unwrap_or(origin)
}

fn validated_footprint_radius(seed: u64, ruleset: u32, anchor: MacroCoordinate, requested: u16) -> u16 {
    if requested == 0 { return 0; }
    let r = i64::from(requested);
    let offsets = [(r,0),(-r,0),(0,r),(0,-r),(r,r),(r,-r),(-r,r),(-r,-r)];
    if offsets.iter().all(|(dx,dy)| {
        anchor.x.checked_add(*dx).zip(anchor.y.checked_add(*dy))
            .and_then(|(x,y)| terrain::sample_surface(seed, ruleset, MacroCoordinate::new(x,y)))
            .is_some_and(|s| s.is_land())
    }) { requested } else { 0 }
}

#[cfg(test)]
#[path = "../../tests/unit/location_geography.rs"]
mod tests;
