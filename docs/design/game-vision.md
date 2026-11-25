# Game Vision: Exploration, Self-Directed Lives, and Inherited History

> Design record migrated from [GitHub issue #364](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/364). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

## Status

Long-term design vision and planning issue.

This issue defines the intended player experience and the principles that should guide future world generation, gameplay systems, content development, presentation, and persistence.

It is not a commitment to implement every described capability immediately. Individual systems require their own design work, milestones, and implementation tasks.

## Vision

The Ever-Unfolding Saga is a dark-fantasy open-world RPG about exploration, self-directed lives, and lasting consequences.

The player enters a world that extends beyond the known frontier, chooses how to live, and leaves consequences that can outlast a single character. When a character dies, another can continue in the same world and inherit its history.

The experience should combine an evocative wilderness, dangerous encounters, meaningful relationships, consequential decisions, and freedom to pursue different lives. Exploration and combat are both central pleasures: players discover what the world contains, then decide how to engage with it, including through direct and sometimes brutal violence. The world should reward curiosity while allowing player actions—constructive, destructive, or otherwise—to shape a persistent history.

The objective is not to reproduce any particular existing game, but to develop a coherent experience from these design aspirations.

## Core design pillars

### 1. Expansive procedural exploration

The world should be capable of extending beyond its initially generated or explored areas. Travelling farther should reveal new regions and places without requiring the entire world to be handcrafted in advance.

Procedural generation should provide more than terrain or randomly named locations. Generated areas should offer contextually appropriate opportunities for exploration, discovery, hunting, gathering, encounters, resources, settlement, and other activities supported by the game.

Generation should support different player interests. A location may be valuable to one character because of its wildlife, to another because of its resources, and to another because of its inhabitants or discoveries.

Not every area needs to contain an encounter or reward. Wilderness, distance, atmosphere, and quiet exploration remain valid parts of the experience. The objective is to make continued exploration worthwhile, not to maximize the density of points of interest.

World expansion, content generation, and simulation are distinct concerns. A large procedural world does not require every entity or ecosystem to be continuously simulated in detail.

Generation should remain deterministic where practical, use stable world and region identities, and preserve established world state. Newly generated areas should fit into the same persistent world rather than behaving as unrelated campaigns.

### 2. Freedom to choose a life

The game should not assume that every character follows the same profession, progression path, or narrative.

Depending on the systems and content available, characters may pursue activities such as:

- Hunting, gathering, and wilderness exploration.
- Crafting, trade, wealth, and other forms of economic activity.
- Relationships, family, settlement, and community.
- Leadership, faction influence, warfare, and political power.
- Religious or ideological movements, organizations, and other collective endeavors.

These are long-term directions, not a claim that every listed activity is already implemented.

Activities should arise from the world and the player's circumstances rather than all being available as disconnected menu options. Access to opportunities may depend on location, knowledge, resources, relationships, reputation, capabilities, and previous decisions.

Different pursuits should be capable of becoming meaningful long-term goals. The game should not require players to complete a predetermined main storyline before they can consider their character's life successful.

### 3. A world that remembers

Character death should be a transition in the world's history rather than an automatic erasure of the player's progress.

The world should preserve important consequences of previous lives. Depending on the relevant systems, these may include discoveries, deeds, property, settlements, relationships, institutions, political changes, conflicts, and other lasting effects.

A subsequent character may inherit the world, encounter the consequences of earlier lives, and continue shaping its future.

Inheritance should preserve the distinction between character-specific state and world-persistent state. A new character should not automatically inherit every ability, relationship, active task, or temporary condition of the previous character.

The precise inheritance rules should be defined alongside the systems they affect.

### 4. Emergent stories and meaningful consequences

The interaction of world state, procedural generation, character goals, relationships, factions, events, and player decisions should create situations that are not limited to a fixed sequence of authored quests.

The world should be able to produce opportunities and consequences from its existing context. Generated situations should be coherent enough to invite meaningful decisions rather than merely providing interchangeable tasks. Player-initiated actions, including violence unrelated to an authored quest, should also be able to create consequential situations. The world may judge a character through fear, admiration, hatred, respect, distrust, or competing interpretations, according to what individuals and groups know or believe.

Consequences should emerge through the world's people and institutions rather than through a universal morality score or a scripted escalation track. Witnesses, survivors, evidence, reports, rumours, investigations, local laws, faction interests, available personnel, and the ability to organize a response all shape what happens. Authorities and other actors must act on information they could plausibly acquire; the simulation should not reveal unseen actions to them automatically. Responses may range from avoidance or local hostility to organized pursuit or a large military operation when the scale of the event and the responding group's actual capabilities justify it. Pursuit must remain part of the world: forces need information, time, leadership, movement, and supplies, and the player may evade, mislead, confront, outlast, or be overtaken by them.

The same event need not produce one universal judgement. Different individuals, settlements, factions, and regions may interpret it differently, and their views may be mistaken, biased, delayed, or changed by later events. A character may be feared by one community, celebrated by its enemies, and unknown elsewhere. Reputation should reflect what each observer or group knows and believes, not omniscient access to the complete event history.

The intended experience allows violence to be enjoyable as moment-to-moment gameplay without making it consequence-free in the persistent world. Combat should be satisfying in itself; consequences should arise from coherent simulation, not from moralizing restrictions that arbitrarily prevent plausible actions. A player may choose to save and reload around an experiment or destructive spree. That is player-controlled use of saves, not an in-world action or a reason for the simulation to judge the player outside the state of the loaded timeline.

Simulation complexity should be introduced where it contributes to the experience. A fully simulated ecosystem or society is not a prerequisite for every procedural feature.

Handcrafted content remains important for distinctive characters, unusual encounters, authored narrative arcs, and memorable decisions.

### 5. Authored content within a generated world

The base game and mods provide authored content that gives the world specificity and narrative depth.

This includes distinctive characters, locations, factions, quests, events, dialogue, encounters, and other content that benefits from deliberate writing and design.

Procedural systems provide breadth and variation; authored content provides carefully constructed experiences. These approaches should complement one another rather than compete.

Where practical, content should remain data-driven and extensible through the existing mod architecture. Authored content should be able to interact with generated world state and supported gameplay systems without requiring every new story to be implemented as a special case in core logic.

## Relationship between generation, simulation, and authored content

These are separate but complementary layers:

- **World generation** establishes geography, regions, locations, and the initial distribution of relevant world features. It should support expansion beyond the known frontier.
- **Content generation** populates the world with contextually appropriate entities, resources, points of interest, and opportunities.
- **World simulation** governs how supported entities and systems change over time and respond to player actions.
- **Authored content** introduces deliberately designed characters, situations, narrative arcs, and exceptional events.
- **Persistence and inheritance** preserve the appropriate world changes and make them relevant across multiple character lives.

The implementation should avoid requiring every layer to be equally complex everywhere. Systems should share data and context where useful, while retaining clear ownership and responsibilities.

## Architectural principles

- Preserve the separation between authoritative gameplay state and presentation.
- Extend existing world, time, event, quest, character, faction, and persistence systems where appropriate rather than creating unrelated parallel systems.
- Keep procedural generation deterministic where practical and avoid rerolling established world state.
- Distinguish character-specific state from world-persistent state.
- Keep authored content data-driven and mod-extensible wherever practical.
- Preserve save compatibility as systems evolve.
- Introduce new systems incrementally, with focused tests and explicit acceptance criteria.
- Ensure the world and its core gameplay rules remain independent of any particular visual presentation or input method where practical.

## Scope and sequencing

This vision is intentionally broader than the current implementation.

The immediate objective is not to implement every profession, simulate every entity, or generate an effectively infinite world in one step.

Future planning should identify the next highest-value capability, its dependencies, the relevant architectural decisions, and a testable definition of completion before implementation begins.
