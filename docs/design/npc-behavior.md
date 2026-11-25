# NPC Behaviour, Perception, and Navigation

This specification defines the shared behavioural foundations for NPCs and other autonomous actors. It complements the world simulation, procedural content, social systems, spatial exploration, and combat specifications; it does not replace their domain-specific rules.

## Goal

Create inhabitants who behave as distinct, understandable individuals within a persistent world. NPCs should act from their own capabilities, needs, goals, relationships, knowledge, and circumstances rather than merely waiting for player interaction or following scripts.

Believability does not require perfect intelligence or a full continuous simulation for every entity. It requires coherent decisions, limited knowledge, consistent tendencies, appropriate mistakes, and continuity when the player leaves.

## 1. Separate behavioural dimensions

An NPC's behaviour should be shaped by distinct dimensions rather than one generic intelligence or personality score:

- **Personality:** relatively stable tendencies such as courage, aggression, caution, patience, curiosity, sociability, greed, ambition, discipline, and cooperativeness.
- **Capabilities:** skills, knowledge, physical attributes, equipment, health, and other factors determining what the NPC can actually do.
- **Motivation and goals:** current needs, ambitions, priorities, fears, plans, and intended outcomes.
- **Relationships and commitments:** target-specific loyalty and trust, affiliations, agreements, obligations, friendships, rivalries, and authority.
- **Knowledge and beliefs:** what the NPC has perceived, learned, been told, remembers, or inferred, including uncertainty and possible error.
- **Circumstances:** location, time, environmental conditions, immediate threats, available resources, injuries, orders, and competing demands.

These dimensions must remain distinguishable. A skilled character need not be courageous; an aggressive character need not be competent; a loyal subordinate need not agree with every order. Personality influences decisions but does not dictate them. Circumstances and stronger commitments can override a usual tendency.

## 2. Data-driven behavioural profiles

Use reusable, data-driven profiles and composable traits rather than a separate AI implementation for every personality.

A profile may provide defaults and inherit from a broader archetype, while an individual can override selected values or add authored characteristics. Procedurally generated profiles should use deterministic inputs and contextual constraints so that individuals remain stable across regeneration, save/load, and inheritance.

Traits should influence concrete decision weights, thresholds, preferred approaches, risk tolerance, and reactions. They must not become decorative labels that only affect dialogue. Conversely, do not expose every internal parameter as a player-facing statistic.

Avoid reducing personality to a handful of rigid classes. Two guards can share a profession and training while differing in caution, initiative, discipline, or willingness to protect others.

Personality should be stable enough to establish recognizable tendencies, but may evolve through consequential experiences where the relevant system supports such change. Temporary emotional states, injuries, stress, fear, and current motivation must not be confused with permanent personality.

## 3. Perception, knowledge, and imperfect information

NPCs do not possess omniscient access to authoritative world state. Each actor acts on information available to them through plausible channels:

- Direct perception, including sight, hearing, and other supported senses.
- Witnessing an event or examining its physical evidence.
- Conversations, reports, orders, rumours, and other information channels.
- Previous experiences and stored memories.
- Reasonable inferences from known facts and circumstances.

Information may be incomplete, delayed, mistaken, biased, or outdated. A report should retain enough provenance and confidence to distinguish direct observation from hearsay or inference where that distinction affects decisions.

Separate authoritative world facts from each NPC's knowledge of those facts. The world may know that a route is blocked while a distant traveller still believes it is open. NPCs should not react to hidden player actions or remote events unless information reaches them through a plausible mechanism.

Perception must be bounded by the environment and the actor's actual capabilities. Occlusion, distance, noise, attention, weather, lighting, and other relevant factors may affect detection as their supporting systems become available. Do not require a physically exhaustive sensory simulation before the basic knowledge boundary is useful.

## 4. Memory and continuity

Memory should preserve experiences that can meaningfully influence future behaviour: witnessed events, important conversations, promises, betrayals, injuries, threats, discoveries, assignments, and consequential outcomes.

Not every observation requires permanent storage. Use relevance, importance, recency, and domain-specific retention rules to keep memory bounded. Summaries or aggregate records may replace low-value detail when they preserve the consequences the game promises.

Memory must have a clear owner and persistence scope. Personal knowledge and relationships may follow character-specific inheritance rules; world history, public events, and lasting physical consequences remain world-persistent. A successor must not automatically inherit another character's private knowledge merely because they inherit the same world.

NPCs should not forget consequential commitments or identity when moving between detailed, aggregate, and deferred simulation. If an implementation cannot preserve a detail at every fidelity level, it must preserve the outcome and any information needed for later decisions.

## 5. Decision-making and action selection

NPC decision-making should select plausible actions from current goals, available information, capabilities, commitments, and circumstances.

A decision process should, as appropriate:

1. Establish the actor's current needs, goals, obligations, and active commitments.
2. Determine which facts the actor knows and how reliable or current they are.
3. Identify feasible actions using capabilities, resources, authority, and environmental constraints.
4. Evaluate likely outcomes, risks, rewards, urgency, and conflicts with other priorities.
5. Select an action consistent with the actor's tendencies and circumstances.
6. Execute, monitor, and revise the action when important conditions change.

The architecture may combine utility scoring, priorities, state machines, planning, authored rules, or other suitable methods. No single algorithm is mandated. The chosen method must remain inspectable, deterministic where required, testable, and appropriate to the scale of the decision.

NPCs should make understandable mistakes. Imperfect perception, limited attention, delayed reactions, poor judgement, conflicting goals, fear, and insufficient capability can produce failure without resorting to arbitrary randomness. Avoid perfect reaction, perfect aim, universal tactical knowledge, and instant adaptation unless a particular creature or supernatural ability justifies it.

An NPC's response should reflect reaction time and attention. A character already occupied, distracted, surprised, exhausted, or unaware should not respond as though they had perfect and immediate knowledge.

## 6. Navigation and tactical awareness

Navigation is more than selecting a shortest path. Actors should be able to use the environment in ways appropriate to their goals and capabilities.

The navigation foundation should support, as relevant:

- Reachability and valid movement through authoritative topology.
- Alternative routes, obstacles, doors, passages, and environmental hazards.
- Cover, concealment, visibility, and defensible positions.
- Approach distance, line of sight, escape routes, and safe retreat.
- Flanking, group spacing, chokepoints, and other tactical opportunities.
- Movement constraints arising from actor size, equipment, injury, or movement capability.

Do not require every actor to evaluate every tactical feature. Expose environmental affordances through reusable queries or annotations and let the decision system consider only those relevant to its role and current goal.

The navigation implementation must respect the authoritative spatial map and world-route topology. A tactical preference cannot authorize movement through blocked geometry, closed or locked doors, unavailable infrastructure, or an undiscovered route that the actor has no means to know.

A navigation mesh is one possible representation for larger spaces, not a required replacement for the current grid and room topology. Choose representations according to the environment and measured needs.

## 7. Group behaviour and communication

Groups should coordinate through plausible information, shared goals, roles, authority, and communication. Membership alone does not create perfect coordination or shared knowledge.

Depending on capability and circumstances, group members may follow orders, share observations, protect allies, maintain formation, pursue separate tasks, disagree, hesitate, or abandon the group. Leaders should delegate outcomes rather than require micromanagement of every movement and action.

Communication has constraints: distance, visibility, audibility, available channels, authority, trust, and time. A group member who cannot receive an order should not act as though they did. Shared knowledge should be an explicit result of communication or another defined mechanism.

Group coordination should reuse the social influence and organization model. Combat-specific tactics belong to the combat system, while this specification defines the general behavioural contract.

## 8. Simulation fidelity and off-screen behaviour

Apply the world simulation's relevance-based fidelity model to NPCs:

- **High detail:** nearby or consequential actors receive the perception, decisions, movement, and action resolution required by the active situation.
- **Reduced detail:** regional actors may update less frequently or use simplified movement and goal progress.
- **Aggregate or deferred:** distant routine activity may resolve through scheduled or aggregate outcomes when intermediate actions do not matter to the player-facing result.

These are not rigid distance bands. An off-screen leader making a consequential decision may need detailed resolution; a nearby background inhabitant may need only a lightweight state update.

Fidelity changes must preserve stable identity, goals, commitments, relationships, knowledge that matters, and consequential outcomes. Catch-up must be bounded, deterministic where required, and idempotent. It must not silently create impossible travel, duplicate rewards, erase obligations, or rewrite established history.

## 9. Integration boundaries

- **World simulation** owns authoritative time, persistence, scheduling, and the rules for detailed, aggregate, and deferred updates.
- **Enpicieus** coordinates eligible world activity and opportunities at world scale; it does not replace individual NPC decision-making or authoritatively dictate every actor's actions.
- **Social influence and organizations** own relationship semantics, agreements, authority, recruitment, assignments, and group membership.
- **Procedural content generation** creates contextually appropriate initial profiles, roles, relationships, and opportunities with stable identities.
- **Spatial exploration** owns authoritative local topology and movement constraints; navigation consumes those rules.
- **Combat** owns encounter-specific tactical decisions, attack timing, target selection, and combat resolution while reusing NPC personality, perception, capabilities, and morale where appropriate.

These boundaries are responsibilities, not a mandate to create separate Rust crates or introduce premature abstractions.

## 10. Validation requirements

Tests and reproducible scenarios should cover:

- Personality differences producing distinct but plausible decisions under identical circumstances.
- Competence, personality, motivation, and temporary state remaining independent.
- NPCs reacting only to information they could plausibly know.
- Delayed, unreliable, or contradictory reports producing understandable uncertainty.
- Reaction delays and attention constraints preventing perfect responses.
- Navigation respecting blocked geometry, doors, routes, and actor capabilities.
- Group communication and orders respecting reachability and authority.
- Save/load, re-entry, and fidelity transitions preserving identity and consequential state.
- Deterministic generation and decision outcomes where stable inputs are required.
- Bounded work for large populations and deferred catch-up.

Tests must validate behavioural invariants and meaningful outcomes, not merely assert that an AI action was produced. Do not weaken expected behaviour to make a failing scenario pass.

## Non-goals

- No requirement for every NPC to run a full planner or continuous simulation.
- No universal personality taxonomy or fixed list of traits.
- No omniscient NPC knowledge or perfect tactical responses.
- No commitment to a particular AI algorithm, navigation-mesh technology, or crate architecture.
- No runtime language model or external AI service requirement.
- No replacement of authored characters, quests, or encounter design with generic procedural behaviour.
