# The Ever-Unfolding Saga — Roadmap

This is the authoritative overview of unfinished work, current priorities, and planned milestones. It should be possible to understand what remains without searching the source tree or relying on open GitHub issues.

Completed work in the current development line belongs in [CHANGELOG.md](CHANGELOG.md); earlier completed work belongs in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md). Detailed design decisions live in [docs/design/](docs/design/README.md), while [system documentation](docs/systems/) describes current implementation behavior and boundaries.

## Current development line — v0.52.x

**Repository version at this roadmap revision: 0.52.50.**

**Theme:** spatial gameplay and presentation foundations.

The line has established the spatial exploration prototype, character/world authoring tools, spatial-first in-session navigation, contextual interactions, player animation, and the first retained-presentation architecture. The delivered scope is recorded in the current changelog.

### Remaining work

1. **Manual spatial validation.** Complete desktop and Android landscape visual/interaction playtesting for directional idle/walk animation, foot grounding and sword alignment, Old Shrine prop and wall occlusion, camera framing, HUD readability, and movement controls. Record reproducible defects as focused follow-up issues. [#446](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/446)
2. **Continue the retained UI migration.** Introduce keyed row reconciliation for dynamic inventories, quests, save slots, and mod lists; further isolate spatial topology changes from HUD/content refreshes; and define coordinated screen-exit transitions. Stable hosts must preserve scroll, focus, and selection. See [UI presentation architecture](docs/systems/ui-presentation.md).
3. **Extend the spatial gameplay foundation.** Define explicit room/door topology before introducing room-aware camera transitions; add further authored local layouts and authoring support; and expand investigation beyond the initial points of interest into clue/evidence progression and dynamic encounters. Preserve renderer-neutral action/target identity and authoritative gameplay rules. See [spatial gameplay design](docs/design/spatial-gameplay.md) and [spatial exploration](docs/systems/spatial-exploration.md).
4. **Formalize the visual and interface language.** Establish the shared pixel-art style, palette, scale, character proportions, perspective, layering, and asset workflow; continue evolving contextual HUD hierarchy, objective/orientation cues, character-hub presentation, and platform-adaptive controls without overloading the spatial view. See [pixel-art visual direction](docs/design/pixel-art-direction.md) and [presentation direction](docs/design/presentation-direction.md).

The interim World Map is complete for its current purpose: a topological view of known locations with pan/zoom and existing fast-travel behavior. Its screen and current presentation are not a pending redesign. A future geography-backed map depends on the world-generation and topology foundations below, not another interim-map UI pass.

## Next milestone — v0.53.x: world topology and spatial-world expansion

**Goal:** extend the local spatial prototype into a persistent world whose connected locations and regions have coherent, deterministic foundations.

- Define shared macro-geography and stable regional/global coordinates. Adjacent regions must agree at their boundaries regardless of generation order.
- Anchor authored and generated locations to physical geography and make route edges correspond to plausible terrain connections and local-map exits.
- Expand frontier regions on demand without generating the entire world or rerolling established geography, routes, local maps, or discoveries.
- Keep physical travel and known-route fast travel complementary, including travel time, encounter/event interruption, and conservative migration of older worlds.
- Make exploration and local maps available consistently across supported location types while preserving direct location-level actions where spatial traversal is unnecessary.
- Extend the spatial experience with dynamic threats, roaming entities, and investigation/clue progression where supported by the underlying world rules.
- Establish the initial state-driven audio/music boundary, transition behavior, separate music/SFX settings, and provenance tracking.
- Continue incremental pixel-art asset work against a documented visual standard.

Design references: [world topology and hybrid travel](docs/design/world-topology-and-travel.md), [spatial gameplay](docs/design/spatial-gameplay.md), [audio system](docs/design/audio-system.md), and [pixel-art direction](docs/design/pixel-art-direction.md).

## Following milestone — v0.54.x: world content, simulation, character lives, and action combat

**Goal:** make the expanded world more coherent and reactive, and let individual character lives create lasting, comprehensible consequences.

- Build deterministic, context-aware procedural content generation for regional and location identity, landmarks, resources, inhabitants, factions, discoveries, quests, and opportunities. Authored content and mods retain precedence; locations need identity, not mandatory loot or encounters.
- Extend mod support with schema-validated, mod-authored procedural templates and archetypes, stable-ID-based additions/overrides, and clear precedence with authored campaign content. Defer arbitrary scripted hooks until a concrete design justifies their complexity.
- Strengthen the authoritative world-time and simulation contract. Support detailed, event-driven, aggregate, and deferred updates according to gameplay relevance, with explicit state ownership, stable ordering, persistence, and duplicate prevention.
- Extend character progression into meaningful choices, optional backgrounds and ambitions, permanent life records, retirement into persistent NPCs, and continuity rules separating world state, recorded knowledge, and individual knowledge.
- Design and introduce real-time spatial action combat as a deliberate replacement path for the current turn-based encounter model. Establish attack phases, Strong/Fast/Group styles, target handling, assisted positioning, combos, block/parry, injury consequences, structured combat events, and desktop/Android controls.
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
