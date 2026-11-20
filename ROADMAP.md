# The Ever-Unfolding Saga — Roadmap

This is the authoritative overview of unfinished work, current priorities, and planned milestones. It should be possible to understand what remains without searching the source tree or relying on open GitHub issues.

Completed work in the current development line belongs in [CHANGELOG.md](CHANGELOG.md); earlier completed work belongs in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md). Detailed design decisions live in [docs/design/](docs/design/README.md), while [system documentation](docs/systems/) describes current implementation behavior and boundaries.

## Previous development line — v0.52.x

**Repository version at the end of this development line: 0.52.70.**

**Theme:** spatial gameplay and presentation foundations.

The experimental hybrid 2D-gameplay / 3D-rendering work is maintained independently in the private [TEUS Lab repository](https://github.com/Golden-Bastion/teus-lab). Its implementation, progress, validation records, and decisions are tracked there, not as remaining TEUS milestone work. The production game continues to use its existing 2D renderer; any future integration requires a separate, explicitly approved TEUS change.

The line has established the spatial exploration prototype, character/world authoring tools, spatial-first in-session navigation, contextual interactions, player animation, data-driven persistent encounter materialization with multiple independently selected encounter groups per local map, and the retained-presentation architecture. World Lab and its map editor now share the Character Lab two-pane workspace. Shared action groups give every button the width of the widest intrinsic button, within the available group width, and preserve word-boundary wrapping; the pause menu explicitly retains full-width actions. The delivered scope is recorded in the current changelog.

### Remaining work

1. **Continue visual and interface development against the approved standards.** The shared pixel-art direction, reference influences, palette principles, character proportions, perspective, and production boundaries are documented. The current Character hub and desktop/Android spatial control arrangements are accepted as complete for the present scope; revisit them only when a concrete feature or playtest finding justifies it. The exploration HUD gives player vitality a stacked panel and a full-width, numeric health bar. Quest tracking is handled in Character Hub → Quests, with the World Map marking the next resolvable objective location; quest titles, objective text, and progress do not occupy the main HUD. The spatial actor renderer now distinguishes NPCs, hostile encounters, corpses, and defeated encounters through role-specific mod-resolvable asset candidates, centralized sprite presentation, and safe existing-art fallbacks. Dedicated base actor sprites and per-character variation remain outstanding art work. Continue incremental pixel-art coverage, and defer useful local-orientation cues until the world-topology foundations mature. Defer bespoke enemy/boss HUD elements and combat-specific asset production until the transitional encounter UI is replaced by real-time spatial combat; do not heavily polish the current combat presentation. See [pixel-art visual direction](docs/design/pixel-art-direction.md) and [presentation direction](docs/design/presentation-direction.md).

The retained-shell and keyed-row UI migration is complete and has been owner-validated through playtesting. New dynamic collections and screen regions should follow the established retained-host and keyed-identity contracts; this is ongoing architectural practice, not an unfinished migration.

The interim World Map is complete for its current purpose: a topological view of known locations with pan/zoom and existing fast-travel behavior. Its screen and current presentation are not a pending redesign. A future geography-backed map depends on the world-generation and topology foundations below, not another interim-map UI pass.

## Current development line — v0.53.x: world topology and spatial-world expansion

**Repository version at this roadmap revision: 0.53.0.**

**Goal:** extend the local spatial prototype into a persistent world whose connected locations and regions have coherent, deterministic foundations.

**Delivered in v0.53.0:** introduced authoritative persistent route records with stable route identities and explicit directionality; migrated legacy location adjacency during save loading without consuming entity IDs; linked local-map passages to route IDs; and added non-destructive passage reconciliation for established maps. Route profiles, terrain constraints, and travel costs are intentionally deferred.

- Define shared macro-geography and stable regional/global coordinates. Adjacent regions must agree at their boundaries regardless of generation order. Deterministically embed existing saved worlds into the coordinate-backed model through a one-time, persisted compatibility migration that preserves their IDs, graph, and local maps.
- Anchor authored and generated locations to physical geography and make route edges correspond to plausible terrain connections and local-map exits.
- Expand frontier regions on demand without generating the entire world or rerolling established geography, routes, local maps, or discoveries.
- Keep physical travel and known-route fast travel complementary, including travel time, encounter/event interruption, and conservative migration of older worlds.
- Make exploration and local maps available consistently across supported location types while preserving direct location-level actions where spatial traversal is unnecessary.
- Expand the data-driven encounter foundation with multiple independent encounters, persistent roaming entities, and investigation/clue progression where supported by the underlying world and simulation rules.
- Establish the initial state-driven audio/music boundary, transition behavior, separate music/SFX settings, and provenance tracking.
- Continue incremental pixel-art asset work against the approved visual standard.

The [world-topology compatibility audit](docs/design/world-topology-compatibility-audit.md) records the implementation baseline and migration invariants. The first route-identity layer is now implemented; next work is coordinate-backed geography and deterministic embedding of existing worlds. Design references: [world topology and hybrid travel](docs/design/world-topology-and-travel.md), [spatial gameplay](docs/design/spatial-gameplay.md), [audio system](docs/design/audio-system.md), and [pixel-art direction](docs/design/pixel-art-direction.md).

## Following milestone — v0.54.x: world content, simulation, character lives, and action combat

**Goal:** make the expanded world more coherent and reactive, and let individual character lives create lasting, comprehensible consequences.

- Build deterministic, context-aware procedural content generation for regional and location identity, landmarks, resources, inhabitants, factions, discoveries, quests, and opportunities. Authored content and mods retain precedence; locations need identity, not mandatory loot or encounters.
- Extend mod support with schema-validated, mod-authored procedural templates and archetypes, stable-ID-based additions/overrides, and clear precedence with authored campaign content. Defer arbitrary scripted hooks until a concrete design justifies their complexity.
- Strengthen the authoritative world-time and simulation contract. Support detailed, event-driven, aggregate, and deferred updates according to gameplay relevance, with explicit state ownership, stable ordering, persistence, and duplicate prevention.
- Extend character progression into meaningful choices, optional backgrounds and ambitions, permanent life records, retirement into persistent NPCs, and continuity rules separating world state, recorded knowledge, and individual knowledge.
- Design and introduce real-time spatial action combat as a deliberate replacement path for the current turn-based encounter model. Establish attack phases, Strong/Fast/Group styles, target handling, assisted positioning, combos, block/parry, injury consequences, structured combat events, and desktop/Android controls. Develop bespoke combat HUD elements and combat-specific art alongside this replacement system rather than polishing the transitional encounter UI in advance.
- Profile representative desktop and Android workloads before deciding simulation limits or presentation budgets. Do not impose arbitrary ceilings on future simulation depth.

Design references: [procedural content generation](docs/design/procedural-content-generation.md), [world simulation](docs/design/world-simulation.md), [character lives and inheritance](docs/design/character-lives-and-inheritance.md), and [real-time spatial combat](docs/design/real-time-spatial-combat.md).

## Later milestone — v0.55.x: social power, livelihoods, and command

**Goal:** turn relationships, resources, and preparation into practical ways to act on and influence the persistent world.

- Establish target-specific loyalty and trust as independent relationship axes; distinguish voluntary commitment, contractual cooperation, fear-induced compliance, coercion, and force.
- Add persistent agreements, recruitment, assignments, group membership, leadership, shared resources, and organizations that can outlive a character.
- Build a small connected livelihood loop using shared resources and inventory across gathering, productive work, crafting, alchemy, and trade.
- Make preparation meaningful through equipment, consumables, information, allies, and supplies without imposing a mandatory continuous-needs-management loop.
- Extend spatial combat toward small-group tactical command, then larger group/organization behavior only when the shared systems support it.

Design references: [social influence and organizations](docs/design/social-influence-and-organizations.md) and [livelihoods, preparation, and command](docs/design/livelihoods-preparation-and-command.md).

## Dependencies and sequencing

- World topology/geography is the foundation for frontier expansion and location-aware procedural content.
- Procedural content builds on stable region/location identities and should materialize content idempotently rather than rebuilding the world.
- World simulation provides the time/update/persistence contract used by character lives, social systems, livelihoods, and world evolution.
- Character lives and inheritance depend on correct world-vs-character ownership and structured persistent consequences.
- Social systems and livelihood/command systems reuse the simulation and character-continuity foundations rather than creating separate management games.
- Real-time combat builds on spatial movement, actor, animation, and input foundations; audio consumes structured gameplay events without owning combat rules.

These version groupings express intended dependency order, not a promise that every bullet ships together. Split them into bounded implementation milestones as dependencies become ready. Update this file whenever a goal is completed, deferred, re-scoped, or superseded.
