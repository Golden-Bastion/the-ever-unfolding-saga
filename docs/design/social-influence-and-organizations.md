# Social Influence, Recruitment, Assignments, and Organizations

> Design record migrated from [GitHub issue #369](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/369). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

Approved design direction for future implementation. The loyalty/trust model is the agreed foundation. This issue is a long-term system specification, not a commitment to implement the full system in one milestone.

The system should extend the existing NPC interaction, faction, quest, world-time, persistence, and simulation foundations. It connects especially to #367 and #368.

## Goal

Let the player turn social influence into practical action, and practical action into lasting institutions and power. The player should be able to secure cooperation from individuals and groups, delegate tasks, establish organizations, and build toward leadership of a company, cult, settlement, warband, or army.

Hiring, persuasion, coercion, and force are different ways of securing cooperation or imposing an outcome. They must not all produce the same generic obedience state.

The system should support a small-scale individual interaction first and scale to organizations and larger groups without requiring unrelated systems for every kind of group.

## 1. NPC agency and willingness

NPCs should be represented as actors with capabilities, motivations, relationships, commitments, and (as systems mature) plans. These characteristics affect what they can do, what they want to do, and how they respond to requests and orders.

Not every NPC needs a deep personality or continuous simulation immediately. Important companions, leaders, and other consequential NPCs can receive richer behavior; routine NPCs and distant groups may use simpler or event-driven models. The same underlying concepts should still apply.

### Loyalty and trust: two independent relationship axes

For a given NPC in relation to a specific person, leader, or group, track two separate values in the range -100 to +100:

- **Loyalty:** whether the NPC is personally committed to the target's interests, leadership, or cause. A strongly negative value represents opposition or rebelliousness; around zero represents little personal attachment; a strongly positive value represents devotion.
- **Trust:** the NPC's confidence or emotional stance toward the target. At the negative end, the NPC is terrified and expects danger, abuse, or betrayal. Around zero, the target is not well known and confidence is limited. At the positive end, the NPC has powerful faith or confidence, potentially reaching fanaticism.

Trust is intentionally broader than ordinary confidence: its negative end includes fear. The simulation must not mistake fear-induced compliance for genuine trust.

The two values are independent. A person may be:
- A **terrified rebel**: obeys because resistance feels too dangerous but shirks, escapes, or sabotages when an opportunity arises.
- A **reliable but uncommitted worker**: trusts a leader's competence but lacks personal allegiance; fulfills agreed work while respecting contractual limits.
- A **fearful loyalist**: cares about the cause or group but doubts or fears the current leader.
- A **fanatic**: deeply committed and convinced, willing to accept extreme or even suicidal risks for the cause.

These are behavioral examples, not rigid score bands or personality classes. Intermediate combinations should remain possible.

Relationships are target-specific. An NPC can be loyal to an organization but distrust its leader, personally devoted to the player but remain a member of another faction, or trust an ally without owing them loyalty.

### Do not reduce all willingness to these two values

Loyalty and trust are core relationship variables, but they are not the whole decision model. An NPC's response should also consider:

- The reason they are working with or obeying the requester.
- Their skills, equipment, health, available resources, and actual capability.
- Their current obligations, orders, contracts, and relationships.
- The proposed task's rewards, risk, moral or ideological fit, and perceived chance of success.
- The authority of the person issuing the request and the consequences of refusing.
- The NPC's personal limits and circumstances.

A fanatical follower may accept a suicide mission if they believe the sacrifice serves their cause, but fanaticism does not make an impossible task succeed or supply missing resources. Likewise, payment can motivate a mercenary without creating personal loyalty.

## 2. Ways to secure cooperation

The interaction model should support four broad approaches that can later be exposed through appropriate contextual actions.

### Hire

Obtain an agreement in exchange for payment or other compensation. Contracts should be able to define the work, compensation, expected duration, expenses, risk, and conditions for completion. A short-term contract does not automatically make someone a permanent follower or subordinate.

Employment may be affected by market conditions, the individual's needs, their skills, reputation, and the perceived risk.

### Persuade

Seek voluntary cooperation through shared interests, incentives, trust, relationships, evidence, promises, or ideological alignment. Successful persuasion should create an agreement, commitment, or change in relationship appropriate to what was discussed. A failed attempt should not always be a binary dead end: it may reveal objections or the conditions that could change the NPC's mind.

### Coerce

Seek compliance using threats, fear, debts, blackmail, leverage, or other pressure. Coercion can produce short-term obedience without increasing loyalty. It can damage trust, lead to low effort or sabotage, and increase the chance that the target escapes, defects, or retaliates when able.

### Force

Use superior force or an established position of power to impose an outcome, such as capturing an enemy, compelling surrender, taking control of a location, or forcing a defeated group into service. Forced compliance is not the same as voluntary membership; resistance, flight, grudges, and future rebellion remain possible.

These approaches should share a common model of NPC motivation, relationship changes, commitments, and outcomes rather than grow into four disconnected minigames.

## 3. Orders and assignments

The player should be able to specify a desired outcome and delegate it to an individual or a group. An assignment should have structured information appropriate to its scope, such as:

- What goal is being pursued and where.
- Who is responsible and who has authority to carry it out.
- Required tools, personnel, money, supplies, and other resources.
- Relevant deadlines or expected duration, when applicable.
- The assignment's risk, constraints, and acceptable use of force or resources.
- Current status, progress, impediments, and final outcome.

Candidate activities, added as their supporting systems exist, include guarding, escorting, scouting, gathering resources, working on a project, constructing a structure, trading, delivering messages, recruiting, spreading influence, fighting enemies, defending a location, or conducting an expedition.

Orders describe goals, not necessarily every individual action. A capable subordinate or group should have some ability to decide how to accomplish a task. This creates a foundation for delegation and command rather than requiring the player to micromanage each worker or soldier.

Issuing an assignment does not guarantee success. The result should depend on capability, equipment, preparation, knowledge, commitment, resources, opposition, and the task's difficulty. The system should report significant progress, setbacks, requests, failures, and consequences.

## 4. Organizations and group structure

Use one coherent organization model for different kinds of collective activity, instead of separate unrelated implementations for every possible group.

An organization may support, as applicable:
- Members and membership status.
- Roles, ranks, leaders, and delegated authority.
- Shared resources, finances, equipment, and obligations.
- Goals, rules, doctrine, or other defining purpose.
- Assignments and subordinate groups.
- Internal relationships, loyalty, trust, morale, disputes, and leadership challenges.
- External relationships with other organizations, factions, and communities.

The existing Faction model is currently much simpler and already tracks reputation and memory. Before implementing a separate Organization model, decide how organizations relate to existing factions so state and behavior are not duplicated. The design should be capable of treating existing factions as a supported kind or aspect of organized collective identity, while preserving faction-specific behavior where needed.

Examples of organization types include:
- **Company or enterprise:** trading, production, services, employees, contracts, and finances.
- **Cult or ideological movement:** doctrine, recruitment, shared belief, loyalty, influence, and possible internal schisms.
- **Mercenary company or warband:** combat capability, contracts, equipment, command, morale, and payment.
- **Army:** structured units, commanders, supply, strategic assignments, and control of territory or battle objectives.
- **Settlement or household:** residents, work, resources, leadership, and shared interests.

These should be different configurations and specializations of the shared model rather than entirely independent foundational systems. Their specific mechanics should only be promised once they are implemented.

Organizations should be able to outlive the active character. The player's death or retirement should not automatically dissolve a company, cult, or army. Leadership may change, members may pursue their own interests, and a successor may encounter a previous character's organization and its changed circumstances. This should integrate with #368's persistent life records and retirement design.

## 5. Group action and scale

The system should work at multiple scales.

- **Individual:** ask, hire, persuade, threaten, recruit, or assign a person to a task.
- **Small group:** assemble a crew, companion party, expedition, cult cell, work team, or warband with roles and a common assignment.
- **Organization:** coordinate multiple groups, delegate authority, allocate resources, and pursue broader goals.

The architecture must not require all group members to be individually simulated at full detail for every operation. Important nearby or consequential situations can receive detailed resolution. Routine or distant work can use scheduled or aggregate resolution when intermediate steps do not materially affect the outcome. This follows #367's principle of choosing simulation fidelity based on relevance and measured cost, not imposing a hard ceiling on future depth.

As combat develops, the same structure should support commanders and units, but a fully featured army simulation is not a prerequisite for the first recruitment and assignment prototype.

## 6. Consequences and counterplay

Power and organization should expand opportunities while introducing meaningful responsibilities and opposition.

Examples of possible consequences:
- Employees expect wages, tools, supplies, and agreed working conditions.
- Companions may leave if commitments are broken or goals diverge.
- A group can suffer low morale, poor leadership, desertion, or internal conflict.
- Coerced subordinates may comply superficially, sabotage tasks, escape, or retaliate.
- A cult or company may develop rivalries, competing leaders, or internal factions.
- An army may be constrained by food, equipment, command, logistics, morale, and supply lines.
- Successful organizations attract attention from competitors, authorities, potential allies, or enemies.

These consequences must follow from state and simulation rules, not arbitrary punishment. Different characters and group types can value different things.

Relationships should change through lived experience: fulfilled promises, fair payment, protection, shared success, and good leadership can strengthen trust or loyalty; betrayal, humiliation, preventable losses, broken agreements, and abuse can damage them. Fear can rise without increasing loyalty; trust can rise without creating allegiance.

## 7. Persistence, ownership, and compatibility

Agreements, relationships, assignments, organizations, leadership, resources, and consequential outcomes must be structured persistent state, not represented only by unstructured memory strings.

Save/load and character inheritance must preserve meaningful ongoing work and organization state. Personal relationships and membership should remain distinct from the world's memory of past deeds. When an NPC retires, dies, defects, or changes organizations, the system should update membership and assignments consistently rather than leave duplicate active actors.

World time should drive deadlines and scheduled progress. Every assignment must have clear handling for cancellation, failure, interruption, loss of its leader, unavailable resources, and time elapsed while its participants are off-screen.

New persisted fields should follow the project's versioned save compatibility practices. Transition logic must be idempotent enough that save/load or repeated events do not duplicate wages, items, members, or assignment outcomes.

## Suggested implementation sequence

1. Define structured NPC–target relationships and implement the two independent loyalty/trust values with clear semantics.
2. Add small contextual interactions for hiring/recruiting and voluntary requests, then coercion and forced compliance, with distinct consequences.
3. Establish a persistent agreement/commitment model so contracts, followers, obligations, and authority do not rely on labels or dialogue alone.
4. Add basic assignments for a small number of already-supported tasks, with time, resources, outcomes, and failure cases.
5. Define the relationship between Faction and Organization, then add a small group with membership, leadership, resources, and delegated assignments.
6. Expand to organization-specific behavior, subordinate command structures, and more detailed or large-scale simulation as the underlying activities become playable.
7. Validate persistence, legacy saves, changing relationships, coerced compliance, assignment failures, group transitions, and continuity across character death and retirement.

## Acceptance criteria

- Loyalty and trust are independent, target-specific relationship values with documented behavioral meanings.
- The same order can lead to different behavior based on relationship, capability, motive, obligations, and circumstances.
- Fear-induced compliance does not automatically imply loyalty or trust; voluntary commitment does not guarantee competence.
- Hiring, persuasion, coercion, and force create distinct forms of cooperation and consequences.
- Assignments have explicit responsible actors, requirements, status, and persistent outcomes.
- NPCs and groups cannot perform tasks for which they have no plausible capability or resources without a defined mechanic explaining the outcome.
- Groups can be assembled and led without duplicating the underlying membership, resource, or relationship state.
- Organizations can persist beyond a character's active life and can be encountered or influenced by a successor.
- Distant or routine work can be updated through suitable lower-fidelity simulation while consequential situations can receive richer detail.
- Save/load and repeated transitions do not duplicate members, property, payments, or completed assignment outcomes.
- Tests cover all four cooperation approaches, contrasting loyalty/trust combinations, task capability and failure, organizational membership changes, and persistence.
