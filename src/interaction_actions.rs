//! Renderer-neutral identities for contextual world interactions.
//!
//! Discovery produces action/target pairs. Presentation may arrange them in a
//! radial wheel or another view; execution remains in authoritative gameplay.

use crate::model::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum InteractionActionKind {
    Talk,
    SearchRemains,
    Examine,
    Enter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum InteractionTarget {
    Npc(EntityId),
    Corpse(EntityId),
    PointOfInterest(String),
    Passage { id: String, destination_id: EntityId },
}

impl InteractionTarget {
    pub(crate) fn stable_key(&self) -> String {
        match self {
            Self::Npc(id) => format!("npc:{id}"),
            Self::Corpse(id) => format!("corpse:{id}"),
            Self::PointOfInterest(id) => format!("poi:{id}"),
            Self::Passage { id, destination_id } => format!("passage:{id}:{destination_id}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AvailableInteraction {
    pub action: InteractionActionKind,
    pub target: InteractionTarget,
    pub label: String,
}

#[cfg(test)]
#[path = "../tests/unit/interaction_actions.rs"]
mod tests;
