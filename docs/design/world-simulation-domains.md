# Simulation Domains and Shared Infrastructure

> This specification records the agreed gameplay scope and responsibility boundaries for TEUS world simulation. It describes intended capabilities and architecture; it does not claim that these systems are implemented. It supplements [World Simulation and Evolution](world-simulation.md) and [World Simulation Architecture](world-simulation-architecture.md).

## Design approach

Simulation domains are organized around coherent gameplay responsibilities and authoritative state ownership. Start with broad domains and split them only when complexity, independent responsibilities, scheduling, persistence, or extension needs justify a separate boundary.

A gameplay feature may involve several domains without requiring a domain of its own. Shared infrastructure is a separate architectural category; not every shared capability needs to become a simulation domain.

The current domain map has five broad domains:

1. Geography and environment.
2. Settlements and economies.
3. NPCs and populations.
4. Factions and organizations.
5. World events.

These are responsibility boundaries, not rigid feature silos. The domains may contain multiple processors, and their internal implementation remains subject to later design.

## Ownership principles

- Each piece of authoritative state and the rules that govern it have one owning domain or system.
- A domain updates its own state through controlled interfaces.
- Other domains interact through exposed queries, operations, and events rather than freely mutating internal state or maintaining competing authoritative copies.
- A feature may coordinate several domains, but coordination does not transfer ownership of their underlying mechanics.
- Shared infrastructure provides common capabilities without becoming an owner of every domain's gameplay state.
- Avoid creating a new domain solely because a mechanic crosses existing boundaries.

This is a rule of clear ownership, not a requirement for universal validation or a global approval pipeline. Add stronger transaction or validation boundaries only where concrete invariants require them.

## Simulation domains

### 1. Geography and environment

Owns the physical world and natural processes.

- Terrain, geology, landforms, climate, weather, and seasonal conditions.
- Rivers, lakes, groundwater, and other water systems.
- Vegetation, ecosystems, habitats, and ecological changes.
- Ordinary wildlife and creature populations as part of their ecological context.
- Natural hazards and environmental events.
- Natural resource deposits, distribution, availability, and regeneration.
- Physical and environmental constraints affecting movement, traversal, and travel.
- Environmental effects on health and other gameplay systems.
- Magical or supernatural phenomena that are part of the environment or its ecology.

This domain owns environmental facts and processes, not the economic activity that uses them. For example, it owns the presence and renewal of a resource deposit; the domain or actors responsible for extraction, production, and trade own those activities.

Environmental processes need not all run continuously. Use generated, derived, scheduled, aggregate, or detailed simulation according to their gameplay relevance.

### 2. Settlements and economies

Owns organized habitation, economic activity, and the services and infrastructure that support them.

- Settlement structure, development, growth, and decline.
- Economic conditions and population-facing services.
- Production, crafting, workshops, manufacturing, and resource distribution.
- Markets, prices, trade, merchants, and commercial relationships.
- Employment, services, livelihoods, and ways for people to earn a living.
- Property and business ownership, including player-owned property and businesses.
- Infrastructure such as roads, bridges, ports, buildings, and other public or commercial works.
- Transportation services and the economic/logistical movement of goods and passengers.
- Economic use of natural resources and the consequences of extraction, scarcity, and distribution.

This domain owns settlement and economic structures, not every person who lives or works in them. NPCs and organizations provide the actors; their individual or collective state remains with those domains. Construction, ownership, operation, and maintenance may involve organizations and individuals while the resulting infrastructure and economic role remain represented here.

Trade routes, caravans, shipping, passenger services, and other logistics use this domain together with geography, NPCs, and organizations. They do not require a separate transportation domain.

### 3. NPCs and populations

Owns individual people and creatures, as well as population-level behavior.

- Individual needs, routines, work, daily activities, and personal livelihoods.
- Skills, expertise, personality, capabilities, and individual development.
- Health, injuries, illnesses, disability, and recovery.
- Memory, knowledge, awareness, beliefs, and individual decision-making.
- Relationships, friendships, romance, families, marriage, parenthood, births, and deaths.
- Learning, education at the individual level, and personal cultural practices.
- Migration, population growth and decline, and other demographic changes.
- Individual creature behavior, intelligence, and social behavior where applicable.
- Reactions to events, changing circumstances, and information acquired.

Individual NPCs and aggregate populations may use different fidelity levels. Not every person or creature needs continuous detailed simulation. Intelligent creatures can use relevant NPC and organization mechanics without being forced into a human model.

Population-level needs and demographic changes belong here; settlements and economies handle the services, resources, and economic structures that respond to them.

### 4. Factions and organizations

Owns organized groups, institutions, and their collective activity. This includes more than political factions:

- Kingdoms, governments, and political factions.
- Guilds, merchant companies, and trade organizations.
- Religious institutions and movements.
- Military organizations and mercenary companies.
- Criminal organizations.
- Other social, cultural, professional, or creature organizations where the world supports them.

Responsibilities include:

- Leadership, succession, membership, recruitment, and internal politics.
- Organizational goals, resources, stability, and collective decisions.
- Alliances, rivalries, diplomacy, territorial influence, and conflicts.
- Loyalty, cooperation, assignments, and organizational structure.
- Institutional knowledge, records, and communication through organizational channels.
- Organized work such as trade, construction, services, and expeditions.

Military activity belongs here rather than in a separate warfare domain. The scope includes recruitment, command structures, supplies and logistics, campaigns, battles, sieges, occupation, and the organizational consequences of warfare. These mechanics should support a living adventure world, not require the player to manage kingdoms or entire armies as in a grand-strategy game.

Different organization types use the mechanics relevant to them; they should not all be forced into a political-faction model.

Organizations may initiate expeditions and invite the player to participate as an activity or quest. Expeditions that do not involve the player may still happen and be resolved abstractly through the existing simulation, using relevant participants, capabilities, preparation, conditions, and deterministic randomness. They do not require a separate exploration domain or a fully rendered simulation of every expedition.

### 5. World events

World Events is a lightweight domain for significant developments that span or connect other domains. It has meaningful but limited persistent state.

It may own a development's identity, status, timeline, major milestones, and historical record. Examples include an ongoing war, a major disaster, a political crisis, an economic collapse, a large migration, an epidemic, or a consequential discovery.

It does **not** own the mechanics that cause or sustain those developments:

- Organizations conduct wars and campaigns.
- NPCs and populations fight, flee, migrate, recover, and react.
- Settlements and economies experience shortages, disruption, and recovery.
- Geography and environment determine relevant physical conditions and hazards.
- Other domains retain their own authoritative state and rules.

World Events connects significant developments and their cross-domain consequences without taking over those systems. Not every routine occurrence needs to become a world event. Its persistent development records are distinct from the shared event-delivery infrastructure described below.

## Cross-domain mechanics

The following are gameplay mechanics supported by the existing domains, not additional simulation domains.

- **Crime and law:** Individual actions and reactions, local customs and settlement conditions, and the authority or interests of relevant organizations. Responses depend on circumstances and knowledge; there is no mandatory universal prison, trial, or sentencing simulation.
- **Reputation and social standing:** Perceptions and relationships held by individuals and organizations, not one universal reputation value.
- **Conflict:** Personal, military, political, and economic conflict handled by the domains that own the relevant actors and state.
- **Religion and belief:** Institutions and movements belong to organizations; individual beliefs and practices belong to NPCs and populations.
- **Culture and traditions:** Practices are carried by communities and populations; organizations may preserve, promote, or challenge them.
- **Education and knowledge:** Individual learning and expertise belong to NPCs; mentors, schools, guilds, libraries, and other institutions provide ways to teach and preserve knowledge.
- **Health and medicine:** Individual health belongs to NPCs; settlements and economies provide access, facilities, and supplies; organizations may provide practitioners and organized care; environmental conditions affect health.
- **Exploration and discovery:** Primarily player-facing spatial gameplay, with organizations able to initiate expeditions. Unrendered expeditions may be resolved abstractly by the existing simulation. Discoveries become world facts that entities can learn and use.
- **Resource acquisition:** Mining, logging, hunting, fishing, gathering, and similar activities use geography and environmental resource state together with the NPCs, organizations, and economic systems performing or using the work.
- **Creatures and monsters:** Use NPC, population, ecological, and organizational mechanics as appropriate to their capabilities and nature.
- **Magic and supernatural activity:** Integrated into the domains where the relevant actors, phenomena, institutions, or consequences belong. Magic is not automatically a separate simulation domain.
- **Survival needs:** Food, water, shelter, fatigue, exposure, and similar needs are supporting mechanics, not a survival-game pillar or separate domain. Include them when they create meaningful decisions or consequences; do not turn them into repetitive maintenance chores that interrupt gameplay unnecessarily.

## Player-facing systems

The player participates in the same world systems as other actors where appropriate. A separate player-facing system may own character-specific progression and presentation without duplicating the world's domain state.

- Character progression, attributes, skills, and personal capabilities remain part of character systems.
- Combat and personal activities remain player-facing gameplay systems, while using NPC, organization, environmental, and world-event state where relevant.
- Personal relationships and family gameplay use the same underlying NPC and population mechanics, without requiring a comprehensive relationship simulator.
- Player-owned property and businesses use the same settlement and economy mechanics available to other actors.
- Player death, succession, and inheritance follow character and world-persistence rules; NPC death and organizational leadership succession remain with their respective domains.

The player should not be required to manage kingdoms, entire armies, or exhaustive survival routines to experience the world.

## Shared infrastructure

These capabilities support multiple domains. They are not automatically independent simulation domains.

### Information and knowledge

Information handling is shared infrastructure used by all relevant domains. It is mostly invisible infrastructure, not a dedicated player-facing detective or rumor-management system.

Its basic principles are:

1. **No omniscience:** Entities act on information they have acquired, not automatically on every authoritative fact in the world.
2. **Acquisition:** Information may be learned through observation, participation, conversations, reports, records, and other plausible means.
3. **Explicit transmission:** Information spreads through actual communication or established channels; it is not automatically copied to every nearby entity or organization.
4. **Imperfect knowledge:** Information can be incomplete or incorrect, and entities may act on what they believe.
5. **Organizational retention:** Information communicated into organizational channels or preserved in records can outlive the members who acquired it. Information held only in an individual's memory may be lost when that individual is gone.

The effects should emerge through NPC behavior, dialogue, decisions, organizational responses, and world developments. Detailed reliability models, communication protocols, memory decay, and investigation procedures remain intentionally unresolved until a concrete gameplay need justifies them.

### Simulation coordination and common services

- **World time:** One authoritative world clock shared by all domains and regions.
- **Scheduling:** Shared coordination for future tasks and deadlines, distinct from records of events that have already happened.
- **Event coordination:** Domain-owned event queues, shared cross-domain event contracts, and deterministic delivery and ordering.
- **Persistence:** Domain-owned persistent state and schemas coordinated by the world-level save lifecycle.
- **Simulation budgets and fidelity:** Global resource and relevance policy coordinated by Enpicieus, with each domain defining its own supported fidelity levels and guarantees.
- **Execution coordination:** Enpicieus coordinates phases, dependencies, parallel work, catch-up, and relevance. It does not own domain mechanics or maintain duplicate authoritative copies of domain state.

World Events' records of significant ongoing developments are not the same thing as immutable event messages or domain-owned event queues. A domain event may report that something happened; World Events may maintain a longer-lived record of a major development when appropriate.

## Cross-domain examples

### An organization sends an expedition

An organization initiates and prepares an expedition; NPCs and creatures participate; geography and environment determine conditions; relevant economic systems account for supplies and costs. If the player joins, it becomes a player-facing activity or quest. If not, the expedition may be resolved abstractly without rendering every step. Significant discoveries or consequences become world facts and may be recorded as world events.

### A war affects a settlement

Organizations conduct the conflict and military operations. NPCs and populations fight, flee, or experience its consequences. Geography affects movement and conditions. Settlements and economies handle disruption, shortages, trade changes, and recovery. World Events may track the war's identity, status, timeline, and major milestones without owning the campaigns or battles.

### A creature threatens a trade route

Geography and environment represent its habitat and ecological context. NPC and population systems govern its behavior. Organizations and merchants respond according to their goals and acquired information. Settlements and economies represent resulting trade and service disruptions. A significant incident may become part of a larger world event.

## Scope and implementation boundaries

- This domain map describes intended gameplay responsibilities, not a commitment to simulate every listed process continuously.
- Detailed, event-driven, aggregate, generated, and deferred approaches may coexist according to gameplay relevance and each domain's guarantees.
- A mechanic should be implemented at the fidelity that creates meaningful gameplay. Avoid unnecessary simulation chores, exhaustive background detail, or player-facing management burdens that do not improve the experience.
- Domain APIs, internal processors, data structures, and implementation order remain open for dedicated technical design.
- A domain may be split later when a concrete responsibility and implementation need justify it.
- Mod extensibility should use explicit supported interfaces; this document does not promise arbitrary third-party domains or unrestricted authoritative entity types.
- Implementation should be guided by concrete gameplay scenarios, clear state ownership, deterministic outcomes, persistence needs, and measured performance on supported platforms.

This specification defines the current gameplay scope and responsibility map. It does not prescribe data structures, APIs, algorithms, or a complete implementation plan.
