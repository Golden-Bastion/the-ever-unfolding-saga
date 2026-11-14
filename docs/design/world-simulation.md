# World Simulation and Evolution

> Design record migrated from [GitHub issue #367](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/367). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

The design follows #364 and the world model in #365. It should integrate with existing time, event, quest, faction, location, spatial-state, persistence, and inheritance systems rather than create an unrelated simulation engine.

## Goal

Create a persistent world that reacts to time and player actions, develops beyond the player's immediate presence, and preserves meaningful consequences across character lives.

**Core principle: simulate what the experience requires, at the fidelity it benefits from.** The architecture must support detailed and potentially extensive simulation where appropriate, while also supporting event-driven updates, aggregate models, and deferred catch-up where those approaches are sufficient.

Rust's performance and the project's architectural control make richer simulation a valid future direction. Performance should be measured and profiled rather than used as a blanket reason to rule out depth. Conversely, fast execution does not remove the need for coherent rules, bounded work, stable state, testability, or sensible simulation ownership.

The initial implementation should establish sound foundations and demonstrate them through a limited set of world changes. This staging keeps implementation practical; it is not a ceiling on future simulation.

## Existing foundation

The codebase already has:

- Authoritative world time in World and a shared advance_time path.
- Data-driven events with triggers, conditions, weighted selection, effects, and cooldowns.
- Persistent world history and location, faction, and NPC state.
- Generated faction relationships and generated-world evolution following quest completion.
- Save/load and character-inheritance behavior that distinguish world-persistent state from character-specific state.

The design should strengthen these foundations rather than replace them without cause. Current code contains some generated-world behavior stored in faction/NPC memory as free-text markers; new mechanical relationships should move toward structured state as the relevant systems are developed.

## 1. Simulation model: support multiple fidelity levels

The architecture should not impose a universal choice between a fully continuous simulation and an entirely event-driven world. Different domains may need different approaches, and the same domain may use different levels of detail depending on context.

### Detailed simulation

Use ongoing, entity-level updates where detailed behavior materially improves gameplay. Potential examples include:

- NPCs and creatures that move, perceive, pursue goals, or react to their surroundings.
- Active local environments, roaming threats, and spatial encounters.
- Interacting resource and production processes where intermediate steps affect player decisions.
- Tactical or social situations whose outcomes depend on sequences of actions rather than only their final state.

### Discrete or event-driven simulation

Use explicit triggers and state transitions when changes occur because of a known action, threshold, scheduled moment, or condition. Examples include:

- A player clears a dangerous location.
- A blockade is established or removed.
- A faction changes its stance following a defined event.
- A scheduled obligation becomes due.
- A temporary effect expires.

### Aggregate or deferred simulation

Use aggregate values or deferred processing for distant, inactive, or lower-detail systems when individual intermediate steps do not materially affect the eventual result. Examples could include regional resource recovery, background trade totals, or the elapsed duration of a temporary condition.

These are tools, not rigid distance-based tiers. An off-screen faction conflict may deserve detailed resolution if it has major consequences; a nearby decorative object may require almost no simulation. Relevance, gameplay purpose, and measured cost should determine fidelity.

Every simulation domain must declare what it guarantees when processed in detail, in aggregate, or after a delay. Deferred processing must preserve consequential state and any intermediate events the game promises the player, even when it does not reproduce every invisible micro-step.

## 2. Authoritative time and update boundaries

The existing discrete in-game clock remains authoritative for world time. The application being open, rendering frames, or real-world hours passing must not independently advance the world.

Player actions, travel, waiting, resting, and other defined activities advance game time through a shared time-management contract. Systems consume that progression according to their rules.

A domain may additionally require a simulation-step clock within an active real-time encounter or local map. Such a clock must have an explicit relationship to world time and must not advance global world state independently from gameplay rules.

The design must distinguish:

- **World time:** canonical in-game time used by travel, deadlines, day/night, cooldowns, and long-term consequences.
- **Simulation step:** a bounded update used by systems that need repeated state processing.
- **Presentation time:** renderer/animation timing, which must not directly own authoritative game rules.
- **Scheduling metadata:** the next time or condition at which a deferred state transition needs attention.

The implementation should centralize advancement and define ordering for actions, time progression, due transitions, event resolution, and presentation of resulting consequences. The order may differ where a particular action semantically needs a pre-action or post-action trigger, but that order must be explicit and tested.

## 3. Event-driven transitions and scheduled work

Triggered events and scheduled changes are complementary to ongoing simulation, not substitutes for every kind of behavior.

**Triggered events** respond to explicit gameplay conditions: discovery, travel arrival, combat outcomes, quest completion, resource extraction, changes in faction relations, and other defined actions.

**Scheduled transitions** become eligible after a world-time threshold or when a defined deadline is due: a temporary blockade expires, a resource site recovers, a commitment becomes due, or a settlement reaches a planned response point.

**Continuous or repeated updates** remain available to domains where intermediate simulation matters. They should use the appropriate simulation step and bounded processing model rather than being forced into one-shot events.

A transition must have identifiable preconditions, effects, and a stable identity where duplicate processing is possible. Repeated updates, save/load, revisiting a region, or creating an inherited character must not apply a one-time result twice.

When several changes become due together, resolve them in a stable, documented order, including dependencies between transitions. Random outcomes should use appropriate deterministic inputs so an unrelated new event does not unexpectedly alter all other outcomes.

A transition may schedule follow-up work, but chains must have explicit stopping conditions or bounded processing to prevent unbounded work from accumulated time.

## 4. Structured world state and causal changes

Simulation should operate on authoritative, structured state rather than treating narrative prose as the source of truth.

The design should distinguish:

- **Character-specific state:** personal reputation, conditions, progression, active activities, and other data governed by inheritance rules.
- **World-persistent state:** cleared threats, discovered routes, established settlements, lasting faction changes, important resource changes, world history, and other shared consequences.
- **Temporary world state:** blockades, temporary hazards, shortages, effects, and other conditions that have an explicit expiration or transition rule.
- **Derived state:** presentation summaries and values that can be computed from authoritative state when appropriate.

Not every location needs every possible field. State belongs to the domain that owns the mechanic. For example, route safety should be represented explicitly if it affects travel, not inferred solely from a location's description. Faction memories may explain a relationship, but future mechanical relationships should have structured identities and values.

New simulation state should have clear ownership, validation, persistence rules, and a defined relationship to related systems. It should not create separate parallel versions of a location, faction, route, or resource that can disagree with the authoritative world model.

## 5. Initial simulation domains

The first implementation scope should validate the simulation contract using domains that are already partly represented in the project.

### 5.1 Location conditions and dangers

Extend the existing dangerous-location and generated-world evolution behavior into explicit state transitions. Possible conditions include a threat being active or cleared, a route being open/threatened/blocked, a temporary hazard being active or expired, and a resource site being available or recovering.

Each condition must have valid transitions and defined causes. A cleared infestation should not automatically return solely because time passed unless a specific rule permits it.

### 5.2 Faction relations and influence

Factions can eventually react to changes in routes, resources, conflicts, settlements, and player actions. Relationship changes should follow identifiable reasons such as shared threats, resource competition, reopened trade, negotiated agreements, or significant events—not random attitude changes caused by elapsed time alone.

Structured faction relations should be distinct from character-specific reputation. A character's standing normally resets according to the established inheritance model, while the world's history and persistent political consequences remain.

The initial implementation does not require comprehensive territorial control or political simulation. The architecture must, however, allow more sophisticated faction behavior later.

### 5.3 Resources and economic consequences

Represent resource state when a gameplay mechanic depends on it: availability, depletion, renewal, ownership, or contest. Recovery rules should depend on resource type. A renewable gathering area might recover over time; a unique relic must not regenerate under a universal timer.

A complete economy with production, consumption, pricing, population needs, and logistics is a separate design. It is not required before the core simulation architecture works, but the architecture should not preclude implementing it later.

### 5.4 NPC and entity behavior

The architecture should support both stable/event-driven characters and entities with detailed goals, perception, movement, and schedules. Do not require full continuous simulation for every background NPC in the first milestone.

When an entity moves, changes role, or otherwise evolves, dependent dialogue, quests, location references, and interactions must remain consistent. Roaming entities and complex AI will receive dedicated design work.

### 5.5 Ecosystems and broader simulation

Wildlife populations, predator-prey behavior, migration, settlement economies, autonomous faction conflicts, and similar systems may justify richer ongoing or aggregate simulation later.

They are not prerequisites for the initial implementation. Their future design should define the expected player-facing experience, entity counts, time model, determinism requirements, and measurable performance budgets before implementation.

## 6. Active, inactive, and distant world regions

World topology from #365 means the world can grow as the player explores. Uncreated regions have no runtime entities to simulate and should not require background processing.

For created regions, simulation fidelity should be determined by domain requirements and relevance—not merely by distance from the player. The architecture may support:

- Full or high-detail updates for active spatial encounters and systems whose intermediate behavior is observable.
- Lower-frequency or aggregate updates for domains where an approximate model preserves the intended consequences.
- Deferred catch-up for inactive systems whose outcomes depend on elapsed time and stored state.
- Scheduled updates for specific future events or deadlines.

Switching fidelity modes must not cause state to reset, duplicate outcomes, or contradict already-established history. If a domain requires detailed intermediate steps to determine a fair or consequential outcome, that requirement must be respected even while inactive.

The project should profile real workloads before introducing elaborate scheduling, threading, or complex simulation infrastructure. Likewise, it should not hard-code a tiny world population based solely on assumptions about performance. Limits and budgets should follow measurement on representative desktop and Android hardware.

## 7. Visible consequences and history

Simulation results should affect what the player can observe or do: a route's availability, a location's state, an NPC's presence or dialogue, faction information, available resources, current opportunities, or encounter behavior.

Not every distant event requires an immediate notification. Abstract changes can be surfaced when the player revisits a location or asks for information. Important developments should be recorded in world history where appropriate.

History and current state serve different purposes:

- **History** explains what happened and preserves important past consequences.
- **Current state** determines what is true now and governs gameplay.

A history record that a bridge collapsed and was repaired does not determine whether the bridge is traversable today; the current route state does. Conversely, a current state change does not always need a verbose history entry if it is trivial or temporary.

Simulation systems should emit structured results or change records that presentation can display appropriately. Presentation must not implement authoritative rules.

## 8. Persistence and inheritance

The save system must store sufficient authoritative data to continue simulation consistently: world time, persistent state changes, relevant scheduled transitions or deadlines, cooldowns, stable identities, and duplicate-prevention markers where required.

On inheritance:

- World-level changes and important history persist.
- Character-specific reputation and progression follow their established inheritance rules.
- Temporary world states continue or expire according to elapsed in-game time and their explicit rules.
- Scheduled work and one-time transitions are not replayed just because a new character is created.
- Established region, location, route, and local-map state must not be regenerated from seed and overwrite later changes.

Older saves require conservative migration with safe defaults. Missing new fields must not result in the established world being regenerated or rearranged.

Event definitions can be rehydrated from the current content package where appropriate, while runtime-created events and world state require their existing explicit persistence path. Content updates must not silently erase established world consequences.

## 9. Performance, determinism, and engineering quality

- **Profile before restricting scope.** Use representative workloads and target-device measurements to identify real bottlenecks before reducing simulation fidelity.
- **Keep ownership explicit.** Each domain defines its state, update method, time semantics, and transition rules.
- **Keep deterministic behavior where practical.** World generation and simulation outcomes should be repeatable from stable state and inputs, with randomness isolated appropriately.
- **Bound deferred work.** Catch-up must handle long elapsed intervals and transition chains without unbounded work.
- **Preserve causal correctness.** Deferred or aggregate updates must not skip consequences the domain promises to model.
- **Test order independence where required.** Unrelated systems should not change outcomes simply because they ran in a different collection order or an irrelevant system was added.
- **Avoid premature threading.** Begin with understandable, testable rules and introduce parallel execution only where data ownership and measured performance justify it.
- **Preserve the gameplay/presentation separation.** Bevy frames, animations, and UI interactions consume authoritative simulation results rather than deciding them.

## Implementation sequence

1. **Define the simulation contract:** authoritative world time, update boundaries, transition ordering, stable identities, and ownership of current state.
2. **Formalize location transitions:** explicit persistent and temporary conditions, valid state changes, and their causes.
3. **Centralize world-time processing:** integrate existing time advancement, events, cooldowns, and generated-world evolution.
4. **Add scheduling and deferred updates:** support due transitions, bounded catch-up, deterministic resolution, and duplicate prevention.
5. **Integrate player-driven consequences:** quests, encounters, route changes, and other actions update authoritative state and record meaningful outcomes.
6. **Introduce limited faction/resource evolution:** only rules supported by the current gameplay, with structured persistent state.
7. **Profile and expand simulation fidelity:** measure real workloads and add richer NPC, entity, economic, ecological, or political simulation where it improves the experience.

Each stage should have its own tests and acceptance criteria. The staged sequence is an implementation strategy, not a requirement to keep the eventual simulation small.

## Acceptance criteria

- World changes are driven by in-game time, explicit gameplay actions, defined schedules, or appropriate ongoing simulation—not real-world waiting or rendering frames.
- Detailed, event-driven, aggregate, and deferred approaches can coexist without duplicating authoritative state.
- Every simulation domain defines its state ownership, update semantics, persistence requirements, and guarantees when processed at lower fidelity.
- Deferred processing preserves promised consequential state and intermediate events; continuous simulation remains available where intermediate behavior matters.
- One-time transitions cannot be applied twice through repeated processing, save/load, region re-entry, or character inheritance.
- Scheduled changes and accumulated time can be processed with bounded work and stable ordering.
- Route, location, resource, NPC, and faction state remain consistent with dependent interactions and descriptions.
- Character-specific reputation and world-persistent political consequences remain distinct.
- Current state and important history remain consistent but serve separate roles.
- Current event definitions and generated-world evolution are integrated through the shared contract.
- Existing saves, established world state, routes, local maps, and history remain intact through conservative migration.
- Tests cover determinism, transition ordering, duplicate prevention, catch-up, persistence, inheritance, and domain-specific consequences.
- Performance decisions are supported by profiling on representative desktop and Android workloads; the design does not arbitrarily rule out richer simulation in advance.

## Details intentionally left for dedicated designs

This issue does not yet define complete resource economies, full NPC schedules and goal systems, autonomous faction warfare, ecology, or every entity's AI. Each deserves its own design if and when its gameplay value justifies the work.

Those designs should build on this simulation contract and specify the desired player-facing behavior, required state, causal rules, fidelity requirements, persistence semantics, test strategy, and measured performance budget.

The objective is a coherent architecture capable of supporting a rich living world, with implementation staged to establish correctness and playable value at every step.
