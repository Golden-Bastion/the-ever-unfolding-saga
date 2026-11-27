# Simulation Processor Architecture

> Status: approved target design. This document records the agreed internal execution architecture for simulation domains. It refines [World Simulation Architecture](world-simulation-architecture.md) without claiming that the described processor framework exists in the current runtime.

## Purpose and relationship to domain ownership

Simulation domains own authoritative state, governing rules, persistence schemas, and the interfaces through which other domains interact with them. Processors are the meaningful units of work used to execute those rules.

The architecture is **domain-owned and processor-oriented**:

- Domains define ownership boundaries, contracts, and the operations they expose.
- A domain may contain multiple focused processors.
- Enpicieus coordinates processor execution globally, resolving declared dependencies and scheduling independent work.
- Processors do not gain authority over another domain's state merely because they need to read or influence it.

This avoids both rigid domain silos and a processor graph with unclear state ownership. A domain remains the owner even when its work is executed by several processors or coordinated with processors from other domains.

## Processor boundaries

A processor represents a meaningful unit of simulation work with identifiable inputs, outputs, and responsibility.

Split work into separate processors when doing so provides a concrete benefit, such as:

- Independent scheduling or parallel execution.
- Selective execution when only some inputs are relevant.
- A distinct update cadence, fidelity policy, or resource profile.
- A clearer ownership or maintenance boundary.
- The ability to suspend, resume, or prioritize the work independently.

Keep tightly coupled operations together when splitting them would add coordination overhead without a corresponding benefit. Avoid creating tiny processors merely to maximize the processor count.

The initial processor set and exact boundaries are implementation decisions. Examples such as NPC needs, routines, relationships, memory, and population changes illustrate possible responsibilities, not a mandatory decomposition.

## Execution triggers

Processor execution is selected per processor. The architecture supports:

- **Event-driven execution:** run when relevant state changes or an event is delivered.
- **Scheduled execution:** run when a task, deadline, or defined world-time condition becomes due.
- **Periodic execution:** run at a declared interval when recurring evaluation is required.
- **Hybrid execution:** combine triggers when a processor has more than one legitimate reason to run.

There is no universal requirement that every processor run every simulation tick. Enpicieus coordinates triggers, dependencies, relevance, fidelity, and resource budgets. Processors should avoid redundant work when none of their relevant conditions require evaluation.

## Communication and state access

No single communication mechanism is mandatory. Use the mechanism that fits the consistency and scheduling requirements:

- Direct domain queries for information needed immediately.
- Domain operations for controlled changes to state owned by that domain.
- Events for immutable facts that have occurred and may be consumed asynchronously.
- Queued commands or scheduled tasks for work that should be performed later or at a defined point.

Batch reads and writes where practical, particularly on hot simulation paths, to avoid excessive fine-grained cross-domain overhead.

Processor dependencies are declared through domain or interface contracts and resolved by Enpicieus. Direct coupling to another processor's private implementation is discouraged and requires a clear reason. The owning domain remains the only authority allowed to mutate its authoritative state.

## Authoritative, derived, and temporary state

Authoritative persistent state belongs to its owning domain. Processors may also maintain:

- Temporary execution state.
- Derived data, indexes, aggregates, and caches.
- Continuation state needed to resume work that spans simulation steps.

Derived data and caches must be rebuildable from authoritative state. Cache invalidation and synchronization must be explicit. They must never silently become competing sources of truth.

Processor-local temporary state does not automatically become save data. Persist continuation state only when it is necessary to preserve correct, deterministic progress across a save/load boundary or another lifecycle interruption. Domains remain responsible for serializing authoritative state; processors rebuild or restore their temporary state through defined lifecycle contracts.

## Execution consistency and concurrency

Enpicieus schedules independent processors concurrently and orders dependent or conflicting work according to declared contracts.

Processors should work against well-defined input snapshots or read boundaries and expose results through controlled commit points. Partially applied updates must not become visible as completed authoritative state. The concrete snapshot and commit mechanism remains an implementation decision; it must preserve the established domain ownership and deterministic simulation contracts.

The same initial authoritative state, inputs, and simulation steps must produce the same authoritative result regardless of worker count, thread scheduling, or how eligible work is divided across simulation steps.

Legitimate shared-state operations require explicit deterministic coordination. Conflicting writes must not be resolved by whichever thread happens to finish first.

## Suspension and continuation

Processors may suspend and resume work across simulation steps when a workload cannot or should not complete within the current budget.

- A processor defines safe suspension points.
- Continuation state identifies the unfinished work and the deterministic position from which it resumes.
- Suspending work must not expose an inconsistent partial authoritative update.
- Dividing work across different numbers of simulation steps must not change its eventual authoritative result.
- Temporary continuation data remains transient unless it must survive a save/load or lifecycle boundary.

Enpicieus may defer work according to priority and budget, but deferral must not silently discard required authoritative work.

## Fidelity and resource budgets

Each processor declares the fidelity levels it supports, the guarantees of each level, and the valid transitions between them. Enpicieus applies global fidelity policy according to relevance and resource availability; it does not need to understand each processor's internal algorithm.

Processors should declare estimated resource costs and requirements. Enpicieus may use observed costs and workload data to improve allocation. Global and platform-specific safety ceilings constrain execution. When work exceeds the available budget, lower-priority work is deferred or processed at an allowed fidelity rather than creating unbounded queues or starving critical work.

Budget availability and fidelity transitions may change when work is performed or how much detail is used, but must not violate declared guarantees or change eventual authoritative outcomes. Catch-up follows the deterministic contracts in [World Simulation Architecture](world-simulation-architecture.md).

## Registration and lifecycle

Built-in domains and their built-in processors are registered statically. Supported extensions may register through explicit runtime extension interfaces.

Enpicieus coordinates processor initialization, activation, suspension, resumption, and shutdown. Runtime registration does not imply unrestricted access to authoritative state or arbitrary execution privileges. Extensions remain subject to dependency, determinism, persistence, resource, and compatibility contracts.

Processor interfaces should be explicit and versionable. The initial extension model does not promise unrestricted runtime code injection or arbitrary mod-defined authoritative domains.

## Failure handling

A processor failure should be isolated where the remaining simulation can continue coherently. Failures must be reported with enough context to diagnose the affected processor, domain, phase, and work item.

Essential authoritative work must not be silently skipped or treated as successful after failure. Enpicieus and the owning domain must apply an explicit recovery, retry, defer, disable, or stop policy appropriate to the failure and the affected state. Recovery must preserve domain invariants and deterministic outcomes; when that cannot be guaranteed, fail the affected operation or simulation step rather than fabricating success.

The precise retry and recovery policy is domain- and operation-specific. A universal catch-and-continue rule is not acceptable.

## Diagnostics and validation

Developer diagnostics should make it possible to identify:

- Processor execution time and resource consumption.
- Work deferred or suspended by budget limits.
- Dependency bottlenecks and processors waiting on prerequisites.
- Fidelity levels and transitions.
- Failures, retries, and recovery decisions.
- Workload and scheduling behavior across simulation steps.

Diagnostics must not impose unnecessary cost on normal gameplay. Testing should cover processors independently, interactions between domains, and complete deterministic simulation runs. Determinism tests should vary worker counts, execution scheduling, and budget-driven work division while comparing authoritative outcomes. Platform-specific execution strategies may differ, but platform differences must not change authoritative results.

## Decisions intentionally left open

This specification does not choose:

- The concrete processor trait/API, registration format, or dependency declaration syntax.
- The exact initial processor inventory and boundaries for each domain.
- The threading library, task executor, worker-pool design, or data-parallel framework.
- The concrete snapshot, write-buffer, transaction, and commit implementation.
- Cache/index invalidation algorithms and ownership details.
- The format of continuation state or which workloads require persistence across save/load.
- Resource-budget units, cost-estimation algorithms, and profiling implementation.
- Exact retry limits, failure classifications, or domain-specific recovery policies.
- The initial mod extension API and its capability restrictions.

These are later technical design decisions. They must preserve the ownership, determinism, consistency, scheduling, and recovery guarantees established here.
