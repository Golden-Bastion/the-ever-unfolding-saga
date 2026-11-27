# World Simulation Architecture

> This specification records the simulation-architecture decisions agreed in the world-simulation design workshop. It supplements [World Simulation and Evolution](world-simulation.md), which defines the gameplay goals, simulation domains, persistence principles, and fidelity model. Where this document makes a more specific architectural decision than the general guidance in that document, this specification governs. These are target architectural decisions, not claims about the current runtime implementation.

## Architectural direction

TEUS will use a **centralized authoritative world-state model with modular domain processors**.

The central model is the coherent authoritative representation of world state. It is not a single monolithic simulator, and it does not mean every system may freely mutate every field. Specialized domain processors own the rules and controlled write access for the state within their responsibility.

The current five-domain responsibility map is defined in [Simulation Domains and Shared Infrastructure](world-simulation-domains.md). That specification establishes gameplay scope and ownership boundaries. The agreed internal execution model—including processor boundaries, triggers, communication, temporary state, suspension, fidelity, lifecycle, and diagnostics—is defined in [Simulation Processor Architecture](simulation-processor-architecture.md); concrete APIs and implementation details remain subject to later technical design.

Persistent entities retain stable identities and authoritative state independently of regions, chunks, loading, and presentation. Domain processors manage the rules that update that state. Enpicieus coordinates simulation execution and relevance; it does not replace domain logic or become the owner of every domain's state.

## 1. State ownership and mutation

- The world has one coherent authoritative state model rather than separate competing copies of the same world facts in individual processors.
- Each domain has explicit ownership over the state and rules it is responsible for.
- A domain may update its owned state through controlled write interfaces.
- Other domains use exposed queries and operations rather than reaching into another domain's internal representation or freely mutating its state.
- Ordinary trusted mutations do not require universal validation or a central approval step.
- Add validation or stronger gates at specific boundaries where they are justified: for example, transactions, invariants, resource constraints, player-controlled inputs, persistence boundaries, or known sources of invalid state.
- Cross-domain operations must preserve coherent outcomes without requiring every individual mutation to pass through one global transaction pipeline.

This is centralized authoritative state with distributed domain responsibility, not a giant mutable data structure with unrestricted access.

## 2. Processor execution and deterministic parallelism

Simulation execution uses a **hybrid phase model**:

- Enpicieus coordinates ordered simulation phases.
- Dependencies determine which processors or operations must run before others.
- Independent work within a phase may run in parallel.
- Work with dependencies or conflicting writes is ordered or explicitly coordinated.
- The execution model must not rely on a particular thread completing first.

Authoritative simulation is strictly deterministic with respect to execution scheduling and worker count: given the same initial authoritative state, inputs, and simulation steps, it must produce the same authoritative result regardless of thread scheduling or the number of workers.

The implementation must therefore provide deterministic handling for:

- Dependency and phase ordering.
- Conflicting writes and shared-resource operations.
- Event delivery and event processing.
- Scheduled work and catch-up.
- Random-number streams and their state.
- Any other operation whose ordering can affect authoritative results.

This requirement does not prescribe a particular threading library or data-parallel framework. It also does not, by itself, establish a cross-platform bit-identical floating-point contract; numeric representation and reproducibility requirements that need that stronger guarantee require a dedicated technical decision.

## 3. Dependencies and write conflicts

Domains declare dependencies where execution order affects correctness. Enpicieus uses those declarations to establish a valid execution order while allowing unrelated domains to proceed independently.

Circular dependencies must be surfaced as architectural problems rather than hidden through arbitrary ordering.

Conflicting writes are **prevented by default** through state ownership and dependency scheduling. When a legitimate operation requires multiple processors or actors to compete for shared state, provide an explicit deterministic coordination mechanism.

For example, two simultaneous purchases cannot both consume the same limited stock. The coordination rule must determine the result consistently, rather than allowing thread timing to decide.

## 4. World time, scheduling, and catch-up

There is one authoritative world clock shared by every region, entity, and simulation domain. Regions are parts of one world, not separate time domains. Differences in simulation fidelity or activity must never cause different places to experience different amounts of elapsed world time.

The world clock advances through the established gameplay time contract. Simulation domains may use bounded internal steps where needed, but such steps remain subordinate to world time and cannot independently advance global time.

The time model is hybrid:

- The world clock provides a consistent authoritative timeline.
- Scheduled tasks identify future work and deadlines.
- Domains may use repeated or detailed updates where intermediate behavior matters.
- Event-driven and deferred processing avoid unnecessary work when continuous updates are not required.

When an entity or system has been inactive, catch-up is also hybrid:

- Process important scheduled events and deadlines.
- Preserve and process intermediate steps when the domain's guarantees or consequences require them.
- Resolve ordinary background changes through deterministic aggregate updates where individual steps do not matter.
- Preserve meaningful causal consequences; aggregation must not erase consequential events such as famine, rebellion, leadership changes, or other developments the simulation promises to represent.

Catch-up must be bounded and deterministic. Its authoritative result must be consistent with the same elapsed world time being processed through the domain's declared update model. Domains must specify which intermediate events they preserve and which routine changes may be aggregated.

## 5. Events and scheduled tasks

Events and future work are separate concepts, coordinated by a shared scheduling architecture.

- **Events** are immutable records that a fact or occurrence has happened. They may be consumed by other domains and retained in history where significant.
- **Scheduled tasks** represent work intended to occur at a future world time or when a defined condition is met. A task is not a historical fact merely because it has been scheduled.
- A task may produce one or more events when it executes.

Event queues are owned by their respective domains. Enpicieus coordinates cross-domain delivery and processing through shared event contracts and deterministic ordering. This avoids one unrestricted global event stream while allowing domains to communicate consistently.

Scheduled tasks must preserve stable identity, due time or condition, relevant execution state, and duplicate-prevention information across save/load and catch-up. Event processing and task execution must not duplicate outcomes.

## 6. Historical retention

History is distinct from current authoritative state. The world should retain important historical events indefinitely where they explain lasting consequences, identity, or the world's past.

Routine events may be compacted into summaries or discarded once their consequences are safely represented in persistent current state. For example, a founding or assassination may remain a permanent historical record, while routine inventory changes need not each become permanent history.

Compaction must not remove causal information that the game promises to preserve. Current state remains authoritative for what is true now; history explains what happened.

## 7. Domain extensibility and registration

The initial architecture uses built-in simulation domains with explicit extension interfaces. It does not attempt to support arbitrary third-party simulation architectures from the outset.

Initially, supported mod extensions may add behavior through established contracts, including extending existing entity types and using exposed state, events, and scheduled tasks. Mods are not initially promised unrestricted authority to introduce arbitrary authoritative entity types or wholly independent simulation domains.

This is a scope decision, not a permanent prohibition. More powerful mod-defined entity types or domains may be introduced later if concrete modding needs justify the infrastructure and compatibility cost.

Built-in domains are registered statically. Supported extensions may register at runtime through defined interfaces. Runtime registration does not imply unrestricted access: extensions remain subject to the supported dependency, execution, determinism, persistence, and compatibility contracts.

## 8. Versioning and load failure

Each domain owns and versions its own persistent state schema. A world-level save version coordinates the complete save format and the migrations or compatibility checks needed to load its constituent domains.

A change to one domain's schema should not require unrelated domains to change their own schema versions. Domain migrations remain coordinated within the world save lifecycle.

Loading uses failure isolation with consistency safeguards:

- If essential authoritative state cannot be recovered, loading the world fails rather than silently discarding or fabricating that state.
- An optional extension that cannot load may be isolated or disabled when its dependencies allow the rest of the world to remain coherent.
- A failed domain's dependents must be handled according to declared dependency requirements.
- Recovery or disabling must be explicit and reported. Saved data must not be silently discarded.
- The original save must remain recoverable.

## 9. Resource budgets and work prioritization

Enpicieus manages a global simulation resource budget and allocates it among domains. Each domain may also define its own limits, priorities, and workload characteristics.

Budgets use a hybrid policy:

- Platform-specific safety ceilings constrain total resource use.
- Allocation adapts within those ceilings according to available capacity and current workload.
- A domain cannot consume unlimited resources or starve critical simulation work.
- Deferring work changes when it is processed, not the authoritative outcome it must eventually produce.

Simulation priority is based on gameplay relevance, not distance alone. Relevant factors include active encounters and threats, visibility and interaction, important NPCs, scheduled deadlines, dependencies, predicted movement, and the consequences of allowing work to wait. Distance may contribute, but does not determine priority by itself.

Budget allocation and prioritization must remain compatible with strict determinism. Adaptive resource availability may change which work is deferred, but it must not change authoritative results. Deferred work must be completed through the deterministic catch-up contract.

## 10. Simulation fidelity ownership

Enpicieus controls the global fidelity policy and coordinates when domains should receive more or less processing according to relevance and resource constraints.

Each domain defines the actual fidelity levels it supports, the guarantees of each level, and the valid transitions between them. Enpicieus does not need to understand each domain's internal simulation algorithm.

A domain must declare what it preserves at each fidelity level, including intermediate events or state transitions that cannot safely be aggregated. Moving between fidelity levels must not reset state, duplicate outcomes, or contradict established history.

Spatial loading, simulation activity, and rendering remain independent decisions, as established in the world and spatial architecture specifications.

## 11. Responsibilities of Enpicieus

Enpicieus coordinates rather than owns every simulation concern. Its responsibilities include:

- Advancing and coordinating the global simulation schedule against authoritative world time.
- Establishing ordered phases from declared dependencies.
- Coordinating parallel execution of independent work.
- Coordinating domain-owned event queues and scheduled tasks.
- Managing global resource budgets, domain allocations, and relevance-based prioritization.
- Applying global fidelity policy while respecting domain-defined fidelity contracts.
- Supporting deterministic catch-up and consistent execution.

Domain processors remain responsible for their own simulation rules, state ownership, supported fidelity levels, and domain-specific persistence behavior. Enpicieus must not maintain parallel authoritative copies of domain state.

## 12. Decisions intentionally left open

These decisions establish the architecture but do not select implementation details that have not yet been designed. The following remain open for later technical work:

- Concrete domain APIs, processor APIs, and the initial set of implemented processors. The agreed responsibility boundaries are defined in [Simulation Domains and Shared Infrastructure](world-simulation-domains.md), while processor execution contracts are defined in [Simulation Processor Architecture](simulation-processor-architecture.md).
- The concrete world-state data layout and controlled write-interface design.
- The simulation phase representation and dependency declaration format.
- The deterministic parallel execution strategy and conflict-coordination primitives.
- Event and task queue data structures, ordering keys, retention, and delivery guarantees.
- Random-stream partitioning and numeric reproducibility requirements.
- The exact save container, domain schema identifiers, and migration protocol.
- Extension API shape, capability limits, and compatibility policy.
- Resource-budget units, measurement strategy, and platform ceilings.
- The concrete fidelity levels and transitions supported by each domain.

These are implementation and subsystem design questions. They must respect the decisions above rather than reopen them implicitly.
