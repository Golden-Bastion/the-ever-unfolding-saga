use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignContent {
    pub version: u32,
    pub world: WorldContent,
    #[serde(default)]
    pub factions: Vec<FactionContent>,
    #[serde(default)]
    pub npcs: Vec<NpcContent>,
    /// Reusable portrait layers supplied by campaign content and mods.
    #[serde(default)]
    pub character_parts: Vec<CharacterPartContent>,
    #[serde(default)]
    pub quests: Vec<QuestContent>,
    #[serde(default)]
    pub encounters: Vec<EncounterContent>,
    #[serde(default)]
    pub atmospheres: Vec<LocationAtmosphere>,
    #[serde(default)]
    pub item_visuals: Vec<ItemVisualContent>,
    #[serde(default)]
    pub events: Vec<EventContent>,
    #[serde(skip)]
    pub loaded_mods: Vec<ModManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldContent {
    pub region: RegionContent,
    pub locations: Vec<LocationContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionContent {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationContent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dangerous: bool,
    #[serde(default)]
    pub exits: Vec<String>,
    #[serde(default)]
    pub scene_art: Option<String>,
    /// Runtime-loaded image reference, resolved from data/assets or a mod asset directory.
    #[serde(default)]
    pub scene_image: Option<String>,
    /// Optional built-in local-map layout identifier (for example, `shrine_chamber`).
    #[serde(default)]
    pub spatial_layout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionContent {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcContent {
    pub id: String,
    pub name: String,
    pub title: String,
    pub location_name: String,
    #[serde(default)]
    pub faction_name: Option<String>,
    #[serde(default)]
    pub memory: Vec<String>,
    #[serde(default)]
    pub portrait: Option<String>,
    /// Optional profile tags used to constrain weighted portrait-part selection.
    #[serde(default)]
    pub appearance_tags: Vec<String>,
}

/// A reusable transparent illustration layer selected by the portrait generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPartContent {
    pub id: String,
    pub slot: String,
    /// PNG asset path resolved through the mod-aware asset loader.
    pub asset: String,
    #[serde(default = "default_character_part_weight")]
    pub weight: u32,
    #[serde(default)]
    pub layer: i32,
    #[serde(default)]
    pub requires_tags: Vec<String>,
    #[serde(default)]
    pub excludes_tags: Vec<String>,
}

fn default_character_part_weight() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestContent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub location_name: String,
    pub faction_name: String,
    pub giver_npc_name: String,
    pub required_item_name: String,
    pub reward_item_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterContent {
    pub location_name: String,
    pub enemy_name: String,
    pub enemy_hp: i32,
    pub enemy_power: i32,
    pub trophy_item_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationAtmosphere {
    pub location_name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemVisualContent {
    pub item_name: String,
    pub art: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventConditionContent {
    #[serde(default)]
    pub night: Option<bool>,
    #[serde(default)]
    pub dangerous: Option<bool>,
    #[serde(default)]
    pub min_day: Option<u32>,
    #[serde(default)]
    pub max_day: Option<u32>,
    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default)]
    pub prior_event_id: Option<String>,
    #[serde(default)]
    pub faction_name: Option<String>,
    #[serde(default)]
    pub min_reputation: Option<i32>,
    #[serde(default)]
    pub max_reputation: Option<i32>,
    #[serde(default)]
    pub required_item_name: Option<String>,
    #[serde(default)]
    pub required_condition_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventEffectContent {
    Message {
        text: String,
    },
    History {
        text: String,
    },
    Pause,
    Heal {
        amount: i32,
    },
    Damage {
        amount: i32,
    },
    AddCondition {
        name: String,
        remaining: u32,
        #[serde(default)]
        penalty: i32,
        #[serde(default)]
        bonus: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContent {
    pub id: String,
    pub trigger: String,
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub chance_percent: Option<u8>,
    #[serde(default)]
    pub cooldown_turns: Option<u32>,
    #[serde(default)]
    pub conditions: Option<EventConditionContent>,
    pub effects: Vec<EventEffectContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub content_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContentLoadReport {
    pub content: CampaignContent,
    pub loaded_mods: Vec<ModManifest>,
    pub warnings: Vec<String>,
}

impl CampaignContent {
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.version != 1 {
            issues.push(format!(
                "content version {} is not recognized",
                self.version
            ));
        }

        validate_unique_ids(
            "location",
            self.world.locations.iter().map(|entry| entry.id.as_str()),
            &mut issues,
        );
        validate_unique_ids(
            "faction",
            self.factions.iter().map(|entry| entry.id.as_str()),
            &mut issues,
        );
        validate_unique_ids(
            "npc",
            self.npcs.iter().map(|entry| entry.id.as_str()),
            &mut issues,
        );
        validate_unique_ids(
            "character part",
            self.character_parts.iter().map(|entry| entry.id.as_str()),
            &mut issues,
        );
        for part in &self.character_parts {
            if part.slot.trim().is_empty() {
                issues.push(format!("character part {} has an empty slot", part.id));
            }
            if part.weight == 0 {
                issues.push(format!("character part {} has zero weight", part.id));
            }
            if !is_valid_character_part_asset(&part.asset) {
                issues.push(format!(
                    "character part {} has an invalid PNG asset reference",
                    part.id
                ));
            }
        }
        validate_unique_ids(
            "quest",
            self.quests.iter().map(|entry| entry.id.as_str()),
            &mut issues,
        );
        validate_unique_ids(
            "item visual",
            self.item_visuals
                .iter()
                .map(|entry| entry.item_name.as_str()),
            &mut issues,
        );
        validate_unique_ids(
            "event",
            self.events.iter().map(|entry| entry.id.as_str()),
            &mut issues,
        );

        let location_names: HashSet<&str> = self
            .world
            .locations
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        let faction_names: HashSet<&str> = self
            .factions
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        let npc_names: HashSet<&str> = self.npcs.iter().map(|entry| entry.name.as_str()).collect();

        for location in &self.world.locations {
            for exit in &location.exits {
                if !self.world.locations.iter().any(|other| other.id == *exit) {
                    issues.push(format!(
                        "location {} exits to unknown location id {}",
                        location.id, exit
                    ));
                }
            }
        }

        for npc in &self.npcs {
            if !location_names.contains(npc.location_name.as_str()) {
                issues.push(format!(
                    "npc {} uses unknown location {}",
                    npc.id, npc.location_name
                ));
            }
            if let Some(faction_name) = npc.faction_name.as_deref() {
                if !faction_names.contains(faction_name) {
                    issues.push(format!(
                        "npc {} uses unknown faction {}",
                        npc.id, faction_name
                    ));
                }
            }
        }

        for quest in &self.quests {
            if !location_names.contains(quest.location_name.as_str()) {
                issues.push(format!(
                    "quest {} uses unknown location {}",
                    quest.id, quest.location_name
                ));
            }
            if !faction_names.contains(quest.faction_name.as_str()) {
                issues.push(format!(
                    "quest {} uses unknown faction {}",
                    quest.id, quest.faction_name
                ));
            }
            if !npc_names.contains(quest.giver_npc_name.as_str()) {
                issues.push(format!(
                    "quest {} uses unknown giver {}",
                    quest.id, quest.giver_npc_name
                ));
            }
        }

        for encounter in &self.encounters {
            if !location_names.contains(encounter.location_name.as_str()) {
                issues.push(format!(
                    "encounter {} uses unknown location {}",
                    encounter.enemy_name, encounter.location_name
                ));
            }
        }

        for event in &self.events {
            if event.id.trim().is_empty() {
                issues.push("event has an empty id".to_string());
            }
            if event.trigger.trim().is_empty() {
                issues.push(format!("event {} has an empty trigger", event.id));
            }
            if event.weight == 0 {
                issues.push(format!("event {} has zero weight", event.id));
            }
            if let Some(chance) = event.chance_percent {
                if chance > 100 {
                    issues.push(format!("event {} has invalid chance {}", event.id, chance));
                }
            }
            if event.effects.is_empty() {
                issues.push(format!("event {} has no effects", event.id));
            }
            if let Some(conditions) = &event.conditions {
                for location in &conditions.locations {
                    if !location_names.contains(location.as_str()) {
                        issues.push(format!(
                            "event {} uses unknown location {}",
                            event.id, location
                        ));
                    }
                }
                if let (Some(min_day), Some(max_day)) = (conditions.min_day, conditions.max_day) {
                    if min_day > max_day {
                        issues.push(format!(
                            "event {} has min_day greater than max_day",
                            event.id
                        ));
                    }
                }
                if let Some(faction_name) = conditions.faction_name.as_deref() {
                    if !faction_names.contains(faction_name) {
                        issues.push(format!(
                            "event {} uses unknown faction {}",
                            event.id, faction_name
                        ));
                    }
                }
                if (conditions.min_reputation.is_some() || conditions.max_reputation.is_some())
                    && conditions.faction_name.as_deref().is_none()
                {
                    issues.push(format!(
                        "event {} has a reputation condition without a faction_name",
                        event.id
                    ));
                }
                if let (Some(min_reputation), Some(max_reputation)) =
                    (conditions.min_reputation, conditions.max_reputation)
                {
                    if min_reputation > max_reputation {
                        issues.push(format!(
                            "event {} has min_reputation greater than max_reputation",
                            event.id
                        ));
                    }
                }
                if conditions
                    .required_item_name
                    .as_deref()
                    .map(|name| name.trim().is_empty())
                    .unwrap_or(false)
                {
                    issues.push(format!(
                        "event {} has an empty required_item_name",
                        event.id
                    ));
                }
                if conditions
                    .required_condition_name
                    .as_deref()
                    .map(|name| name.trim().is_empty())
                    .unwrap_or(false)
                {
                    issues.push(format!(
                        "event {} has an empty required_condition_name",
                        event.id
                    ));
                }
            }
        }

        for atmosphere in &self.atmospheres {
            if !location_names.contains(atmosphere.location_name.as_str()) {
                issues.push(format!(
                    "atmosphere uses unknown location {}",
                    atmosphere.location_name
                ));
            }
        }

        issues
    }

    pub fn encounter_for(&self, location_name: &str) -> Option<&EncounterContent> {
        self.encounters
            .iter()
            .find(|entry| entry.location_name == location_name)
    }

    pub fn location_art_for(&self, location_name: &str) -> Option<&str> {
        self.world
            .locations
            .iter()
            .find(|entry| entry.name == location_name)
            .and_then(|entry| entry.scene_art.as_deref())
    }

    pub fn location_scene_image_for(&self, location_name: &str) -> Option<&str> {
        self.world
            .locations
            .iter()
            .find(|entry| entry.name == location_name)
            .and_then(|entry| entry.scene_image.as_deref())
    }

    pub fn portrait_for(&self, npc_name: &str) -> Option<&str> {
        self.npcs
            .iter()
            .find(|entry| entry.name == npc_name)
            .and_then(|entry| entry.portrait.as_deref())
    }

    pub fn appearance_tags_for(&self, npc_name: &str) -> &[String] {
        self.npcs
            .iter()
            .find(|entry| entry.name == npc_name)
            .map(|entry| entry.appearance_tags.as_slice())
            .unwrap_or(&[])
    }

    pub fn item_art_for(&self, item_name: &str) -> Option<&str> {
        self.item_visuals
            .iter()
            .find(|entry| entry.item_name == item_name)
            .map(|entry| entry.art.as_str())
    }
}

fn validate_unique_ids<'a, I>(kind: &str, ids: I, issues: &mut Vec<String>)
where
    I: Iterator<Item = &'a str>,
{
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            issues.push(format!("duplicate {} id {}", kind, id));
        }
    }
}

fn is_valid_character_part_asset(reference: &str) -> bool {
    !reference.trim().is_empty()
        && reference.ends_with(".png")
        && !reference.starts_with('/')
        && !reference.contains('\\')
        && reference
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn default_true() -> bool {
    true
}
