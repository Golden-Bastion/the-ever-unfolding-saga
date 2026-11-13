//! Renderer-neutral world-map view derived from established locations and routes.
//!
//! The interim map deliberately contains no geography or persistent layout data.
//! Its visible graph is based on locations with persistent local maps, plus legacy
//! visits recoverable from retained character-lineage travel history and their exits.
//! Presentation code arranges these stable identities.

use std::collections::{HashMap, HashSet};

use crate::model::{GameState, Location};
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
    let is_explored = |location_id: u64| {
        crate::game::actions::has_visited_location(state, location_id)
    };

    let mut visible_ids = HashSet::from([current.id]);
    for location in &state.world.locations {
        if location.id == current.id || is_explored(location.id) {
            visible_ids.insert(location.id);
            visible_ids.extend(location.exits.iter().copied());
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
                region_id: location.region_id,
                explored: location.id == current.id || is_explored(location.id),
                current: location.id == current.id,
                reachable: crate::game::actions::has_reciprocal_location_connection(
                    state,
                    location.id,
                ),
                fast_travel_index: fast_travel.map(|(index, _)| index),
                fast_travel_steps: fast_travel.map(|(_, steps)| steps),
            }
        })
        .collect();

    let mut routes: Vec<WorldMapRouteView> = Vec::new();
    for source in &state.world.locations {
        if source.id != current.id && !is_explored(source.id) {
            continue;
        }
        for target_id in &source.exits {
            if !visible_ids.contains(target_id) {
                continue;
            }
            let Some(target) = state.world.location_by_id(*target_id) else {
                continue;
            };
            let (first_id, second_id) = if source.id <= target.id {
                (source.id, target.id)
            } else {
                (target.id, source.id)
            };
            let bidirectional = target.exits.contains(&source.id);
            if let Some(existing) = routes.iter_mut().find(|route| {
                let key = if route.from_id <= route.to_id {
                    (route.from_id, route.to_id)
                } else {
                    (route.to_id, route.from_id)
                };
                key == (first_id, second_id)
            }) {
                existing.bidirectional |= bidirectional;
            } else {
                routes.push(WorldMapRouteView {
                    // Retain the actual source -> destination orientation for one-way routes.
                    from_id: source.id,
                    to_id: target.id,
                    bidirectional,
                });
            }
        }
    }

    let current_location = Some(location_view(state, current, &campaign));

    WorldMapView {
        current_location,
        nodes,
        routes,
        fast_travel_destinations,
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
