# Simulation Domains and Shared Infrastructure

> This specification records the mechanics-first decisions from the world-simulation design workshop. It describes the intended gameplay scope and architectural direction; it does not claim that these systems are implemented. It supplements [World Simulation and Evolution](world-simulation.md) and [World Simulation Architecture](world-simulation-architecture.md).

## Design approach

Simulation domains are identified from the gameplay mechanics the world must support, then organized around clear state ownership and responsibilities.

Begin with broad domains rather than splitting every mechanic into a specialized processor. A domain may be divided later when its responsibilities, state ownership, scheduling needs, persistence, or extension requirements justify an independent boundary.

The areas below are initial organizational groupings, not a permanent or exhaustive list. A gameplay mechanic may involve multiple domains without requiring a separate domain of its own. Shared infrastructure is also a valid architectural category; not every shared capability needs to become a simulation domain.

## Initial gameplay areas

### 1. Geography and environment

The world simulation should support:

- Terrain, geology, and climate.
- Weather and seasonal changes.
- Rivers, lakes, groundwater, and other water systems.
- Vegetation and ecological changes.
- Wildlife populations and behavior.
- Natural hazards and environmental events.
- Environmental effects on movement, travel, health, and survival.

The eventual implementation may combine generated or derived information with scheduled, aggregate, or ongoing simulation. This scope does not require every environmental process to be continuously simulated.

### 2. Settlements and economies

The world simulation should support:

- Settlement growth and decline.
- Population needs.
- Production and resource distribution.
- Markets and prices.
- Trade routes and merchants.
- Employment and infrastructure.

The detailed economic model and the fidelity required for different settlements remain subjects for later design.

### 3. NPCs and populations

The world simulation should support:

- Individual needs, routines, work, and daily activities.
- Relationships, families, births, and deaths.
- Migration and population growth or decline.
- Skills, health, and personality.
- Memory, knowledge, and awareness.
- Reactions to events and changing circumstances.

Individual NPCs and aggregate populations may use different simulation fidelity levels. They need not receive identical processing detail, provided shared world time and authoritative outcomes remain consistent.

### 4. Factions and organizations

This area covers organizations broadly, not only political factions. Examples include:

- Political factions, kingdoms, and governments.
- Guilds, merchant companies, and trade organizations.
- Religious institutions and movements.
- Military organizations and mercenary companies.
- Criminal organizations.
- Leadership, succession, internal politics, and membership.
- Alliances, rivalries, diplomacy, territorial influence, and conflicts.
- Recruitment, loyalty, and organizational stability.

Different organization types use the mechanics relevant to them; they should not all be forced into a political-faction model.

### 5. World events

World events cover developments that may affect multiple domains, including:

- Wars and other large-scale conflicts.
- Disasters and natural crises.
- Political upheavals.
- Economic crises.
- Migrations.
- Epidemics.
- Discoveries and other consequential developments.

World events connect domains and communicate that developments have occurred. They do not take ownership of the underlying mechanics or authoritative state involved.

## Cross-domain mechanics

Some mechanics naturally involve multiple areas. They should initially remain with the domains responsible for their underlying state and rules rather than being split into new domains solely because they cross boundaries.

Examples include:

- **Crime and law:** Interactions among individuals, settlements, and organizations. Local customs, authorities, and circumstances determine responses.
- **Reputation and social standing:** Perceptions and relationships held by individuals and organizations, rather than one universal reputation value.
- **Conflict:** Personal, military, political, and economic conflicts handled by the relevant domains.

These groupings may be revisited if a concrete need establishes a genuinely independent responsibility.

## Shared information and knowledge infrastructure

Information handling is a shared capability used by simulation domains. It is not initially a separate player-facing gameplay system or a mandatory detective subsystem.

Its basic principles are:

1. **Knowledge is not omniscient.** An entity acts on information it has acquired, not automatically on every authoritative fact in the world.
2. **Information has sources.** Entities may learn through observation, participation, conversations, reports, records, or other established means.
3. **Transmission uses explicit mechanisms.** Information spreads through actual communication or defined channels; it is not automatically copied to every nearby entity or organization.
4. **Information may be incomplete or incorrect.** Entities can form beliefs and make decisions based on mistaken or partial information.
5. **Organizations can retain collective knowledge.** Information becomes available to an organization when it is communicated into organizational channels or preserved in records. Such knowledge can outlive the members who originally acquired it. Information held only in an individual's memory may be lost when that individual is gone.

The infrastructure supports the simulation behind the scenes. Its effects should emerge through NPC behavior, dialogue, decisions, organizational responses, and world events. It does not require a dedicated information-management interface, rumor dashboard, or investigation gameplay loop.

The detailed representation of reliability, communication protocols, memory decay, and investigation procedures is intentionally unresolved. Those details should only be designed when a concrete gameplay requirement justifies them.

## Scope and implementation boundaries

- These areas describe desired gameplay capabilities, not a commitment to simulate every listed process continuously.
- Domain boundaries, internal processors, and implementation order remain open to refinement.
- Shared infrastructure should be introduced where multiple domains need a common capability, without turning every shared concern into a new domain.
- More specialized domains may emerge when real responsibilities and implementation needs justify the split.
- Implementation should be guided by concrete gameplay scenarios, authoritative state ownership, deterministic outcomes, persistence needs, and measured performance on supported platforms.

This specification defines the current scope and direction. It does not prescribe data structures, APIs, algorithms, or a complete implementation plan.
