# Spatial Gameplay: Exploration, Investigation, and Action Combat

> Design record migrated from [GitHub issue #228](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/228). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

Long-term design umbrella for the broader hybrid spatial-exploration, investigation, and action-combat vision.

The first bounded exploration milestone is tracked separately in #353, with three implementation tasks: #354, #355, and #356.

Those tasks cover only the initial playable exploration slice, not the full vision in this document. The design umbrella issue #228 was closed after its settled direction was migrated here; remaining implementation work—persistent roaming entities, awareness/pursuit, encounter lifecycle, and investigation/clue progression beyond the initial authored chain—is tracked in the root roadmap. Real-time spatial combat has its own detailed design in #252.

## Vision

Build a character-focused RPG in which spatial exploration and future action combat are graphics-first experiences, supported by a persistent simulation and existing narrative/record systems. Extend the current game deliberately: preserve useful world rules, contextual interactions, dialogue, quests, events, NPCs, time, and persistence instead of replacing the whole structure or maintaining two competing games.

The existing world model and gameplay rules remain authoritative. Text-based dialogue, contextual feedback, history, and record screens continue to matter; they support the spatial experience rather than defining the game's overall identity.

## Core interaction model

Spatial gameplay is the normal player-facing context for locations, including settlements and safe areas. There is no separate dashboard or location-level `Explore` action: the player inhabits the current location through its spatial map and moves between locations through physical passages or eligible known-route fast travel.

Ordinary world interactions are contextual and proximity-driven. Nearby eligible NPCs expose Talk through the spatial interaction wheel; searching remains, examining points of interest, and entering passages use the same in-world interaction model. Trade, when implemented, should be added to this contextual NPC interaction flow rather than as a detached dashboard or location-level button.

Meditation is a deliberate character activity exposed through the spatial pause menu, and remains subject to the authoritative safe-to-meditate rules. Other character records and system functions remain in their dedicated screens and menus. This distinction is about where an interaction belongs, not whether the current location has a spatial map.

Investigation is contextual to the world state. An eligible active threat exposes the existing Investigate action in spatial gameplay, which enters the authoritative combat flow; clue- and environment-based investigation can build on spatial examination and discovery rather than a separate Explore mode.

## Dynamic world

The spatial world should not be an empty decorative map. Smaller threats and other encounters should appear according to world time, location, current conditions, quests, and other authored rules rather than being limited to a single global threat flag.

The encounter layer groups authored profiles by optional `spawn_group`. One eligible weighted alternative from each group may materialize as a persistent stationary record when a local map is entered. Profiles without an explicit group share the default pool, while distinct named groups can produce multiple independent records on one map. Each record snapshots its identity, quest link, combat profile, health, group, and location; later content changes or re-entry do not silently reroll or reset it. A group whose conditions are not yet satisfied may materialize when the player next enters the map after those conditions become true.

Enemies may later roam through the spatial world rather than remaining permanently fixed to one tile/location. Roaming, awareness, pursuit, encounter respawning, and off-screen updates remain broader simulation work.

The existing world/time/event systems should be extended to drive this behavior where practical instead of creating an unrelated second simulation.

## Spatial combat direction

Combat should eventually become spatial and action-oriented while retaining the game's RPG/stat foundations and textual feedback where useful.

Combat should include movement and positioning so distance, positioning, timing, and risk affect the outcome.

Player movement should be assisted rather than requiring pixel-perfect manual positioning. The player should generally be auto-positioned or magnetically aligned to meaningful combat ranges/targets while the player controls higher-level combat actions such as attack timing, hit combos, parrying, blocking, and similar decisions.

Assistance should be influenced by character capability. Higher-level or more capable characters could have better reflexes, timing, or positioning assistance. This may require a new agility/dexterity/reflex-oriented stat, but the stat model should be designed deliberately rather than added solely for this feature.

## Important design constraint

This is a **substantial evolution of the existing game**, not a mandate to replace every screen with spatial gameplay and not a commitment to preserve a text-first product identity. Spatial presentation should be the primary experience for movement, exploration, environmental interaction, and future action combat. Dialogue, records, history, menus, and context-sensitive actions may remain text-based where that best serves the character-focused experience.
