# Design specifications

This directory contains maintained detailed specifications for future and evolving game systems. These documents describe intended behavior, system boundaries, constraints, and acceptance criteria; they do not claim that described systems are implemented.

The root [ROADMAP.md](../../ROADMAP.md) is the only project-wide index of remaining work and milestone sequencing. [CHANGELOG.md](../../CHANGELOG.md) records completed work in the current development line; [CHANGELOG-HISTORY.md](../../CHANGELOG-HISTORY.md) preserves earlier milestone history.

## Product direction

- [Game vision](game-vision.md) — character-focused experience, self-directed lives, exploration, and inherited world history.
- [Primary player experience](primary-player-experience.md) — exploration and combat as central pleasures, player freedom, world judgment, lethal consequences for offenses, pursuit, and legacy.
- [Spatial gameplay](spatial-gameplay.md) — safe-location actions, spatial exploration, investigation, dynamic encounters, and eventual action combat.
- [World scale and travel](world-scale-and-travel.md) — minimum sense of scale for local maps, locations, regions, overland journeys, and travel time.
- [Pixel-art visual direction](pixel-art-direction.md) — graphics-first visual identity, asset consistency, and incremental migration.
- [Audio system](audio-system.md) — music, ambient sound, state-driven transitions, volume settings, and asset provenance.

## Presentation and interface

- [Presentation direction](presentation-direction.md) — intended visual language, exploration/combat HUD hierarchy, character hub, map presentation, shared feedback, and desktop/Android adaptation.
- [3D spatial world and presentation](3d-spatial-world-and-presentation.md) — approved authoritative 3D gameplay world, seamless traversal, hybrid geometry, movement, environmental modification, coordinates, chunk streaming, persistence, and 2D-led presentation.
- [Historical 2D-gameplay/3D-rendering proposal](hybrid-2d-gameplay-3d-rendering-history.md) — superseded design record retained for context; it is not the current target.

## World and character systems

- [World topology and seamless traversal](world-topology-and-travel.md) — natural regions, geographical adjacency, travel connectivity, progressive expansion, and migration from the current graph/local-map implementation.
- [World-topology compatibility audit](world-topology-compatibility-audit.md) — current runtime identities, generation, graph/exit, local-map, and save contracts to preserve during v0.53.x expansion.
- [Procedural content generation](procedural-content-generation.md) — coherent region/location identity, inhabitants, opportunities, authored-content precedence, and determinism.
- [World simulation](world-simulation.md) — authoritative time, continuous/event-driven/deferred simulation, persistent consequences, and update guarantees.
- [World simulation architecture](world-simulation-architecture.md) — centralized authoritative state, deterministic parallel execution, world time, events, extensibility, versioning, budgets, and fidelity coordination.
- [Simulation processor architecture](simulation-processor-architecture.md) — domain-owned state, focused processors, execution triggers, communication, derived state, suspension, fidelity, concurrency, and diagnostics.
- [Startup recovery and safe mode](startup-recovery-and-safe-mode.md) — Bevy-independent startup diagnostics, copyable crash reports, manual and automatic safe-mode entry, and recovery limits.
- [Simulation domains and shared infrastructure](world-simulation-domains.md) — five-domain responsibility map, authoritative state ownership, cross-domain mechanics, player-facing scope, and shared infrastructure.
- [Challenge and difficulty philosophy](challenge-and-difficulty.md) — fixed world challenge, no difficulty presets or adaptive scaling, preparation, fairness, failure, and accessibility.
- [NPC behaviour](npc-behavior.md) — personality, perception, knowledge, decision-making, navigation, group behaviour, and simulation fidelity.
- [Character lives and inheritance](character-lives-and-inheritance.md) — progression, ambitions, permanent life records, retirement, death, and successor continuity.
- [Social influence and organizations](social-influence-and-organizations.md) — loyalty/trust, cooperation, recruitment, assignments, group structure, and persistence.
- [Livelihoods, preparation, and command](livelihoods-preparation-and-command.md) — connected resources, production, trade, expedition preparation, and tactical command.

## Combat

- [Real-time spatial combat](real-time-spatial-combat.md) — attack styles, timing, combos, defense, positioning, injuries, enemy behavior, and combat presentation.

## Maintenance rules

Keep detailed design decisions here rather than in the active roadmap or scattered implementation documents. When a design changes, update the relevant specification and preserve important rationale. Keep the roadmap focused on what remains, when it is intended to happen, and its dependencies. Issues may be closed when they served only as design records and their full substance has been migrated here; implementation work must remain visible in the roadmap or active task issues.
