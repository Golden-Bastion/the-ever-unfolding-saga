//! Presentation-facing data models shared by frontend renderers.
//!
//! These types intentionally contain no ratatui/crossterm types and no gameplay
//! behavior. Gameplay and screen modules can translate authoritative game state
//! into these models, while a frontend is responsible for rendering them.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CharacterView {
    pub name: String,
    pub title: String,
    pub hp: i32,
    pub max_hp: i32,
}

impl CharacterView {
    pub(crate) fn display_name(&self) -> String {
        if self.title.is_empty() {
            self.name.clone()
        } else {
            format!("{} the {}", self.name, self.title)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AttributesView {
    pub might: i32,
    pub insight: i32,
    pub endurance: i32,
    pub effective_might: i32,
    pub effective_insight: i32,
    pub effective_endurance: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConditionView {
    pub name: String,
    pub remaining: u32,
    pub penalty: i32,
    pub bonus: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CharacterSheetView {
    pub character: CharacterView,
    pub level: u32,
    pub experience: u32,
    pub next_level_experience: u32,
    pub attributes: AttributesView,
    pub conditions: Vec<ConditionView>,
    pub factions: Vec<FactionView>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ItemView {
    pub id: u64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LocationView {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub region_name: String,
    pub dangerous: bool,
    pub scene_image: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ThreatView {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum HistoryEntryViewType {
    #[default]
    Narrative,
    Event,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct HistoryEntryView {
    pub day: u32,
    pub entry_type: HistoryEntryViewType,
    pub text: String,
    pub event_id: Option<String>,
    pub location_name: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FactionView {
    pub name: String,
    pub reputation: i32,
    pub memories: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NpcView {
    pub name: String,
    pub title: String,
    pub faction_name: Option<String>,
}

impl NpcView {
    pub(crate) fn display_name(&self) -> String {
        if self.title.is_empty() {
            self.name.clone()
        } else {
            format!("{} the {}", self.name, self.title)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestObjectiveView {
    pub label: String,
    pub progress: u32,
    pub required: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestView {
    pub title: String,
    pub description: String,
    pub objectives: Vec<QuestObjectiveView>,
    pub status: String,
    pub completed: bool,
    pub reward_claimed: bool,
    pub reward_item_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatantView {
    pub name: String,
    pub current_hp: i32,
    pub max_hp: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatView {
    pub character: CharacterView,
    pub player_condition: Option<String>,
    pub enemy: CombatantView,
    pub enemy_power: i32,
    pub location_name: String,
    pub turn: u32,
    pub events: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatResultView {
    pub combat: CombatView,
    pub result_title: String,
    pub result_note: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorldView {
    pub world_name: String,
    pub time: String,
    pub time_points: u32,
    pub day: u32,
    pub character: CharacterView,
    pub location: Option<LocationView>,
    pub art: Option<String>,
    pub atmosphere: Option<String>,
    pub conditions: Vec<ConditionView>,
    pub threat: Option<ThreatView>,
    pub history: Vec<HistoryEntryView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct HistoryView {
    pub world_name: String,
    pub time: String,
    pub character: CharacterView,
    pub entries: Vec<HistoryEntryView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NavigationView {
    pub current_location: Option<LocationView>,
    pub destinations: Vec<LocationView>,
    pub art: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TalkView {
    pub npcs: Vec<NpcView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CharacterPortraitLayerView {
    pub id: String,
    pub slot: String,
    pub asset: String,
    pub layer: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CharacterPortraitView {
    pub layers: Vec<CharacterPortraitLayerView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConversationView {
    pub player: CharacterView,
    pub player_portrait: CharacterPortraitView,
    pub npc: NpcView,
    pub npc_portrait: CharacterPortraitView,
    /// Legacy ASCII portrait retained as a temporary fallback.
    pub portrait: Option<String>,
    pub memory: Option<String>,
    pub options: Vec<String>,
    pub available: bool,
    pub unavailable_message: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InventoryView {
    pub character: CharacterView,
    pub items: Vec<ItemView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InventoryDetailView {
    pub item: ItemView,
    pub position: usize,
    pub total: usize,
    pub art: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestLogView {
    pub character: CharacterView,
    pub quests: Vec<QuestView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MeditationTargetView {
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MeditationView {
    pub character: CharacterView,
    pub current_time: String,
    pub safe_to_meditate: bool,
    pub unavailable_message: Option<String>,
    pub targets: Vec<MeditationTargetView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MeditationResultView {
    pub ending_time: String,
    pub portions: u32,
    pub hp_recovered: i32,
    pub exhausted_removed: bool,
    pub well_rested_applied: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ScreenView {
    pub title: String,
    pub subtitle: Option<String>,
    pub art: Option<String>,
    pub body: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DeathView {
    pub screen: ScreenView,
    pub character: CharacterView,
    pub location_name: String,
    pub turn: u32,
    pub deeds: Vec<String>,
    pub faction_standing: Vec<FactionView>,
    pub dropped_items: Vec<ItemView>,
    pub memory_note: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemainsEntryView {
    pub id: u64,
    pub label: String,
    pub former_name: String,
    pub former_title: String,
    pub scavenged: bool,
    pub items: Vec<ItemView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemainsView {
    pub location_name: String,
    pub remains: Vec<RemainsEntryView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemainsResultView {
    pub location_name: String,
    pub former_name: String,
    pub former_title: String,
    pub items: Vec<ItemView>,
    pub hidden_item: Option<ItemView>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConsoleCandidateView {
    pub value: String,
    pub hint: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ConsoleScrollView {
    #[default]
    Follow,
    Offset(usize),
    Home,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConsoleView {
    pub output: Vec<String>,
    pub input: String,
    pub scroll: ConsoleScrollView,
    pub completion_scroll: usize,
    pub candidates: Vec<ConsoleCandidateView>,
    pub selected: usize,
    pub autocomplete: bool,
}

#[cfg(test)]
#[path = "../tests/unit/presentation.rs"]
mod tests;
