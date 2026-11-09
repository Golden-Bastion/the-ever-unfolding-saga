//! Renderer-neutral world-map view derived from established locations and routes.
//!
//! The interim map deliberately contains no geography or persistent layout data.
//! Its visible graph is based on locations whose local maps have been established,
//! plus their immediate exits. Presentation code arranges these stable identities.

use std::collections::HashSet;

use crate::model::{GameState, Location};
use crate::presentation::{LocationView, WorldMapNodeView, WorldMapRouteView, WorldMapView};

pub(crate) fn build_view(state: &GameState) -> WorldMapView {
    let Some(current) = state.world.location_by_id(state.character.location_id) else {
        return WorldMapView::default();
    };

    let campaign = state
        .campaign_content
        .clone()
        .unwrap_or_else(crate::content::load_campaign_content);
    let explored_ids = state
        .world
        .local_maps
        .iter()
        .map(|map| map.location_id.as_str())
        .collect::<HashSet<_>>();

    let is_explored = |location_id: u64| {
        let stable_id = format!("location.{location_id}");
        explored_ids.contains(stable_id.as_str())
    };

    let mut visible_ids = HashSet::from([current.id]);
    for location in &state.world.locations {
        if location.id == current.id || is_explored(location.id) {
            visible_ids.insert(location.id);
            visible_ids.extend(location.exits.iter().copied());
        }
    }
    visible_ids.retain(|id| state.world.location_by_id(*id).is_some());

    let nodes = state
        .world
        .locations
        .iter()
        .filter(|location| visible_ids.contains(&location.id))
        .map(|location| WorldMapNodeView {
            location: location_view(state, location, &campaign),
            region_id: location.region_id,
            explored: location.id == current.id || is_explored(location.id),
            current: location.id == current.id,
            reachable: current.exits.contains(&location.id),
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
            let (from_id, to_id) = if source.id <= target.id {
                (source.id, target.id)
            } else {
                (target.id, source.id)
            };
            let bidirectional = target.exits.contains(&source.id);
            if let Some(existing) = routes
                .iter_mut()
                .find(|route| route.from_id == from_id && route.to_id == to_id)
            {
                existing.bidirectional |= bidirectional;
            } else {
                routes.push(WorldMapRouteView {
                    from_id,
                    to_id,
                    bidirectional,
                });
            }
        }
    }

    let current_location = Some(location_view(state, current, &campaign));
    let destinations = current
        .exits
        .iter()
        .filter_map(|id| state.world.location_by_id(*id))
        .map(|location| location_view(state, location, &campaign))
        .collect();

    WorldMapView {
        current_location,
        nodes,
        routes,
        destinations,
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
