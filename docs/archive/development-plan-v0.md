
> Historical reference only. This preserves the project's original development plan. Its early text-first assumptions and implementation sequence are superseded by the current [design principles](../design-principles.md), [design specifications](../design/README.md), and [roadmap](../../ROADMAP.md). Do not use this document as current project guidance.

## 1) Core vision

The Ever-Unfolding Saga is a dark-fantasy, text-first RPG with permadeath, procedural world generation, and persistent world inheritance. The player creates a character, plays through a run, and when that character dies they choose one of two paths: start a completely new world, or inherit the existing world and create a new character inside it.

If the player inherits, the world keeps its changes, consequences, dead bodies, abandoned items, destroyed places, faction shifts, records, and other long-term effects. The old character is gone, but the world remembers them.

## 2) Design pillars

The game is built around four non-negotiable pillars:

1. Text comes first. The first dev stage is fully text-based.
2. The world persists. Death ends a character, not the world.
3. Content stays data-driven. New items, quests, events, factions, and later images should not require engine rewrites.
4. The codebase must stay modular. Systems should not become tangled together.

## 3) Language choice

Use Rust for the core game.

Rust is a strong fit because it encourages a clear architecture, keeps state handling safer as the simulation grows, and works well for a content-heavy game where world logic, save data, and procedural systems need to stay predictable.

## 4) Architecture

The project should be split into four layers:

### Core engine
Handles world simulation, turn/state progression, combat resolution, inventory, character stats, faction logic, event processing, save/load, and procedural generation.

### Content layer
Contains items, weapons, armor, locations, NPC templates, factions, quests, events, dialogue, world-generation tables, and mod files.

### Presentation layer
Handles text output, menus, prompts, story logs, and later optional images.

### Mod/content loading layer
Loads base data, loads mod data, resolves conflicts, validates content, and reports errors clearly.

## 5) Game state model

The game should treat the world as a simulation rather than a single-player save file.

Key entity types:
- World
- Region
- Location
- Faction
- Character
- Item
- Encounter
- Event
- Quest
- Relationship
- Historical record
- Corpse or remains

Every entity must have a stable unique ID. Names are for display only.

## 6) Persistence and inheritance

The save system must support two world modes:

### New world
Creates a fresh procedural world from scratch.

### Inherited world
Loads the existing world state and continues it with a new character.

Inherited world should preserve map state, discovered locations, faction standings, destroyed or altered locations, dead bodies, placed items, quest consequences, world history, and NPC memory where relevant.

## 7) Procedural generation strategy

Do not generate the whole world in one giant pass and forget how it was built.

Use layered generation:
- World skeleton: continents, regions, climate, routes, major factions, major settlements
- Local content: towns, dungeons, roads, landmarks, ruins, camps, NPCs, resources
- Run-specific content: local events, random crises, side quests, encounters, loot placement
- Runtime mutation: destroyed buildings, dead NPCs, stolen items, changed faction control, corpses, relics, altered relationships

This keeps generation debuggable while still feeling alive.

## 8) Data-driven content

Almost all gameplay data should live outside engine code.

Recommended approach:
- Use strict structured data files for gameplay content.
- Validate content on load.
- Reject broken references early.
- Give useful error messages.

Every content type should have a schema:
- Item schema
- NPC schema
- Event schema
- Quest schema
- Location schema
- Faction schema
- Dialogue schema

## 9) Event system

Events should be built from triggers, conditions, weights, effects, and cooldowns.

The game should not hardcode one-off events everywhere. The event system needs to support random encounters, scripted scenes, world changes, quest hooks, and consequences.

## 10) Quest system

Quests should be component-based, not a pile of special cases.

A quest needs:
- objectives
- conditions
- rewards
- failure states
- branching outcomes
- persistence rules

Quests must survive world inheritance properly when their consequences matter.

## 11) Character progression

Character growth can include:
- stats
- skills
- perks
- traits
- equipment mastery
- reputation
- faction alignment
- injuries
- knowledge
- moral or ideological choices

Progression should affect how the game describes and reacts to the character, not just the numbers on a sheet.

## 12) Memory and history

This is one of the most important systems.

The world should keep a history log of meaningful events:
- who did what
- where
- when
- who witnessed it
- what changed afterward

NPCs should have memory rules for short-term, long-term, and faction-level memory. Not everyone remembers everything. Only store what matters. The current prototype now carries a first pass of faction reputation, NPC memory, and a simple quest consequence loop.

## 13) Corpses, items, and remnants

When a character dies, the game should decide what remains:
- corpse
- inventory
- equipped items
- dropped quest items
- evidence
- scars on the world

These remnants should be world objects, not just flavor text.

## 14) Text-first presentation

Stage 1 must be entirely text-based.

That means menus, descriptive scenes, combat logs, event narration, faction updates, world history summaries, character sheets, and inventory lists. Presentation must stay separated from logic so images can be added later without rewriting the game.

## 15) Mod support strategy

Build toward mod support in stages:
- Phase 1: internal content only
- Phase 2: external data loading for items, events, NPCs, and locations
- Phase 3: override support for replace, extend, add, and tweak
- Phase 4: scripted mod hooks for custom event logic, quest logic, worldgen rules, and AI behavior

Mods should use stable IDs and be validated before load.

## 16) Testing and tooling

The project should ship with internal developer tools and automated checks for:
- save/load integrity
- missing content references
- invalid IDs
- broken quest states
- worldgen edge cases
- duplicate entity creation
- mod conflicts
- inheritance persistence
- corpse and item cleanup
- history log consistency

Useful dev tools include a content validator, debug world inspector, entity viewer, event debugger, and save file checker.

## 17) Practical implementation rules

- Every entity has a unique ID.
- Every reference uses IDs, not names.
- Every content file has a schema.
- Every mod is validated before use.
- Every system has clear ownership.
- No system should directly edit unrelated systems.
- World changes must go through explicit mutation functions.
- Important events must be recorded in history.

## 18) First milestone

The first playable milestone should allow a player to:
- create a character
- enter a small procedural world
- travel between locations
- encounter simple events or threats
- manage safety before meditating or saving
- die
- choose new world or inherited world
- return to the death site and see leftover consequences

That proves the concept.

## 19) Final direction

This project should be built as a persistent world simulation with text-driven presentation, not as a content pile glued onto a combat system.

If the architecture is right early, adding items, images, quests, mods, and new systems later stays manageable. If the architecture is wrong, every new feature will break something else.
