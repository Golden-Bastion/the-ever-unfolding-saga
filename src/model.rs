use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::content::{load_campaign_content, CampaignContent};
use crate::geography::{RegionCoordinate, GEOGRAPHY_RULESET_VERSION};
use crate::spatial::{ExplorationSession, LocalMap};

pub type EntityId = u64;
const MAX_HISTORY_ENTRIES: usize = 500;

pub(crate) fn is_internal_memory_entry(entry: &str) -> bool {
    entry.starts_with("[authored anchor]")
        || entry.starts_with("[generated relationship]")
        || entry.starts_with("A generated faction shaped by")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldMode {
    New,
    Inherited,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldGenerationMetadata {
    pub seed: u64,
    pub region_count: usize,
    pub location_count: usize,
    pub extra_edges: usize,
    /// Version of the deterministic macro-geography rules used by this world.
    /// Zero is retained for saves created before geography metadata existed.
    #[serde(default)]
    pub geography_ruleset_version: u32,
}

/// Direction in which a persistent world route can be travelled.
/// Endpoint A is always the lower numeric location ID, so directionality has
/// the same meaning regardless of the order used when constructing a route.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RouteDirection {
    #[default]
    Bidirectional,
    AtoB,
    BtoA,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldRoute {
    /// Stable route identity, independent of display names and map positions.
    pub id: String,
    /// Lower of the two endpoint location IDs.
    pub location_a_id: EntityId,
    /// Higher of the two endpoint location IDs.
    pub location_b_id: EntityId,
    pub direction: RouteDirection,
}

impl WorldRoute {
    pub fn new(
        id: impl Into<String>,
        first_location_id: EntityId,
        second_location_id: EntityId,
        direction: RouteDirection,
    ) -> Self {
        let (location_a_id, location_b_id, direction) = if first_location_id <= second_location_id {
            (first_location_id, second_location_id, direction)
        } else {
            let direction = match direction {
                RouteDirection::Bidirectional => RouteDirection::Bidirectional,
                RouteDirection::AtoB => RouteDirection::BtoA,
                RouteDirection::BtoA => RouteDirection::AtoB,
            };
            (second_location_id, first_location_id, direction)
        };
        Self {
            id: id.into(),
            location_a_id,
            location_b_id,
            direction,
        }
    }

    pub fn connects(&self, first_location_id: EntityId, second_location_id: EntityId) -> bool {
        (self.location_a_id == first_location_id && self.location_b_id == second_location_id)
            || (self.location_a_id == second_location_id && self.location_b_id == first_location_id)
    }

    pub fn allows_travel(&self, source_id: EntityId, destination_id: EntityId) -> bool {
        if source_id == self.location_a_id && destination_id == self.location_b_id {
            return matches!(
                self.direction,
                RouteDirection::Bidirectional | RouteDirection::AtoB
            );
        }
        if source_id == self.location_b_id && destination_id == self.location_a_id {
            return matches!(
                self.direction,
                RouteDirection::Bidirectional | RouteDirection::BtoA
            );
        }
        false
    }

    pub fn destination_from(&self, source_id: EntityId) -> Option<EntityId> {
        if self.allows_travel(source_id, self.location_b_id) {
            Some(self.location_b_id)
        } else if self.allows_travel(source_id, self.location_a_id) {
            Some(self.location_a_id)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreatState {
    pub active: bool,
    pub source_location_id: Option<EntityId>,
    pub label: String,
    pub description: String,
}

impl ThreatState {
    pub fn activate(
        &mut self,
        source_location_id: EntityId,
        label: impl Into<String>,
        description: impl Into<String>,
    ) {
        self.active = true;
        self.source_location_id = Some(source_location_id);
        self.label = label.into();
        self.description = description.into();
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub id: EntityId,
    pub name: String,
    pub mode: WorldMode,
    pub next_id: EntityId,
    #[serde(default)]
    pub generation: Option<WorldGenerationMetadata>,
    pub regions: Vec<Region>,
    pub locations: Vec<Location>,
    /// Authoritative persistent connections between world locations.
    #[serde(default)]
    pub routes: Vec<WorldRoute>,
    pub history: Vec<HistoryEntry>,
    #[serde(default)]
    pub time_points: u32,
    #[serde(default)]
    pub day: u32,
    #[serde(default, alias = "completed_quest_titles")]
    pub completed_quest_ids: Vec<String>,
    #[serde(default)]
    pub event_cooldowns: Vec<EventCooldown>,
    /// World-persistent local maps and their lasting discoveries/changes.
    #[serde(default)]
    pub local_maps: Vec<LocalMap>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EventCooldown {
    pub event_id: String,
    pub ready_at_turn: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: EntityId,
    pub name: String,
    pub description: String,
    pub location_ids: Vec<EntityId>,
    /// Stable signed chunk coordinate in the shared macro-geography lattice.
    /// `None` is reserved for existing worlds awaiting compatibility embedding.
    #[serde(default)]
    pub coordinate: Option<RegionCoordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: EntityId,
    pub name: String,
    pub description: String,
    pub region_id: EntityId,
    #[serde(default)]
    pub dangerous: bool,
    #[serde(default)]
    pub corpse_ids: Vec<EntityId>,
    /// Legacy adjacency data read from pre-route saves. Rebuilt from world routes
    /// after loading and omitted from new saves to avoid duplicate authority.
    #[serde(default, skip_serializing)]
    pub exits: Vec<EntityId>,
    /// Content-selected spatial layout for newly generated local maps.
    /// Existing maps remain authoritative after creation.
    #[serde(default)]
    pub spatial_layout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: EntityId,
    pub turn: u32,
    pub text: String,
    #[serde(default)]
    pub entry_type: HistoryEntryType,
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub location_name: Option<String>,
    #[serde(default)]
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum HistoryEntryType {
    #[default]
    Narrative,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: EntityId,
    pub name: String,
    pub title: String,
    /// Stable modular portrait-part IDs selected during character creation.
    #[serde(default)]
    pub appearance_parts: Vec<String>,
    /// Evidence this character has personally examined and understood.
    /// World maps retain physical discoveries separately across lives.
    #[serde(default)]
    pub known_evidence_ids: Vec<String>,
    /// The active quest this character has chosen to follow on the World Map.
    #[serde(default)]
    pub tracked_quest_id: Option<EntityId>,
    pub hp: i32,
    pub max_hp: i32,
    pub location_id: EntityId,
    pub inventory: Vec<Item>,
    pub alive: bool,
    pub turn: u32,
    #[serde(default)]
    pub experience: u32,
    #[serde(default = "default_character_level")]
    pub level: u32,
    #[serde(default)]
    pub attributes: Attributes,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    pub notes: Vec<String>,
    /// Character-specific active spatial exploration session, if any.
    #[serde(default)]
    pub exploration_session: Option<ExplorationSession>,
    /// Exact continuous local-map position in tile coordinates.
    #[serde(default)]
    pub exploration_position: Option<ExplorationPosition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExplorationPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Attributes {
    #[serde(default)]
    pub might: i32,
    #[serde(default)]
    pub insight: i32,
    #[serde(default)]
    pub endurance: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub name: String,
    #[serde(default)]
    pub remaining: u32,
    #[serde(default)]
    pub penalty: i32,
    #[serde(default)]
    pub bonus: i32,
}

impl Condition {
    pub fn new(name: impl Into<String>, remaining: u32, penalty: i32) -> Self {
        Self {
            name: name.into(),
            remaining,
            penalty,
            bonus: 0,
        }
    }
}

fn default_character_level() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: EntityId,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Corpse {
    pub id: EntityId,
    pub former_name: String,
    pub former_title: String,
    pub location_id: EntityId,
    pub turn_of_death: u32,
    pub inventory: Vec<Item>,
    pub epitaph: String,
    #[serde(default)]
    pub scavenged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Faction {
    pub id: EntityId,
    pub name: String,
    #[serde(default)]
    pub reputation: i32,
    #[serde(default)]
    pub memory: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Npc {
    pub id: EntityId,
    pub name: String,
    pub title: String,
    pub location_id: EntityId,
    #[serde(default)]
    pub faction_id: Option<EntityId>,
    #[serde(default)]
    pub memory: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuestObjectiveKind {
    #[default]
    AcquireItem,
    VisitLocation,
    DefeatEnemy,
    DiscoverEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuestObjective {
    pub kind: QuestObjectiveKind,
    pub target: String,
    #[serde(default = "default_objective_required")]
    pub required: u32,
    #[serde(default)]
    pub progress: u32,
    #[serde(default)]
    pub completed: bool,
    /// Optional presentation label; target remains the stable evidence ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

fn default_objective_required() -> u32 {
    1
}

impl QuestObjective {
    pub fn new(kind: QuestObjectiveKind, target: impl Into<String>, required: u32) -> Self {
        let required = required.max(1);
        Self {
            kind,
            target: target.into(),
            required,
            progress: 0,
            completed: false,
            target_label: None,
        }
    }

    pub fn discover_evidence(evidence_id: impl Into<String>, label: impl Into<String>) -> Self {
        let mut objective = Self::new(QuestObjectiveKind::DiscoverEvidence, evidence_id, 1);
        objective.target_label = Some(label.into());
        objective
    }

    pub fn display_label(&self) -> String {
        match self.kind {
            QuestObjectiveKind::AcquireItem => format!("Acquire {}", self.target),
            QuestObjectiveKind::VisitLocation => format!("Visit {}", self.target),
            QuestObjectiveKind::DefeatEnemy => format!(
                "Defeat {}",
                self.target_label.as_deref().unwrap_or(&self.target)
            ),
            QuestObjectiveKind::DiscoverEvidence => format!(
                "Discover {}",
                self.target_label.as_deref().unwrap_or(&self.target)
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Quest {
    pub id: EntityId,
    #[serde(default)]
    pub content_id: String,
    pub title: String,
    pub description: String,
    pub target_location_id: EntityId,
    pub faction_id: EntityId,
    #[serde(default)]
    pub giver_npc_id: EntityId,
    #[serde(default)]
    pub required_item_name: String,
    #[serde(default)]
    pub reward_item_name: String,
    #[serde(default)]
    pub objectives: Vec<QuestObjective>,
    #[serde(default)]
    pub completed_by: Option<String>,
    #[serde(default)]
    pub offered: bool,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub reward_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub world: World,
    pub character: Character,
    #[serde(default)]
    pub threat: ThreatState,
    #[serde(default)]
    pub corpses: Vec<Corpse>,
    #[serde(default)]
    pub factions: Vec<Faction>,
    #[serde(default)]
    pub npcs: Vec<Npc>,
    #[serde(default)]
    pub quests: Vec<Quest>,
    #[serde(default)]
    pub last_announced_location_id: Option<EntityId>,
    #[serde(default)]
    pub rng_state: u64,
    #[serde(skip)]
    pub campaign_content: Option<CampaignContent>,
}

impl World {
    pub fn new(name: &str, mode: WorldMode) -> Self {
        Self {
            id: 1,
            name: name.to_string(),
            mode,
            next_id: 2,
            generation: None,
            regions: Vec::new(),
            locations: Vec::new(),
            routes: Vec::new(),
            history: Vec::new(),
            time_points: 3,
            day: 1,
            completed_quest_ids: Vec::new(),
            event_cooldowns: Vec::new(),
            local_maps: Vec::new(),
        }
    }

    pub fn allocate_id(&mut self) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Insert a route after validating its identity and both endpoints.
    /// Routes are sorted by stable ID to keep traversal deterministic.
    pub fn add_route(&mut self, route: WorldRoute) -> bool {
        if route.id.trim().is_empty()
            || route.location_a_id == route.location_b_id
            || self.routes.iter().any(|existing| existing.id == route.id)
            || self.location_by_id(route.location_a_id).is_none()
            || self.location_by_id(route.location_b_id).is_none()
        {
            return false;
        }
        self.routes.push(route);
        self.routes.sort_by(|left, right| left.id.cmp(&right.id));
        self.rebuild_location_exit_cache();
        true
    }

    pub fn remove_route(&mut self, route_id: &str) -> bool {
        let original_len = self.routes.len();
        self.routes.retain(|route| route.id != route_id);
        let removed = self.routes.len() != original_len;
        if removed {
            self.rebuild_location_exit_cache();
        }
        removed
    }

    pub fn remove_routes_between(
        &mut self,
        first_location_id: EntityId,
        second_location_id: EntityId,
    ) -> usize {
        let original_len = self.routes.len();
        self.routes
            .retain(|route| !route.connects(first_location_id, second_location_id));
        let removed = original_len - self.routes.len();
        if removed > 0 {
            self.rebuild_location_exit_cache();
        }
        removed
    }

    pub fn route_by_id(&self, route_id: &str) -> Option<&WorldRoute> {
        self.routes.iter().find(|route| route.id == route_id)
    }

    pub fn routes_from(
        &self,
        source_id: EntityId,
    ) -> impl Iterator<Item = (&WorldRoute, EntityId)> {
        self.routes.iter().filter_map(move |route| {
            route
                .destination_from(source_id)
                .map(|destination_id| (route, destination_id))
        })
    }

    pub fn can_travel_between(&self, source_id: EntityId, destination_id: EntityId) -> bool {
        self.routes
            .iter()
            .any(|route| route.allows_travel(source_id, destination_id))
    }

    pub fn can_travel_via_route(
        &self,
        route_id: &str,
        source_id: EntityId,
        destination_id: EntityId,
    ) -> bool {
        self.route_by_id(route_id)
            .is_some_and(|route| route.allows_travel(source_id, destination_id))
    }

    pub fn has_route_between(
        &self,
        first_location_id: EntityId,
        second_location_id: EntityId,
    ) -> bool {
        self.routes
            .iter()
            .any(|route| route.connects(first_location_id, second_location_id))
    }

    /// Convert legacy adjacency lists into deterministic route records, then
    /// rebuild the transient adjacency cache. No entity IDs are allocated.
    pub fn migrate_legacy_exits(&mut self) {
        use std::collections::{BTreeMap, HashSet};

        let location_ids = self
            .locations
            .iter()
            .map(|location| location.id)
            .collect::<HashSet<_>>();
        let mut pairs = BTreeMap::<(EntityId, EntityId), (bool, bool)>::new();

        for location in &self.locations {
            for target_id in &location.exits {
                if *target_id == location.id || !location_ids.contains(target_id) {
                    continue;
                }
                let (a, b, from_a) = if location.id < *target_id {
                    (location.id, *target_id, true)
                } else {
                    (*target_id, location.id, false)
                };
                let entry = pairs.entry((a, b)).or_insert((false, false));
                if from_a {
                    entry.0 = true;
                } else {
                    entry.1 = true;
                }
            }
        }

        for ((a, b), (legacy_a_to_b, legacy_b_to_a)) in pairs {
            let (mut existing_a_to_b, mut existing_b_to_a) = (false, false);
            for route in self.routes.iter().filter(|route| route.connects(a, b)) {
                existing_a_to_b |= route.allows_travel(a, b);
                existing_b_to_a |= route.allows_travel(b, a);
            }

            let missing_a_to_b = legacy_a_to_b && !existing_a_to_b;
            let missing_b_to_a = legacy_b_to_a && !existing_b_to_a;
            if !missing_a_to_b && !missing_b_to_a {
                continue;
            }

            let base_route_id = format!("route.compat.legacy.{a}.{b}");
            if let Some(route) = self
                .routes
                .iter_mut()
                .find(|route| route.id == base_route_id && route.connects(a, b))
            {
                let a_to_b = route.allows_travel(a, b) || missing_a_to_b;
                let b_to_a = route.allows_travel(b, a) || missing_b_to_a;
                route.direction = match (a_to_b, b_to_a) {
                    (true, true) => RouteDirection::Bidirectional,
                    (true, false) => RouteDirection::AtoB,
                    (false, true) => RouteDirection::BtoA,
                    (false, false) => continue,
                };
                continue;
            }

            let route_id = if self.routes.iter().any(|route| route.id == base_route_id) {
                let mut suffix = 1_u32;
                loop {
                    let candidate = format!("{base_route_id}.{suffix}");
                    if self.routes.iter().all(|route| route.id != candidate) {
                        break candidate;
                    }
                    suffix += 1;
                }
            } else {
                base_route_id
            };
            let direction = match (missing_a_to_b, missing_b_to_a) {
                (true, true) => RouteDirection::Bidirectional,
                (true, false) => RouteDirection::AtoB,
                (false, true) => RouteDirection::BtoA,
                (false, false) => continue,
            };
            self.routes.push(WorldRoute::new(route_id, a, b, direction));
        }
        self.routes.sort_by(|left, right| left.id.cmp(&right.id));
        self.rebuild_location_exit_cache();
    }

    /// Recreate the transient outgoing-neighbor view from route directionality.
    /// This is not saved; persistent connectivity lives only in world routes.
    pub fn rebuild_location_exit_cache(&mut self) {
        for location in &mut self.locations {
            location.exits.clear();
        }
        for route in &self.routes {
            let a_id = route.location_a_id;
            let b_id = route.location_b_id;
            if route.allows_travel(a_id, b_id) {
                if let Some(location) = self.locations.iter_mut().find(|item| item.id == a_id) {
                    location.exits.push(b_id);
                }
            }
            if route.allows_travel(b_id, a_id) {
                if let Some(location) = self.locations.iter_mut().find(|item| item.id == b_id) {
                    location.exits.push(a_id);
                }
            }
        }
        for location in &mut self.locations {
            location.exits.sort_unstable();
            location.exits.dedup();
        }
    }

    pub fn location_by_id(&self, id: EntityId) -> Option<&Location> {
        self.locations.iter().find(|location| location.id == id)
    }

    pub fn location_by_id_mut(&mut self, id: EntityId) -> Option<&mut Location> {
        self.locations.iter_mut().find(|location| location.id == id)
    }

    pub fn location_by_name(&self, name: &str) -> Option<&Location> {
        self.locations.iter().find(|location| location.name == name)
    }

    pub fn location_by_name_mut(&mut self, name: &str) -> Option<&mut Location> {
        self.locations
            .iter_mut()
            .find(|location| location.name == name)
    }

    pub fn location_is_dangerous(&self, id: EntityId) -> bool {
        self.location_by_id(id)
            .map(|location| location.dangerous)
            .unwrap_or(false)
    }

    pub fn record_history(&mut self, turn: u32, text: impl Into<String>) {
        let entry = HistoryEntry {
            id: self.allocate_id(),
            turn,
            text: text.into(),
            entry_type: HistoryEntryType::Narrative,
            event_id: None,
            location_name: None,
            outcome: None,
        };
        self.history.push(entry);
        self.trim_history();
    }

    pub fn record_event_history(
        &mut self,
        turn: u32,
        event_id: impl Into<String>,
        location_name: impl Into<String>,
        outcome: impl Into<String>,
    ) {
        let event_id = event_id.into();
        let location_name = location_name.into();
        let outcome = outcome.into();
        let text = outcome.clone();
        let entry = HistoryEntry {
            id: self.allocate_id(),
            turn,
            text,
            entry_type: HistoryEntryType::Event,
            event_id: Some(event_id),
            location_name: Some(location_name),
            outcome: Some(outcome),
        };
        self.history.push(entry);
        self.trim_history();
    }

    fn trim_history(&mut self) {
        if self.history.len() > MAX_HISTORY_ENTRIES {
            let excess = self.history.len() - MAX_HISTORY_ENTRIES;
            self.history.drain(0..excess);
        }
    }

    pub fn spawn_character(&mut self, name: String, title: String) -> Character {
        let location_id = self.locations.first().map(|loc| loc.id).unwrap_or(0);
        let character_id = self.allocate_id();
        Character::new(character_id, name, title, location_id)
    }
}

impl Character {
    pub fn new(id: EntityId, name: String, title: String, location_id: EntityId) -> Self {
        Self {
            id,
            name,
            title,
            appearance_parts: Vec::new(),
            known_evidence_ids: Vec::new(),
            tracked_quest_id: None,
            hp: 10,
            max_hp: 10,
            location_id,
            inventory: Vec::new(),
            alive: true,
            turn: 0,
            experience: 0,
            level: 1,
            attributes: Attributes {
                might: 1,
                insight: 1,
                endurance: 1,
            },
            conditions: Vec::new(),
            notes: vec!["Born into ash, with no past worth keeping.".to_string()],
            exploration_session: None,
            exploration_position: None,
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} the {}", self.name, self.title)
    }

    pub fn knows_evidence(&self, evidence_id: &str) -> bool {
        self.known_evidence_ids
            .iter()
            .any(|known| known == evidence_id)
    }

    /// Record one stable evidence identity for this character only.
    pub fn record_evidence(&mut self, evidence_id: impl Into<String>) -> bool {
        let evidence_id = evidence_id.into();
        if evidence_id.trim().is_empty() || self.knows_evidence(&evidence_id) {
            return false;
        }
        self.known_evidence_ids.push(evidence_id);
        true
    }

    pub fn effective_might(&self) -> i32 {
        self.attributes.might + condition_penalty(&self.conditions, "Wounded")
    }

    pub fn effective_insight(&self) -> i32 {
        self.attributes.insight + condition_penalty(&self.conditions, "Exhausted")
    }

    pub fn effective_endurance(&self) -> i32 {
        self.attributes.endurance
            + self
                .conditions
                .iter()
                .map(|condition| condition.bonus)
                .sum::<i32>()
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}

impl Faction {
    pub fn new(id: EntityId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            reputation: 0,
            memory: Vec::new(),
        }
    }
}

impl Npc {
    pub fn new(
        id: EntityId,
        name: impl Into<String>,
        title: impl Into<String>,
        location_id: EntityId,
        faction_id: Option<EntityId>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            title: title.into(),
            location_id,
            faction_id,
            memory: Vec::new(),
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} the {}", self.name, self.title)
    }
}

impl Quest {
    #[allow(clippy::too_many_arguments)] // for new()
    pub fn new(
        id: EntityId,
        content_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        target_location_id: EntityId,
        faction_id: EntityId,
        giver_npc_id: EntityId,
        required_item_name: impl Into<String>,
        reward_item_name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            content_id: content_id.into(),
            title: title.into(),
            description: description.into(),
            target_location_id,
            faction_id,
            giver_npc_id,
            required_item_name: required_item_name.into(),
            reward_item_name: reward_item_name.into(),
            objectives: Vec::new(),
            completed_by: None,
            offered: false,
            completed: false,
            reward_claimed: false,
        }
    }
}

fn condition_penalty(conditions: &[Condition], name: &str) -> i32 {
    conditions
        .iter()
        .filter(|condition| condition.name == name)
        .map(|condition| condition.penalty)
        .sum()
}

fn new_world_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

pub fn create_new_state(
    world_name: &str,
    mode: WorldMode,
    character_name: String,
    title: String,
) -> GameState {
    let content = load_campaign_content();
    let config = crate::procedural::WorldGenerationConfig::default();
    let seed = new_world_seed();
    let mut world = crate::procedural::generate_world(world_name, seed, config);
    world.mode = mode;
    world.generation = Some(WorldGenerationMetadata {
        seed,
        region_count: config.region_count,
        location_count: config.location_count,
        extra_edges: config.extra_edges,
        geography_ruleset_version: GEOGRAPHY_RULESET_VERSION,
    });
    crate::procedural::place_authored_content(&mut world, &content);
    world.record_history(0, "A new world stirs beneath ash and ruin.");
    let mut character = world.spawn_character(character_name, title);
    character.appearance_parts =
        crate::character_appearance::default_character_appearance_ids(&content.character_parts);
    world.record_history(
        0,
        format!("{} entered the world.", character.display_name()),
    );
    GameState {
        world,
        character,
        threat: ThreatState::default(),
        corpses: Vec::new(),
        factions: Vec::new(),
        npcs: Vec::new(),
        quests: Vec::new(),
        last_announced_location_id: None,
        rng_state: crate::rng::DeterministicRng::new(seed ^ 0xA5A5_5A5A_1234_5678).state(),
        campaign_content: Some(content),
    }
}

pub fn create_inherited_state(
    state: &GameState,
    character_name: String,
    title: String,
) -> GameState {
    let mut world = state.world.clone();
    world.mode = WorldMode::Inherited;
    let content = state
        .campaign_content
        .clone()
        .unwrap_or_else(load_campaign_content);
    content.seed_world(&mut world);
    let mut character = world.spawn_character(character_name, title);
    character.appearance_parts = state.character.appearance_parts.clone();
    let turn = world.history.last().map(|entry| entry.turn).unwrap_or(0);
    world.record_history(
        turn,
        format!("{} inherited the world.", character.display_name()),
    );
    let mut inherited_factions = state.factions.clone();
    for faction in &mut inherited_factions {
        faction.reputation = 0;
    }
    GameState {
        world,
        character,
        threat: ThreatState::default(),
        corpses: state.corpses.clone(),
        factions: inherited_factions,
        npcs: state.npcs.clone(),
        quests: Vec::new(),
        last_announced_location_id: None,
        rng_state: state.rng_state,
        campaign_content: Some(content),
    }
}

#[cfg(test)]
#[path = "../tests/unit/model.rs"]
mod tests;
