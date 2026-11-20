use crate::content::definitions::CampaignContent;
use crate::geography::RegionCoordinate;
use crate::model::{Location, Region, RouteDirection, World, WorldRoute};

impl CampaignContent {
    pub fn seed_world(&self, world: &mut World) -> usize {
        if world.regions.is_empty() {
            let region_id = world.allocate_id();
            world.regions.push(Region {
                id: region_id,
                name: self.world.region.name.clone(),
                description: self.world.region.description.clone(),
                location_ids: Vec::new(),
                coordinate: Some(RegionCoordinate::ORIGIN),
            });
        }

        let region_id = world
            .regions
            .first()
            .map(|region| region.id)
            .unwrap_or_else(|| {
                let id = world.allocate_id();
                world.regions.push(Region {
                    id,
                    name: self.world.region.name.clone(),
                    description: self.world.region.description.clone(),
                    location_ids: Vec::new(),
                    coordinate: Some(RegionCoordinate::ORIGIN),
                });
                id
            });

        let mut added = 0usize;
        for location in &self.world.locations {
            if world.location_by_name(&location.name).is_none() {
                let id = world.allocate_id();
                world.locations.push(Location {
                    id,
                    name: location.name.clone(),
                    description: location.description.clone(),
                    region_id,
                    dangerous: location.dangerous,
                    corpse_ids: Vec::new(),
                    exits: Vec::new(),
                    spatial_layout: location.spatial_layout.clone(),
                });
                added += 1;
            }
        }

        let authored_runtime_ids = self
            .world
            .locations
            .iter()
            .filter_map(|location| {
                world
                    .location_by_name(&location.name)
                    .map(|runtime| (location.id.as_str(), runtime.id))
            })
            .collect::<std::collections::HashMap<_, _>>();
        let mut authored_edges = std::collections::BTreeMap::<(u64, u64), (bool, bool)>::new();

        for location in &self.world.locations {
            let Some(source_id) = authored_runtime_ids.get(location.id.as_str()).copied() else {
                continue;
            };
            if let Some(world_location) = world.location_by_name_mut(&location.name) {
                world_location.description = location.description.clone();
                world_location.dangerous = location.dangerous;
                world_location.spatial_layout = location.spatial_layout.clone();
            }
            for exit_id in &location.exits {
                let Some(target_id) = authored_runtime_ids.get(exit_id.as_str()).copied() else {
                    continue;
                };
                if source_id == target_id {
                    continue;
                }
                let (a, b, from_a) = if source_id < target_id {
                    (source_id, target_id, true)
                } else {
                    (target_id, source_id, false)
                };
                let edge = authored_edges.entry((a, b)).or_insert((false, false));
                if from_a {
                    edge.0 = true;
                } else {
                    edge.1 = true;
                }
            }
        }

        for ((a, b), (a_to_b, b_to_a)) in authored_edges {
            let direction = match (a_to_b, b_to_a) {
                (true, true) => RouteDirection::Bidirectional,
                (true, false) => RouteDirection::AtoB,
                (false, true) => RouteDirection::BtoA,
                (false, false) => continue,
            };
            let route_id = format!("route.content.{a}.{b}");
            if world.route_by_id(&route_id).is_none() {
                world.add_route(WorldRoute::new(route_id, a, b, direction));
            }
        }
        world.rebuild_location_exit_cache();

        if let Some(region) = world
            .regions
            .iter_mut()
            .find(|region| region.id == region_id)
        {
            region.name = self.world.region.name.clone();
            region.description = self.world.region.description.clone();
            region.location_ids = world
                .locations
                .iter()
                .filter(|location| location.region_id == region_id)
                .map(|location| location.id)
                .collect();
        }

        added
    }
}

#[cfg(test)]
#[path = "../../tests/unit/content/seeding.rs"]
mod tests;
