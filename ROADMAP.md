# The Ever-Unfolding Saga — Roadmap

This is the authoritative overview of unfinished work, current priorities, and planned milestones. It should be possible to understand what remains without searching the source tree or relying on open GitHub issues.

Completed work in the current development line belongs in [CHANGELOG.md](CHANGELOG.md); earlier completed work belongs in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md). Detailed design decisions live in [docs/design/](docs/design/README.md), while [system documentation](docs/systems/) describes current implementation behavior and boundaries.

## Current development line — v0.53.x: world topology and spatial-world expansion

**Repository version at this roadmap revision: 0.53.12.**

**Goal:** extend the local spatial prototype into a persistent world whose connected locations and regions have coherent, deterministic foundations.

**Delivered in v0.53.0:** introduced authoritative persistent route records with stable route identities and explicit directionality; migrated legacy location adjacency during save loading without consuming entity IDs; linked local-map passages to route IDs; and added non-destructive passage reconciliation for established maps.

**Delivered in v0.53.1:** established signed global macro-sample and region-chunk coordinates, fixed the region resolution at 256 × 256 samples, added deterministic coordinate-derived sample seeds and geography-ruleset metadata, and assigned stable coordinates to newly generated regions. Legacy worlds remain unembedded for the separately planned compatibility migration.

**Delivered in v0.53.2:** added a deterministic fixed-point surface elevation field with continent-scale, regional, and detail noise, producing connected landmasses and smaller coastal/island variation on the shared global sample lattice.

**Delivered in v0.53.3:** added deterministic regional temperature and moisture fields, elevation cooling, ocean-influenced moisture, precipitation potential, and renderer-independent surface biome classification. The initial catalogue covers ocean, coast, desert, grassland, temperate forest, taiga, tundra, and alpine environments.

**Delivered in v0.53.4:** added explicit, direction-aware shoreline samples for both land and water, derived from neighboring global elevation samples and shared by climate biome classification.

**Delivered in v0.53.5:** added a hierarchical watershed foundation with coarse downstream links, local within-cell D8 drainage, and bounded tracing to ocean or closed inland outlets.

**Delivered in v0.53.6:** added deterministic, bounded upstream contributing-area estimates and river-channel classification on the coarse watershed network. Queries inspect at most 128 contributing cells and explicitly report truncated estimates.

**Delivered in v0.53.7:** added bounded depression analysis for closed inland watershed outlets. A deterministic minimax search resolves the lowest local spill elevation and connected below-spill footprint within a fixed 17 × 17 coarse-cell window; it describes terrain basin potential without claiming persistent lake placement or simulated water volume.

**Delivered in v0.53.8:** extended bounded reverse-watershed accumulation to sum each contributing coarse cell's deterministic precipitation potential alongside contributing area. The exposed Q16 × cell value is a relative index, not physical discharge; existing channel classification remains area-based, and truncated sums are explicitly lower bounds.

**Delivered in v0.53.9:** added deterministic inland-lake coverage from connected closed-basin footprints, precipitation-scaled fill levels, and stable basin-derived IDs. Added optional world-persisted generic modifications keyed by those IDs, with backward-compatible defaults for existing saves. River inflow is exposed as an initial outlet-channel indicator; physical discharge and connected water simulation remain future work.

**Delivered in v0.53.10:** added a renderer-independent surface-water sampler that composes ocean, inland-lake, and river-channel classification. River samples expose their immediate downstream cell and classify the destination as ocean, lake, river, lower-threshold land, an inconclusive truncated estimate, or a closed basin, reusing the existing downhill watershed graph rather than introducing a second routing simulation.

**Delivered in v0.53.11:** added deterministic, warped ridged mountain belts whose bounded uplift contributes to the shared surface elevation field. Climate and watershed systems therefore sample the same mountain-shaped terrain. Geography ruleset 2 enables this field while ruleset 1 preserves the original elevation contract.

**Delivered in v0.53.12:** exposed the existing mountain-ridge strength and barrier classification through the shared surface-terrain sample, allowing future route and location-placement systems to evaluate mountain crossings without duplicating sampling. Ridge strength is a soft crossing-resistance signal; weaker corridors can guide candidate mountain-pass placement. Water crossing constraints remain distinct through the existing hydrology sampler. No geography ruleset or save-format change was required.

The v0.52.x development line is complete. The project owner has confirmed the integrated desktop/Android spatial-first gameplay verification and the separate spatial presentation/player-animation playtesting tracked by [#421](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/421) and [#446](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/446). The full delivered scope and final verification record are preserved in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md); the concise published summary and release assets are available in the [v0.52.70 GitHub Release](https://github.com/Golden-Bastion/the-ever-unfolding-saga/releases/tag/v0.52.70).

The initial world-topology compatibility audit is complete in [world-topology compatibility audit](docs/design/world-topology-compatibility-audit.md). It records the current identity, generation, route, local-map, and save invariants, plus the owner-selected requirement to deterministically embed existing worlds through a one-time persisted migration. It does not change gameplay or save data.

### Remaining work

1. **Generate continuous continental macro-geography.** In progress: added deterministic continental elevation, climate/biomes, shorelines, watersheds, relative river-channel estimates, inland lakes, connected surface-water classification, mountain ranges, and route-facing mountain-barrier signals on the shared global sample lattice. Remaining: a dedicated underground-strata generator and further geography integration with location footprints and physical route constraints. Physical discharge calibration and seasonal water balance are not required; seasonal rainfall/drought and their gameplay consequences belong to the time-driven world-simulation work. Neighboring chunks must remain coherent regardless of generation order.
2. **Embed existing saved worlds.** Implement the one-time persisted compatibility placement and seed migration, preserving region/location IDs, `next_id`, routes, history, authored content, and saved local maps.
3. **Anchor authored and generated locations to physical geography.** Define stable placement/footprint and environmental constraints; make route edges correspond to plausible terrain connections and local-map passages without moving established placements.
4. **Expand frontier regions on demand** without generating the entire world or rerolling established geography, routes, local maps, or discoveries.
5. **Keep physical travel and known-route fast travel complementary,** including travel time, encounter/event interruption, and conservative migration of older worlds.
6. **Make exploration and local maps available consistently across supported location types** while preserving direct location-level actions where spatial traversal is unnecessary.
7. **Expand the data-driven encounter foundation** with multiple independent encounters, persistent roaming entities, and investigation/clue progression where supported by the underlying world and simulation rules.
8. **Establish the initial state-driven audio/music boundary,** transition behavior, separate music/SFX settings, and provenance tracking.
9. **Continue incremental pixel-art production** against the approved visual standard as new visual needs emerge. The dedicated NPC, hostile, and remains sprites are included in v0.52.70; [#484](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/484) remains open only for the owner's desktop/Android runtime-scale review of those new assets. Broader asset coverage remains ongoing alongside topology implementation.
10. **Extend modular character appearance into animated spatial actors.** Build on the existing stable-ID character-part catalogue and dialogue portrait compositor to assemble the player, NPCs, hostiles, and relevant corpse/aftermath visuals from independently authored anatomy, clothing, equipment, and condition layers. Keep all layers aligned to shared directional animation frames; support pose-aware limb/equipment variants, deterministic selection, mod overrides, and cached composition rather than excessive per-layer render entities. Include state-driven bruises, wounds, bloodied/torn clothing, damaged equipment, and gore/aftermath layers, while leaving injury outcomes and persistent blood/corpse lifecycle authoritative in gameplay/combat systems. Preserve whole-sprite fallbacks and profile desktop/Android memory and rendering costs. Track implementation in [#491](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/491).

The compatibility audit, persistent route-identity layer, coordinate contract, surface-elevation field, initial climate/biome layer, explicit shoreline data, hierarchical watershed foundation, and bounded river-channel extraction with precipitation-potential weighting, bounded depression/spill analysis, initial deterministic lake generation with persistent modification records, and first-pass river-to-waterbody connectivity are complete. Continue the remaining macro-geography features before legacy embedding and geography-backed location placement. Design references: [world topology and hybrid travel](docs/design/world-topology-and-travel.md), [macro-geography system](docs/systems/macro-geography.md), [spatial gameplay](docs/design/spatial-gameplay.md), [audio system](docs/design/audio-system.md), and [pixel-art direction](docs/design/pixel-art-direction.md).

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
