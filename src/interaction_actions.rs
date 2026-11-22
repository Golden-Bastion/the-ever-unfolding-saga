//! Renderer-neutral discovery and identities for contextual world interactions.
//!
//! Discovery produces action/target pairs. Presentation may arrange them in a
//! radial wheel or another view; execution remains in authoritative gameplay.

use crate::model::{EntityId, GameState};
use crate::spatial::{
    DoorState, GridPosition, LocalMap, MapExit, SpatialEncounterStatus, SpatialEntityKind,
};

const TILE_HALF_WIDTH: f32 = 24.0;
const TILE_HALF_HEIGHT: f32 = 12.0;
const INTERACTION_RANGE_PX: f32 = 34.0;

pub(crate) type SpatialPosition = (f32, f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum InteractionActionKind {
    Talk,
    SearchRemains,
    Examine,
    Enter,
    OpenDoor,
    CloseDoor,
    TryLockedDoor,
    EngageEncounter,
}

impl InteractionActionKind {
    pub(crate) const fn stable_key(self) -> &'static str {
        match self {
            Self::Talk => "talk",
            Self::SearchRemains => "search_remains",
            Self::Examine => "examine",
            Self::Enter => "enter",
            Self::OpenDoor => "open_door",
            Self::CloseDoor => "close_door",
            Self::TryLockedDoor => "try_locked_door",
            Self::EngageEncounter => "engage_encounter",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum InteractionTarget {
    Npc(EntityId),
    Corpse(EntityId),
    PointOfInterest(String),
    Passage {
        id: String,
        destination_id: EntityId,
    },
    Door {
        id: String,
    },
    Encounter(String),
}

impl InteractionTarget {
    pub(crate) fn stable_key(&self) -> String {
        match self {
            Self::Npc(id) => format!("npc:{id}"),
            Self::Corpse(id) => format!("corpse:{id}"),
            Self::PointOfInterest(id) => format!("poi:{id}"),
            Self::Passage { id, destination_id } => format!("passage:{id}:{destination_id}"),
            Self::Door { id } => format!("door:{id}"),
            Self::Encounter(id) => format!("encounter:{id}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AvailableInteraction {
    pub action: InteractionActionKind,
    pub target: InteractionTarget,
    pub label: String,
}

impl AvailableInteraction {
    /// Stable identity for this action bound to this exact target.
    pub(crate) fn stable_key(&self) -> String {
        format!("{}:{}", self.action.stable_key(), self.target.stable_key())
    }
}

/// Discover every currently eligible action/target pair without depending on a
/// renderer or frontend input type. The tuple position is a continuous local
/// grid-space coordinate; projection is used only to preserve the established
/// three-quarter-view interaction radius and ordering.
pub(crate) fn discover_available_interactions(
    map: &LocalMap,
    game: &GameState,
    position: SpatialPosition,
) -> Vec<AvailableInteraction> {
    let mut candidates = Vec::new();

    for poi in &map.points_of_interest {
        if !poi.discovered
            || !poi.is_available_to(&game.character.known_evidence_ids)
            || game.character.knows_evidence(&poi.id)
            || !point_of_interest_interaction_available(map, position, poi)
        {
            continue;
        }
        let Some(distance) = point_of_interest_distance_px(map, position, poi) else {
            continue;
        };
        let item = AvailableInteraction {
            action: InteractionActionKind::Examine,
            target: InteractionTarget::PointOfInterest(poi.id.clone()),
            label: format!("Examine: {}", poi.title),
        };
        candidates.push((distance, 2_u8, item.stable_key(), item));
    }

    for placement in &map.interactables {
        let target_position = (
            f32::from(placement.position.x) + 0.5,
            f32::from(placement.position.y) + 0.5,
        );
        let distance = projected_distance(map, position, target_position);
        if distance > INTERACTION_RANGE_PX
            || !has_clear_line_of_sight(map, position, target_position)
        {
            continue;
        }
        let (priority, item) = match placement.kind {
            SpatialEntityKind::Npc => {
                let Some(npc) = game.npcs.iter().find(|npc| {
                    npc.id == placement.entity_id && npc.location_id == game.character.location_id
                }) else {
                    continue;
                };
                (
                    0_u8,
                    AvailableInteraction {
                        action: InteractionActionKind::Talk,
                        target: InteractionTarget::Npc(npc.id),
                        label: format!("Talk: {}", npc.name),
                    },
                )
            }
            SpatialEntityKind::Corpse => {
                let Some(corpse) = game.corpses.iter().find(|corpse| {
                    corpse.id == placement.entity_id
                        && corpse.location_id == game.character.location_id
                        && !corpse.inventory.is_empty()
                }) else {
                    continue;
                };
                (
                    1_u8,
                    AvailableInteraction {
                        action: InteractionActionKind::SearchRemains,
                        target: InteractionTarget::Corpse(corpse.id),
                        label: format!("Search: {}", corpse.former_name),
                    },
                )
            }
        };
        candidates.push((distance, priority, item.stable_key(), item));
    }

    let encounters_belong_here =
        map.location_id == format!("location.{}", game.character.location_id);
    if encounters_belong_here {
        for encounter in &map.encounters {
            if encounter.status != SpatialEncounterStatus::Active {
                continue;
            }
            let target_position = (
                f32::from(encounter.position.x) + 0.5,
                f32::from(encounter.position.y) + 0.5,
            );
            let distance = projected_distance(map, position, target_position);
            if distance > INTERACTION_RANGE_PX
                || !has_clear_line_of_sight(map, position, target_position)
            {
                continue;
            }
            let item = AvailableInteraction {
                action: InteractionActionKind::EngageEncounter,
                target: InteractionTarget::Encounter(encounter.id.clone()),
                label: format!("Confront: {}", encounter.enemy_name),
            };
            candidates.push((distance, 0_u8, item.stable_key(), item));
        }
    }

    for door in &map.doors {
        let midpoint = (
            (f32::from(door.edge.a.x) + f32::from(door.edge.b.x) + 1.0) * 0.5,
            (f32::from(door.edge.a.y) + f32::from(door.edge.b.y) + 1.0) * 0.5,
        );
        let distance = projected_distance(map, position, midpoint);
        if distance > INTERACTION_RANGE_PX || !has_clear_line_of_sight(map, position, midpoint) {
            continue;
        }
        let (action, label) = match door.state {
            DoorState::Open => (InteractionActionKind::CloseDoor, "Close door"),
            DoorState::Closed => (InteractionActionKind::OpenDoor, "Open door"),
            DoorState::Locked => (InteractionActionKind::TryLockedDoor, "Locked door"),
        };
        let item = AvailableInteraction {
            action,
            target: InteractionTarget::Door {
                id: door.id.clone(),
            },
            label: label.to_string(),
        };
        candidates.push((distance, 2_u8, item.stable_key(), item));
    }

    for exit in &map.exits {
        let pending_frontier = exit.frontier.is_some() && exit.target_location_id == 0;
        let pending_cave = exit.cave_target.is_some() && exit.target_location_id == 0;
        if (!pending_frontier
            && !pending_cave
            && !crate::game::actions::has_location_travel_route(
                game,
                exit.target_location_id,
                exit.route_id.as_deref(),
            ))
            || !map_exit_interaction_available(map, position, exit)
        {
            continue;
        }
        let label = if pending_cave {
            match exit.cave_target.as_ref().map(|target| target.kind) {
                Some(crate::geography::caves::CaveConnectionKind::SurfaceEntrance) => {
                    "Descend into cave".to_string()
                }
                Some(crate::geography::caves::CaveConnectionKind::VerticalShaft) => {
                    "Use vertical shaft".to_string()
                }
                Some(crate::geography::caves::CaveConnectionKind::HorizontalPassage) => {
                    "Enter cave passage".to_string()
                }
                None => "Use cave passage".to_string(),
            }
        } else if pending_frontier {
            let direction = exit
                .frontier
                .as_ref()
                .map(|frontier| frontier.direction.label())
                .unwrap_or("unknown");
            format!("Explore {direction} frontier")
        } else {
            let destination_name = game
                .world
                .location_by_id(exit.target_location_id)
                .map(|location| location.name.clone());
            let resolved_ferry = exit
                .route_id
                .as_deref()
                .and_then(|route_id| game.world.route_by_id(route_id))
                .and_then(|route| route.geography.as_ref())
                .is_some_and(|geography| geography.has_resolved_lake_crossing());
            if resolved_ferry {
                format!(
                    "Board ferry to {}",
                    destination_name.as_deref().unwrap_or("unknown destination")
                )
            } else if let Some(destination_name) = destination_name {
                format!("Enter: {destination_name}")
            } else {
                "Use passage".to_string()
            }
        };
        let item = AvailableInteraction {
            action: InteractionActionKind::Enter,
            target: InteractionTarget::Passage {
                id: exit.id.clone(),
                destination_id: exit.target_location_id,
            },
            label,
        };
        candidates.push((
            map_exit_distance_px(map, position, exit),
            3_u8,
            item.stable_key(),
            item,
        ));
    }

    candidates.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then(a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    candidates.into_iter().map(|(_, _, _, item)| item).collect()
}

/// Check whether a point of interest is within interaction range and has an
/// unobstructed approach.
pub(crate) fn point_of_interest_interaction_available(
    map: &LocalMap,
    position: SpatialPosition,
    poi: &crate::spatial::PointOfInterest,
) -> bool {
    if !poi.discovered
        || point_of_interest_distance_px(map, position, poi)
            .is_none_or(|distance| distance > INTERACTION_RANGE_PX)
    {
        return false;
    }
    let target = (
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );
    has_clear_line_of_sight(map, position, target)
}

/// Check whether a physical passage is in range, walkable, and approachable.
pub(crate) fn map_exit_interaction_available(
    map: &LocalMap,
    position: SpatialPosition,
    exit: &MapExit,
) -> bool {
    if !map.contains(exit.position)
        || !map.is_walkable(exit.position)
        || map_exit_distance_px(map, position, exit) > INTERACTION_RANGE_PX
    {
        return false;
    }
    let target = (
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    has_clear_line_of_sight(map, position, target)
}

/// Grid-space line of sight using the same prop/terrain walkability rules as
/// movement and the existing spatial interaction implementation.
pub(crate) fn has_clear_line_of_sight(
    map: &LocalMap,
    start: SpatialPosition,
    target: SpatialPosition,
) -> bool {
    let distance = grid_distance(start, target);
    let steps = (distance * 8.0).ceil() as u32;
    for step in 1..steps {
        let progress = step as f32 / steps.max(1) as f32;
        let sample = (
            start.0 + (target.0 - start.0) * progress,
            start.1 + (target.1 - start.1) * progress,
        );
        if sample.0 < 0.0 || sample.1 < 0.0 {
            return false;
        }
        let tile = GridPosition {
            x: sample.0.floor() as u16,
            y: sample.1.floor() as u16,
        };
        if !map.is_walkable(tile) {
            return false;
        }
    }
    true
}

fn point_of_interest_distance_px(
    map: &LocalMap,
    position: SpatialPosition,
    poi: &crate::spatial::PointOfInterest,
) -> Option<f32> {
    if !map.contains(poi.position) {
        return None;
    }
    let target = (
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );
    Some(projected_distance(map, position, target))
}

fn map_exit_distance_px(map: &LocalMap, position: SpatialPosition, exit: &MapExit) -> f32 {
    let target = (
        f32::from(exit.position.x) + 0.5,
        f32::from(exit.position.y) + 0.5,
    );
    projected_distance(map, position, target)
}

fn projected_distance(map: &LocalMap, start: SpatialPosition, target: SpatialPosition) -> f32 {
    let a = project_position(map, start);
    let b = project_position(map, target);
    (b.0 - a.0).hypot(b.1 - a.1)
}

fn grid_distance(start: SpatialPosition, target: SpatialPosition) -> f32 {
    (target.0 - start.0).hypot(target.1 - start.1)
}

fn project_position(map: &LocalMap, position: SpatialPosition) -> SpatialPosition {
    let width = f32::from(map.width);
    let height = f32::from(map.height);
    (
        (position.0 - position.1 - (width - height) * 0.5) * TILE_HALF_WIDTH,
        ((width + height) * 0.5 - position.0 - position.1) * TILE_HALF_HEIGHT,
    )
}

#[cfg(test)]
#[path = "../tests/unit/interaction_actions.rs"]
mod tests;
