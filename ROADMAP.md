# The Ever-Unfolding Saga — Roadmap

This is the authoritative overview of unfinished work, current priorities, and planned milestones. It should be possible to understand what remains without searching the source tree or relying on open GitHub issues.

Completed work in the current development line belongs in [CHANGELOG.md](CHANGELOG.md); earlier completed work belongs in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md). Detailed design decisions live in [docs/design/](docs/design/README.md), while [system documentation](docs/systems/) describes current implementation behavior and boundaries.

## Current development line — v0.53.x: world topology and spatial-world expansion

**Final v0.52.x package version: 0.52.70.**

**Goal:** extend the local spatial prototype into a persistent world whose connected locations and regions have coherent, deterministic foundations.

The v0.52.x development line is complete. The project owner has confirmed the integrated desktop/Android spatial-first gameplay verification and the separate spatial presentation/player-animation playtesting tracked by [#421](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/421) and [#446](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/446). The full delivered scope and final verification record are in [CHANGELOG.md](CHANGELOG.md), which is also the source for the v0.52.70 release body.

The initial world-topology compatibility audit is complete in [world-topology compatibility audit](docs/design/world-topology-compatibility-audit.md). It records the current identity, generation, route, local-map, and save invariants, plus the owner-selected requirement to deterministically embed existing worlds through a one-time persisted migration. It does not change gameplay or save data.

### Remaining work

1. **Establish first-class persistent route records.** Replace implicit location adjacency with stable route IDs, explicit endpoints and directionality, and one authoritative topology source for travel and map presentation. Migrate legacy exits deterministically without changing runtime IDs, existing graph connectivity, or saved local maps. Start with [#488](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/488).
2. **Define shared macro-geography and stable regional/global coordinates.** Adjacent regions must agree at their boundaries regardless of generation order. Deterministically embed existing saved worlds into the coordinate-backed model through a one-time, persisted compatibility migration that preserves their IDs, graph, and local maps.
3. **Anchor authored and generated locations to physical geography.** Make route edges correspond to plausible terrain connections and local-map exits, while preserving authored-content precedence and established placements.
4. **Expand frontier regions on demand** without generating the entire world or rerolling established geography, routes, local maps, or discoveries.
5. **Keep physical travel and known-route fast travel complementary,** including travel time, encounter/event interruption, and conservative migration of older worlds.
6. **Make exploration and local maps available consistently across supported location types** while preserving direct location-level actions where spatial traversal is unnecessary.
7. **Expand the data-driven encounter foundation** with multiple independent encounters, persistent roaming entities, and investigation/clue progression where supported by the underlying world and simulation rules.
8. **Establish the initial state-driven audio/music boundary,** transition behavior, separate music/SFX settings, and provenance tracking.
9. **Continue incremental pixel-art production** against the approved visual standard as new visual needs emerge. The dedicated NPC, hostile, and remains sprites are included in v0.52.70; [#484](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/484) remains open only for the owner's desktop/Android runtime-scale review of those new assets. Broader asset coverage remains ongoing alongside topology implementation.

The compatibility audit is complete; begin persistent topology implementation with [#488](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/488). Design references: [world topology and hybrid travel](docs/design/world-topology-and-travel.md), [spatial gameplay](docs/design/spatial-gameplay.md), [audio system](docs/design/audio-system.md), and [pixel-art direction](docs/design/pixel-art-direction.md).

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
