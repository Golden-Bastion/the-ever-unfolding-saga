# Procedural Content Generation: Regions, Locations, and Opportunities

> Design record migrated from [GitHub issue #366](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/366). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

The existing generators and content pipeline are the foundation to extend, not replace. Implementation should be staged and coordinated with #365.

## Goal

Make generated regions and locations feel as though they belong to a world, rather than being names and random encounters attached to empty maps. Procedural generation should create contextually appropriate opportunities for different player interests without making every location a checklist of rewards.

**Core rule:** every generated location should have a recognizable identity and a reason to exist, but does not need to contain loot, combat, a quest, or a major revelation. Quiet wilderness and atmosphere remain valid. Lack of identity or purpose is the problem, not the absence of a reward.

## Proposed content-generation architecture

Generation is divided into five complementary layers:

1. **Regional identity:** environment, resources, population patterns, danger, prosperity, and regional character.
2. **Location identity:** what kind of place it is, its role in the region, and how it relates to neighboring locations and routes.
3. **Location content:** landmarks, inhabitants, resources, hazards, discoveries, routes, and interactable features.
4. **Opportunities and relationships:** possible activities, conflicts, quests, events, and consequences that follow from the generated context.
5. **Persistence and evolution:** preserve established content and let supported world state change through explicit gameplay effects.

Creating a region does not require generating all its quests, spawning every inhabitant, simulating every creature, or immediately creating a detailed local terrain map.

## 1. Regional identity

Each region should have a recognizable identity derived from its geography and history. Existing characteristics—theme, climate, resources, prosperity, danger, and population—remain useful.

The regional profile may additionally represent:

- Terrain and environmental conditions.
- Dominant resources and their local uses.
- Settlement patterns and degree of isolation.
- Faction presence and contested interests.
- Distinctive environmental or historical features.
- Connections to neighboring regions and established routes.

Not every property needs to be a numeric statistic; categorical tags and structured relationships may be more appropriate.

Characteristics should reinforce one another. For example, resource-rich highlands could support mining settlements, dangerous mountain routes, and competition over ore or trade. Wet marshlands could emphasize waterways, scarce dry ground, medicinal plants, and difficult travel.

A region is more than a visual palette: its environment and society should influence what can be found there. Regions should be recognizable across their constituent locations.

## 2. Location identity and distribution

Location types should be selected using the regional profile, graph connections, nearby location roles, and the kinds of places already present in the area.

Use **weighted selection plus structural constraints**. Weights determine plausible outcomes; constraints prevent incoherent distributions such as a region with no viable destinations, every location serving the same role, or repeated adjacent sites with identical functions.

Generation should consider:

- Regional suitability.
- The roles of neighboring locations.
- Whether a location provides a necessary travel connection.
- The distribution of settlements, wilderness, ruins, mines, shrines, crossroads, and other archetypes.
- Whether sufficient content exists to make the destination worth visiting.
- Authored locations and their required relationships.

A crossroads connected to several destinations serves a different purpose from an isolated shrine or remote mine. No rigid location-type ratio is required for every region; coherent variation should prevent repetition.

## 3. Give every location an identity, not necessarily a reward

Every generated location should have a recognizable identity and a reason to exist. A visitor should have something to understand, investigate, use, or navigate around, but not necessarily an immediate reward.

A location can have one primary feature and, where appropriate, a small number of secondary features. Examples include:

- **Abandoned mine:** an ore deposit or dangerous excavation, perhaps with collapsed passages or traces of former workers.
- **Isolated settlement:** inhabitants with needs or local circumstances, possibly offering trade, rumors, or faction tensions.
- **Ancient shrine:** a relic, historical clue, or unusual site, perhaps with a hazard or hidden chamber.
- **Wilderness:** wildlife, a natural landmark, or resource, perhaps with tracks or an optional discovery.
- **Mountain crossing:** a route between destinations, perhaps with a difficult passage, abandoned camp, or encounter.

These are design patterns, not templates to repeat verbatim. Quiet wilderness can contribute atmosphere and world continuity. A location should not contain a large empty map simply because it exists in the world graph.

Local maps should reflect the location's identity. The actual number of points of interest, encounters, and secondary features should be tuned through playtesting rather than fixed universally now.

## 4. Reusable building blocks and contextual descriptions

The base game and mods should provide authored building blocks that procedural rules combine under explicit constraints, including:

- Location and environmental archetypes.
- Descriptive fragments and contextual details.
- Resources and resource-specific interactions.
- Landmarks, ruins, and discovery types.
- NPC roles, faction interests, and social relationships.
- Hazard and encounter definitions.
- Opportunity and event templates.
- Local-map features and points of interest.

The existing data-driven content system supports authored locations, factions, NPCs, quests, encounters, atmosphere descriptions, and events. General-purpose procedural template definitions for all the categories above are not yet present and should be introduced deliberately as the categories are designed.

The first implementation should use authored templates and deterministic composition. It does not require a runtime language model or external service.

Descriptions should reflect actual generated situations. If a description mentions a functioning forge, local population, particular resource, or other feature, the world state must support that claim or explicitly treat it as narrative history. Generated descriptions must not promise features that do not exist.

Names and descriptions should not rely on endlessly combining generic adjectives. They should convey what a place is, who used it, what happened there, or what remains, as supported by the location's generated features.

## 5. NPCs and factions belong to the world

Generated factions should have contextual roles: interests, influence, resource dependencies, alliances, rivalries, or reasons to be present in the region.

NPC presence should reflect the purpose and population of each location. Settlements may have service providers, resource owners, information holders, or participants in local disputes. Remote ruins should not automatically receive a conventional population simply because NPCs are expected everywhere.

Generated NPCs should have at least a coherent role and relevant relationships. Important personalities can receive more detailed authored characterization. Background inhabitants do not all require a full biography, complex personal goals, or continuous simulation. Their initial behavioural profiles should follow the shared [NPC behaviour specification](npc-behavior.md): distinguish stable personality tendencies from competence, current motivation, relationships, knowledge, and temporary circumstances. Generate traits from contextual, deterministic inputs rather than unconstrained random combinations; population roles, regional conditions, faction ties, and authored characterization should produce plausible individuals. Persist generated identities and profiles so re-entry, save/load, and inheritance do not reroll them.

Faction relationships should remain coherent across locations. If a faction influences a settlement, controls a route, or competes over resources, related content should reflect those facts rather than rerolling them independently at each site.

This does not require full territorial, economic, or social simulation for every faction. A small set of structured relationships and explicit state changes can support meaningful consequences.

## 6. Opportunities emerge from context, not from a quest quota

Procedural quests should be one possible consequence of a generated situation, not the default output of every region or location.

Possible situations include:

- A resource deposit that makes a difficult journey worthwhile.
- A clue that gives a reason to investigate another location.
- A local dispute that offers a choice of whom to support.
- An encounter that reveals something about a nearby faction.
- A dangerous route that can be cleared, avoided, or exploited.
- An abandoned place whose significance becomes clearer after another discovery.

Some opportunities may be immediately visible; others can emerge after conversation, examination, route discovery, relevant knowledge, or changes in world state.

A generated quest must have coherent objectives and consequences. Visiting a location can be a valid objective when discovery itself matters, but should not be the dominant pattern for every quest.

The current quest model supports a relatively limited set of objective types. Content generation must use supported mechanics or be accompanied by separately designed gameplay capabilities. It must not create quests whose required actions do not exist.

Generation should account for existing opportunities and active player circumstances. It should prevent duplicate identities, near-identical quests across neighboring sites, repeated event placement, and rewards with no practical value. Limits on simultaneously active generated quests should be tuned through testing.

Completing an opportunity can change world state, such as reducing a local danger, altering faction presence, changing route safety, or unlocking a follow-up. Consequences must be explicit gameplay effects rather than inferred from quest text.

## 7. Authored content and mods take precedence

Procedural generation expands the world around deliberately designed content; it must not overwrite that content.

Priority:

1. **Explicit authored content** retains its intended identity, characters, events, and relationships.
2. **Mod-authored additions and overrides** follow the existing stable-ID and merge rules.
3. **Procedural content** fills remaining world structure and responds to authored content already present.
4. **Runtime evolution** changes supported state through explicit gameplay effects while preserving the identity and consequences of established content.

Authored locations remain where their content definitions put them. Generated locations can connect to them, populate the surrounding region, or create related opportunities, but must not silently replace them.

The content system should eventually allow mods to add reusable procedural templates and archetypes as well as authored locations and quests. For example, a mod could introduce a ruin type, resource, or faction concept eligible in appropriate regions. This is optional and must be schema-validated.

The existing content loader and mod-merge behavior remain the foundation; a complete mod-loading redesign is not required.

## 8. Determinism, persistence, provenance, and performance

The world-expansion model from #365 requires generation to be independent of exploration order.

Generated regions, locations, entities, and opportunities should have stable generation identities. Randomness should derive from the world seed, relevant stable keys, and a generation-ruleset version. Independent content categories should use separate deterministic streams or equivalent namespaced seeds so adding a decorative detail does not unexpectedly change NPC populations or quest outcomes.

When a region is created, its world-level content should be materialized once and attached to persistent state. Re-entering the region, loading a save, or inheriting the world must not regenerate its inhabitants, names, relationships, or opportunities.

Detailed local-map geometry can be generated lazily when Explore is selected, as defined in #365. Meaningful discoveries and changes must be persisted.

The current global bootstrap generation flow must evolve into idempotent, region- and location-scoped generation as frontier expansion is implemented. Creating a region must not repopulate the entire world.

As models evolve, use structured identities or origin metadata to distinguish generated from authored content instead of relying only on free-text markers that later code has to parse.

Performance should scale primarily with the region or location being created or visited. Generating a location should not require rebuilding every faction, NPC, description, and opportunity across the entire world.

## Implementation sequence

1. **Stable generation identity and provenance.** Establish persistent keys and idempotent generation boundaries compatible with #365.
2. **Coherent regional and location profiles.** Refine characteristics and choose location roles using regional context and neighboring locations.
3. **Location content plans.** Define primary features, secondary features, landmarks, resources, and local interactions, guaranteeing identity but not loot.
4. **Contextual populations and factions.** Generate inhabitants and faction presence according to location purpose and regional structure.
5. **Opportunity generation.** Create eligible quests, encounters, and discoveries from actual context, with duplicate prevention and explicit effects.
6. **Spatial exploration integration.** Translate the location plan into a deterministic local map with reachable, meaningful features and route exits.
7. **Mod extensibility and balance.** Add validated template definitions, tune content density and repetition through playtesting, and preserve existing saves and authored campaigns.

These stages should be split into smaller implementation issues when work begins. They do not require replacing the current spatial prototype or implementing comprehensive world simulation first.

## Acceptance criteria

- Generated regions have coherent profiles, and their locations reflect those profiles rather than being independent random selections.
- Every generated location has a recognizable identity and reason to exist, without requiring rewards or encounters everywhere.
- Descriptions and visible features agree with the generated world state.
- Factions, NPCs, and opportunities relate coherently to regional context and nearby locations.
- Generated quests use supported mechanics and have explicit objectives, completion effects, and persistence.
- Generation is idempotent and deterministic under stable inputs, independent of the order in which regions are visited.
- Established content does not reroll on re-entry, save/load, or character inheritance.
- Authored campaign content and compatible mod content retain precedence and stable references.
- Each region or location can be generated without rebuilding the entire generated world.
- Generated local maps can express location identity and meaningful features, with valid route connections.
- Tests cover repeatability, content-reference integrity, authored-content preservation, save compatibility, duplicate prevention, and bounded generation cost.

## Details intentionally left open

Exact population counts, a fixed number of points of interest per location, the ideal percentage of dangerous regions, and a universal quota of generated quests are balancing decisions and should not be locked in before playtesting.

A complete economy, ecosystem simulation, independently goal-driven NPC society, procedural narrative generator, and complex long-term faction simulation are not prerequisites. Each may justify a separate design if it materially improves the experience.

The content generator's responsibility is to make a varied, coherent, persistent world that offers meaningful possibilities. Dedicated systems determine how those possibilities evolve and how the player acts on them.

## Related design direction

- #364
- #365
- #353
