//! Renderer-neutral world-map view derived from established locations and routes.
//!
//! The interim map deliberately contains no geography or persistent layout data.
//! Its visible graph is based on locations with persistent local maps, plus legacy
//! visits recoverable from retained character-lineage travel history and their exits.
//! Presentation code arranges these stable identities.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::model::{GameState, Location, QuestObjectiveKind};
use crate::presentation::{
    LocationView, WorldMapDestinationView, WorldMapNodeView, WorldMapRouteView, WorldMapView,
};

pub(crate) fn build_view(state: &GameState) -> WorldMapView {
    let Some(current) = state.world.location_by_id(state.character.location_id) else {
        return WorldMapView::default();
    };

    let campaign = state
        .campaign_content
        .clone()
        .unwrap_or_else(crate::content::load_campaign_content);
    let is_explored =
        |location_id: u64| crate::game::actions::has_visited_location(state, location_id);
    let tracked_destination_id = tracked_quest_destination_id(state, &campaign);

    let mut visible_ids = HashSet::from([current.id]);
    if let Some(destination_id) = tracked_destination_id {
        // Tracking is character-known information, independent of world cartography.
        visible_ids.insert(destination_id);
    }
    for route in &state.world.routes {
        if state.world.is_route_discovered(&route.id) {
            visible_ids.insert(route.location_a_id);
            visible_ids.insert(route.location_b_id);
        }
    }
    visible_ids.retain(|id| state.world.location_by_id(*id).is_some());

    let mut fast_travel_destinations = state
        .world
        .locations
        .iter()
        .filter(|location| location.id != current.id && is_explored(location.id))
        .filter_map(|location| {
            crate::game::actions::find_known_travel_path(state, location.id).map(|path| {
                WorldMapDestinationView {
                    location: location_view(state, location, &campaign),
                    route_steps: path.len(),
                }
            })
        })
        .collect::<Vec<_>>();
    fast_travel_destinations.sort_by(|left, right| {
        left.location
            .region_name
            .cmp(&right.location.region_name)
            .then_with(|| left.location.name.cmp(&right.location.name))
            .then_with(|| left.location.id.cmp(&right.location.id))
    });
    let fast_travel_indices = fast_travel_destinations
        .iter()
        .enumerate()
        .map(|(index, destination)| (destination.location.id, (index, destination.route_steps)))
        .collect::<HashMap<_, _>>();

    let nodes = state
        .world
        .locations
        .iter()
        .filter(|location| visible_ids.contains(&location.id))
        .map(|location| {
            let fast_travel = fast_travel_indices.get(&location.id).copied();
            WorldMapNodeView {
                location: location_view(state, location, &campaign),
                tracked_objective: tracked_destination_id == Some(location.id),
                region_id: location.region_id,
                explored: location.id == current.id || is_explored(location.id),
                current: location.id == current.id,
                reachable: state.world.routes_from(state.character.location_id).any(
                    |(route, destination)| {
                        destination == location.id && state.world.is_route_discovered(&route.id)
                    },
                ),
                fast_travel_index: fast_travel.map(|(index, _)| index),
                fast_travel_steps: fast_travel.map(|(_, steps)| steps),
            }
        })
        .collect();

    let mut route_directions = std::collections::BTreeMap::<(u64, u64), (bool, bool)>::new();
    for route in &state.world.routes {
        if !state.world.is_route_discovered(&route.id) {
            continue;
        }
        {
            let a = route.location_a_id;
            let b = route.location_b_id;
            let directions = route_directions.entry((a, b)).or_insert((false, false));
            directions.0 |= route.allows_travel(a, b);
            directions.1 |= route.allows_travel(b, a);
        }
    }

    let routes: Vec<WorldMapRouteView> = route_directions
        .into_iter()
        .filter_map(|((a, b), (a_to_b, b_to_a))| {
            if a_to_b {
                Some(WorldMapRouteView {
                    from_id: a,
                    to_id: b,
                    bidirectional: b_to_a,
                })
            } else if b_to_a {
                Some(WorldMapRouteView {
                    from_id: b,
                    to_id: a,
                    bidirectional: false,
                })
            } else {
                None
            }
        })
        .collect();

    let current_location = Some(location_view(state, current, &campaign));

    WorldMapView {
        current_location,
        current_region_overview: region_overview(state, current.region_id),
        nodes,
        routes,
        fast_travel_destinations,
    }
}

fn region_overview(state: &GameState, region_id: u64) -> Vec<String> {
    if !state.world.is_region_surveyed(region_id) {
        return Vec::new();
    }
    let Some(region) = state.world.regions.iter().find(|region| region.id == region_id) else {
        return Vec::new();
    };
    let Some(coordinate) = region.coordinate else {
        return Vec::new();
    };
    let seed = state
        .world
        .geography_seed
        .or_else(|| state.world.generation.map(|generation| generation.seed));
    let Some(seed) = seed else {
        return Vec::new();
    };
    let ruleset = if state.world.geography_ruleset_version == 0 {
        state.world.generation.map_or(0, |generation| generation.geography_ruleset_version)
    } else {
        state.world.geography_ruleset_version
    };

    let mut features = BTreeMap::<&'static str, usize>::new();
    for y in [32, 96, 160, 224] {
        for x in [32, 96, 160, 224] {
            let Some(sample) = coordinate.global_sample(x, y) else {
                continue;
            };
            let Some(climate) = crate::geography::climate::sample_surface(seed, ruleset, sample) else {
                continue;
            };
            let label = match climate.biome() {
                crate::geography::climate::Biome::Ocean => "Ocean",
                crate::geography::climate::Biome::Coast => "Coast",
                crate::geography::climate::Biome::Desert => "Desert",
                crate::geography::climate::Biome::Grassland => "Open country",
                crate::geography::climate::Biome::TemperateForest => "Forest",
                crate::geography::climate::Biome::Taiga => "Taiga",
                crate::geography::climate::Biome::Tundra => "Tundra",
                crate::geography::climate::Biome::Alpine => "Highlands",
            };
            *features.entry(label).or_default() += 1;
            if crate::geography::terrain::sample_surface(seed, ruleset, sample)
                .is_some_and(|terrain| terrain.is_mountain_barrier())
            {
                *features.entry("Mountain ridges").or_default() += 1;
            }
        }
    }

    let mut overview = features.into_iter().collect::<Vec<_>>();
    overview.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    overview
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .take(4)
        .map(|(label, _)| label.to_string())
        .collect()
}

fn tracked_quest_destination_id(
    state: &GameState,
    campaign: &crate::content::CampaignContent,
) -> Option<u64> {
    let tracked_id = state.character.tracked_quest_id?;
    let quest = state
        .quests
        .iter()
        .find(|quest| quest.id == tracked_id && quest.offered && !quest.completed)?;
    let objective = quest
        .objectives
        .iter()
        .find(|objective| !objective.completed)?;

    match &objective.kind {
        QuestObjectiveKind::VisitLocation => {
            let mut matching_locations = state
                .world
                .locations
                .iter()
                .filter(|location| location.name == objective.target);
            match (matching_locations.next(), matching_locations.next()) {
                (Some(location), None) => Some(location.id),
                (None, _) => objective
                    .target
                    .parse::<u64>()
                    .ok()
                    .and_then(|location_id| state.world.location_by_id(location_id))
                    .map(|location| location.id),
                // A duplicated name cannot safely identify a single destination.
                (Some(_), Some(_)) => None,
            }
        }
        QuestObjectiveKind::DiscoverEvidence => {
            let mut matching_locations = campaign.world.locations.iter().filter(|location| {
                location
                    .spatial_discoveries
                    .iter()
                    .any(|discovery| discovery.id == objective.target)
            });
            let content_location = match (matching_locations.next(), matching_locations.next()) {
                (Some(location), None) => location,
                _ => return None,
            };
            let mut world_locations = state
                .world
                .locations
                .iter()
                .filter(|location| location.name == content_location.name);
            match (world_locations.next(), world_locations.next()) {
                (Some(location), None) => Some(location.id),
                _ => None,
            }
        }
        QuestObjectiveKind::AcquireItem | QuestObjectiveKind::DefeatEnemy => state
            .world
            .location_by_id(quest.target_location_id)
            .map(|location| location.id),
    }
}

fn location_view(
    state: &GameState,
    location: &Location,
    campaign: &crate::content::CampaignContent,
) -> LocationView {
    let region_name = state
        .world
        .regions
        .iter()
        .find(|region| region.id == location.region_id)
        .map(|region| region.name.clone())
        .unwrap_or_else(|| "Unknown region".to_string());

    LocationView {
        id: location.id,
        name: location.name.clone(),
        description: location.description.clone(),
        region_name,
        dangerous: location.dangerous,
        scene_image: campaign
            .location_scene_image_for(&location.name)
            .map(str::to_string),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/game/world_map.rs"]
mod tests;
