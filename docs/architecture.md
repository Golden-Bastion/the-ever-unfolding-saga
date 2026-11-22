# The Ever-Unfolding Saga — Architecture

## Purpose

This document defines the structural boundaries of the game. It describes where responsibilities live, how systems depend on one another, and the rules that keep the codebase modular. Product-level goals are defined in [the canonical design principles](design-principles.md); this document translates them into technical boundaries and does not claim that every planned system is already implemented.

## Architectural goals

The architecture is built around four principles:

- World simulation is independent from presentation.
- Gameplay content is data-driven rather than embedded in engine logic.
- Persistent world state is separate from character-specific state where possible.
- Systems communicate through explicit responsibilities instead of reaching into one another's internals.

The project should prefer clear responsibility boundaries over splitting modules merely to reduce line counts.

## High-level layers

```text
Application / Game Flow (Bevy)
        │
        ├── Lifecycle / Navigation / Screen orchestration
        ├── Presentation adapters
        └── Semantic input
        │
        ▼
Gameplay Systems
        │
        ├── Gameplay Actions
        ├── Gameplay Interactions
        ├── Character Progression
        ├── Legacy / Death
        ├── Combat
        ├── Console Commands
        └── World / Bootstrap
        │
        ▼
Core State & Models
        │
        ├── World state
        ├── Character state
        ├── Entities
        └── Historical state
        │
        ├───────────────┐
        ▼               ▼
Content / Events    Persistence
```

Content loading supplies definitions to the runtime but should not own live world state. Presentation reads state and produces output but should not implement game rules. Persistence serializes and restores state but should not decide gameplay outcomes.

## Current module direction

The current codebase has been progressively decomposed from large modules into responsibility-based modules. The intended structure is approximately:

```text
src/
├── main.rs                  # Bevy application entry point
├── bevy_app.rs              # Bevy app/bootstrap configuration
├── bevy_presentation.rs     # shared Bevy UI primitives and semantic input routing
├── bevy_lifecycle.rs        # start/load/creation/quit/death lifecycle orchestration
├── bevy_navigation_bridge.rs # lifecycle completion and spatial exploration hand-off
├── bevy_gameplay.rs         # spatial pause/navigation overlay and interim World Map
├── bevy_interactions.rs     # NPC dialogue and remains-recovery screens
├── bevy_records.rs          # Character hub, record details, meditation, and journal flows
├── bevy_combat.rs           # combat presentation and interaction
├── bevy_spatial.rs          # Bevy spatial exploration rendering, movement, camera, and controls
├── bevy_console.rs          # developer-console presentation and input
├── game.rs                  # gameplay module facade
├── game/
│   ├── actions.rs           # core gameplay actions and renderer-neutral action logic
│   ├── character.rs         # character progression and character-sheet data
│   ├── combat.rs            # combat encounter processing and result view construction
│   ├── console.rs           # developer-console session/state boundary
│   ├── console_commands.rs  # developer-console command implementation
│   ├── console_ui.rs        # renderer-neutral console interaction state and view construction
│   ├── history_screen.rs    # renderer-neutral history view construction
│   ├── interactions.rs      # NPC dialogue, quest interaction, faction memory/reputation
│   ├── legacy.rs            # death, corpses, previous-life recovery
│   ├── navigation.rs        # renderer-neutral world navigation views/rules
│   ├── quests.rs            # quest state and progression rules
│   ├── records.rs           # renderer-neutral record views and mutations
│   ├── state_effects.rs     # shared state-effect helpers
│   ├── time.rs              # time calculations and display values
│   ├── world.rs              # world bootstrap and loaded-state validation
├── presentation.rs          # frontend-independent presentation/view models
├── content.rs               # content module facade
├── content/
│   ├── definitions.rs       # schemas and validation definitions
│   ├── diagnostics.rs      # content diagnostics
│   ├── loader.rs            # base/mod loading and merging
│   └── seeding.rs           # content-to-world translation
├── events.rs                # event runtime
├── model.rs                 # shared game-state and entity models
├── geography.rs             # renderer-neutral macro-geography and persistent placement foundations
│   ├── terrain.rs           # deterministic surface elevation fields on the global macro lattice
│   ├── locations.rs         # persistent location anchors, footprints, and legacy embedding
│   ├── routes.rs            # bounded terrain-aware route planning and crossing profiles
│   ├── frontier.rs          # stable on-demand frontier identities and adjacent-region materialization
│   ├── underground.rs       # deterministic chamber nodes and reciprocal cave-network passages
│   ├── vertical.rs          # bounded deterministic surface entrances and inter-stratum shafts
│   ├── mountains.rs         # deterministic mountain-ridge strength and terrain uplift
│   ├── watershed.rs         # hierarchical coarse drainage links and local D8 sampling
│   ├── rivers.rs            # bounded contributing-area and precipitation-potential estimates for river channels
│   ├── depressions.rs       # bounded closed-basin spill-level and footprint analysis
│   ├── waterbodies.rs       # deterministic inland-lake coverage and stable identities
│   ├── hydrology.rs         # unified surface-water classification and immediate river destinations
│   ├── coast.rs             # shoreline adjacency and direction masks derived from elevation
│   └── climate.rs           # deterministic surface climate and biome classification
├── spatial.rs               # renderer-neutral local-map model and deterministic generation
├── persistence.rs           # save/load and migrations
├── input.rs                 # frontend-neutral semantic interaction events
└── rng.rs                   # deterministic/random generation helpers
```

The geography module owns signed global/region coordinate conversion, deterministic coordinate-derived seeds, renderer-independent surface and underground terrain fields, persisted location anchors/footprints, one-time legacy-world embedding, bounded terrain-aware route planning, and on-demand cardinal frontier materialization. Deterministic vertical-connection candidates join exposed surface land to open cave space and adjacent underground strata, but still await world-route/local-map integration. Route records persist canonical macro paths, broad profiles, and explicit crossing requirements. New local-map passages use the planned route heading and inherit its profile without moving established passages. The direct-corridor sampler remains diagnostic; the A* planner is overland-only, blocks ocean terrain, and treats elevation and mountain ridges as soft costs. Underground cave sampling exposes sparse chamber nodes and reciprocal passages on negative strata, but route connections to caves await explicit vertical entrances. Surface elevation, shoreline adjacency, regional temperature and moisture, biome classification, hierarchical watershed drainage, contributing-area and precipitation-potential estimates for river channels, bounded closed-basin depression analysis, deterministic inland-lake coverage, and unified surface-water connectivity are sampled from the shared global lattice. Truncated river estimates below the channel threshold remain explicitly inconclusive. Terrain and climate use deterministic fixed-point noise; coarse watershed links descend strictly by representative elevation, while local D8 directions remain within their coarse cell. The precipitation-potential sum is a relative index rather than physical discharge, and channel classification remains area-based. Hydrology remains renderer-independent and derives on demand. The hydrology facade composes ocean, lake, and river samples and exposes immediate river destinations over the existing strictly downhill watershed graph; it does not simulate discharge or water transport. Generated waterbody geometry is not saved; only explicit, stable-ID-keyed waterbody modifications are persisted in world state.

The exact module list may evolve, but new modules should represent meaningful responsibilities rather than arbitrary slices of large files.

## Game flow

`main.rs` starts the Bevy application. Bevy lifecycle systems own start/load/creation/quit/death presentation and navigation. The gameplay-menu adapter owns the spatial pause/navigation overlay and interim World Map, while dedicated Bevy adapters own records, interaction, combat, and console screens.

Gameplay systems operate on the authoritative model and expose renderer-neutral view data where a frontend needs it. Character progression owns experience gain, level advancement, and character-sheet data. Gameplay interactions own NPC dialogue, quest offering/turn-in, faction memory/reputation updates, NPC availability, and remains recovery. Legacy gameplay owns character death, corpse creation, corpse recovery, and previous-life item recovery. Combat is isolated from general action handling. World/bootstrap logic owns world initialization and validation. Persistence serializes and restores state. The developer console owns command/session state while Bevy owns its rendering and input presentation.

This keeps frontend orchestration separate from simulation rules without maintaining a parallel terminal game loop.

## State ownership

The model represents the authoritative simulation state. Systems should mutate state through explicit functions belonging to the appropriate owner.

World-level state includes persistent locations, factions, world history, event cooldowns, corpses, placed items, explicit waterbody modifications, and other changes that survive character death.

Character-level state includes attributes, experience, conditions, inventory, active quests, and other properties belonging to the current life.

When a character dies and a world is inherited, character-specific state is discarded or intentionally reconstructed while persistent world state remains.

## Entity identity

Every persistent entity uses a stable unique ID. References should use IDs rather than display names.

Names, descriptions, and other presentation fields are not identity. This allows content to be renamed without breaking relationships and makes save compatibility and mod merging more predictable.

## Content architecture

Gameplay definitions are loaded from structured content rather than hardcoded throughout runtime code.

The content layer is responsible for:

- Definitions and schemas.
- Content validation.
- Base content loading.
- Mod discovery and loading.
- Merging by stable identifiers and keys.
- Translation of definitions into initial world state.

The runtime consumes loaded content and should not need to know whether a definition came from the base pack or a mod.

See [`systems/content.md`](systems/content.md) for content-specific details.

## Event architecture

Events are data-driven and executed by a reusable runtime. Definitions can specify triggers, weights, chance gates, conditions, effects, and cooldowns. Event execution can produce persistent world changes and structured history records.

The event system should remain independent of individual hardcoded travel or quest branches.

See [`systems/events.md`](systems/events.md) for details.

## Persistence architecture

Persistence serializes the authoritative world/character state and restores it into a valid runtime state. Save compatibility and migrations belong to the persistence boundary.

Campaign content itself is runtime data and should not be redundantly embedded in save files when it can be safely reloaded from the current content definitions. Runtime-generated procedural events are persisted separately from authored content so procedural progression survives save/load.

## Presentation architecture

Presentation consumes authoritative state and produces frontend-independent view data before frontend-specific rendering occurs. The root `presentation.rs` module contains shared view models expressed only through domain-neutral owned data such as strings, scalars, and collections; it does not depend on Bevy, ratatui, crossterm, or gameplay actions.

Bevy modules translate those view models into UI nodes and semantic input. `bevy_presentation.rs` owns reusable visual primitives, navigation state, and conversion from Bevy keyboard/button interaction into `InputEvent` values. Gameplay, records, interactions, combat, and console adapters consume these semantic events rather than engine-specific keyboard values in their gameplay rules.

The developer console keeps its command editing, history, completion, scrolling, and view construction in `game/console_ui.rs` and `game/console.rs`; Bevy renders that `ConsoleView` without coupling the command state to a renderer.

The terminal frontend has been removed. Ratatui/crossterm are no longer application dependencies, and there is no parallel terminal screen or renderer layer in the shipped application.

Character-sheet presentation is owned by the character module because it is directly tied to character progression state rather than a general action dispatcher.

Legacy mechanics remain independent from lifecycle presentation: `legacy.rs` owns death/corpse state changes and recovery data construction, while Bevy lifecycle systems coordinate the death flow.

Interaction mechanics remain independent from frontend presentation: `interactions.rs` owns dialogue, quest interactions, faction/NPC memory, and availability, while `bevy_interactions.rs` owns selection and rendering.

See [`systems/ui.md`](systems/ui.md) for details.

## Product-direction constraints

- The simulation remains authoritative; visual presentation and input adapters do not decide gameplay outcomes.
- Spatial gameplay is the intended primary mode for exploration and future real-time action combat, while text-based dialogue, records, and contextual feedback remain important.
- The character’s plausible knowledge and ability to influence events constrain both simulation-facing information and command interfaces.
- Keep world-persistent state distinct from character-specific state to support death, inheritance, and long-running campaigns.
- Target desktop and mobile with platform-specific controls/presentation where practical, but do not represent an unvalidated platform as officially supported.
- Single-player gameplay and local persistence should not depend on network services.

These are design constraints, not a statement that every planned feature has already shipped.

## Dependency rules

A system should depend on abstractions or shared models appropriate to its responsibility, not on unrelated implementation details.

In particular:

- Presentation should not implement simulation rules.
- Persistence should not decide gameplay outcomes.
- Content loading should not directly own runtime character state.
- World/bootstrap code should not depend on gameplay action implementations merely to perform world initialization.
- Actions should not duplicate combat, interaction, progression, legacy, presentation, or persistence logic that already has a dedicated owner.
- Gameplay interactions may use action-owned turn/progression helpers where those helpers are still shared gameplay infrastructure, but interaction-specific rules belong in `interactions.rs`.
- Legacy mechanics should remain independent from lifecycle presentation and own only death, corpse, and previous-life recovery responsibilities.
- Character progression should remain independent from world/bootstrap and persistence implementation details.
- Shared models should remain focused on state and domain representation rather than becoming a catch-all service module.
- Bevy frontend systems should consume renderer-neutral views and semantic input rather than embed gameplay rules or persistence decisions.

## Compatibility and refactoring

Refactoring should preserve gameplay behavior, save compatibility, and screen flow unless the change explicitly intends to alter them.

When a module becomes large, first identify cohesive responsibilities and move them behind clear interfaces. Do not split code solely because a file has many lines.

Architecture changes should include focused tests around affected behavior and should avoid introducing parallel state systems.

## Documentation boundaries

`ROADMAP.md` is the authoritative overview of unfinished work and upcoming milestones.

`CHANGELOG.md` and `CHANGELOG-HISTORY.md` record completed work in the current and previous development lines. Detailed future design decisions live in `docs/design/`, while `docs/development-process.md` records engineering and documentation practices.

This file describes structural architecture and responsibility boundaries. The files under `docs/systems/` describe current system behavior and contracts; they are not alternate roadmaps.
