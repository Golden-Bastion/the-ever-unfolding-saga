# The Ever-Unfolding Saga — Design Principles

This document is the canonical record of the current product direction. It supersedes earlier design assumptions wherever they conflict, especially older descriptions of the game as fundamentally text-first. This includes conflicting product-identity wording in design issues #228 and #229; their scoped technical details remain useful where consistent with this document. These principles describe intended design, not a claim that every feature is already implemented. Existing behavior and prototype status remain documented in the roadmap and system references.

## I. Core identity and player agency

1. **Character-focused RPG.** The character’s life, exploration, relationships, and adventures are the primary experience. Other systems enrich that experience rather than competing with it.
2. **Character-centred agency.** Players directly control plausible character actions while NPCs and wider systems retain autonomy.
3. **Integrated broader systems.** Broader systems connect with the main gameplay experience rather than feeling wholly disconnected.
4. **Contextual presentation.** Systems appear when relevant; not every subsystem needs permanent screen space.
5. **Plausible influence.** Orders and influence require plausible communication, proximity, visibility, authority, and circumstances. The character cannot arbitrarily control remote events.
6. **Player-perspective simulation.** Off-screen events should not become an omniscient second game. Players generally receive broad outcomes and reports rather than detailed control over remote activity.
7. **Long-term legacy.** Retirement, death, succession, inheritance, and the former character’s continued existence matter when relevant without remaining constantly visible.

## II. Exploration, gameplay, and interface

8. **Exploration is central.** Exploration is a primary way to encounter places, characters, stories, dangers, and opportunities.
9. **Supporting activities serve the main experience.** Their depth depends on how much they enrich the character’s life and gameplay.
10. **Meaningful-experience test.** Evaluate features by the meaningful experiences or consequences they create for the character.
11. **Clear interface responsibilities.** Spatial gameplay and menus/records have distinct responsibilities. Clarify and harmonize the existing interface; do not assume a wholesale redesign is required.
12. **Graphics-first spatial gameplay.** Visual, spatial play takes precedence during movement and action. Text remains important for dialogue, descriptions, logs, and records, but does not substitute for spatial gameplay. Older “text-first RPG” descriptions are no longer authoritative.
13. **Consistent spatial presentation.** Exploration, atmosphere, and combat should share a coherent spatial view.
14. **Real-time action combat direction.** The intended combat direction combines real-time action with preparation, abilities, enemy behaviour, positioning, timing, and tactical decisions. The current turn-based implementation is a transitional compatibility baseline until replacement is deliberately implemented.
15. **Character-centred large battles.** The player participates directly and influences nearby allies or subordinates rather than commanding a remote strategy map.
16. **Autonomous allies.** Allies act competently and contextually. Orders, relationships, and morale matter; constant micromanagement is not required.
17. **Knowledge-based information.** Information reflects character knowledge. Unknown motives and events remain unknown; known indicators and uncertainty may be communicated.
18. **Character screen as a broad hub.** Personal development, relationships, followers, work, reputation, and legacy can live in distinct tabs or sections. Inventory, Quests, and Journal belong under Character rather than as separate pause-menu destinations.
19. **Hybrid navigation.** Keep spatial gameplay clean and immersive, make frequent or contextual actions accessible, and put broader records and decisions in menus.
20. **Hierarchical information density.** Keep spatial gameplay relatively uncluttered and organise detailed information in record screens.
21. **Adaptive HUD.** Keep essential indicators visible and surface extra context during combat, interactions, and other situations that need it.
22. **Spatial interaction with contextual actions.** Movement happens in the world, with appropriate actions near people, objects, exits, and opportunities. Complex decisions may use menus.
23. **Location-based travel.** Use distinct local spatial areas connected by travel choices and transitions, not a required seamless global terrain map.
24. **Procedural foundations with authored meaning.** Procedural generation provides variety; deliberate placement and authored content make locations distinctive and meaningful.
25. **Authored and emergent stories.** Handcrafted stories create memorable experiences; simulation and player actions produce consequences and unexpected developments.
26. **Distinct, autonomous NPCs.** Important NPCs receive greater individuality and depth; ordinary NPCs still have enough identity and autonomy to create a living world. Behaviour emerges from separate personality, capability, motivation, relationships, knowledge, and circumstances. NPCs act only on information they could plausibly acquire, react with believable limits and delays, and retain consequential identity and commitments across simulation-fidelity changes.
27. **Hybrid relationship systems.** Underlying values and traits drive behaviour, while dialogue, events, actions, and relationship stages communicate outcomes.
28. **Hybrid progression.** Combine attributes, skills, equipment, milestones, practice, and significant life experiences.
29. **Consequential failure and death.** Failures can have lasting effects. Death may end the current character’s story while the wider world and potentially the legacy continue.
30. **Open-ended life and story.** Authored stories are opportunities, not mandatory routes. Players may ignore them, pursue ambitions, or follow emergent developments.
31. **Personal ambitions and opportunities.** Goals reflect traits, circumstances, and experience. Players can pursue, abandon, or replace them as the world presents possibilities.

## III. Organisations and world simulation

32. **Character-centred organisations.** Players may build or join complex organisations, but the experience focuses on the character’s role, relationships, actions, and influence.
33. **Simulation-driven economy.** Supply, demand, production, regional resources, and organisational finances follow a coherent simulation. Complexity does not require players to manage every detail.
34. **Meaningful economic consequences.** Player and NPC actions affect prices, resources, production, and regional prosperity within plausible limits.
35. **Hybrid time model.** Spatial gameplay runs in real time; deliberate activities advance time according to duration; menus and records generally pause time or avoid advancing it.
36. **Slow-motion tactical orders.** Tactical slow motion lets players issue orders while the world continues moving slowly. This supports tactical choices in large battles while retaining real-time action and autonomous allies. Orders remain constrained by plausible communication and authority.
37. **World-driven challenge.** Dangers follow world logic rather than automatically scaling to the player or adapting to player performance. Preparation, equipment, allies, knowledge, tactics, and activity shape risk; players may encounter threats beyond their current capabilities and may avoid, retreat from, or return to them later.
38. **Context-sensitive survival needs.** Needs matter during expeditions and harsh conditions, injuries, or shortages without turning ordinary play into a maintenance loop.
39. **Exploration-linked gathering and crafting.** Gathering rewards exploration; alchemy and selective crafting support preparation and experimentation rather than becoming a full crafting simulator.
40. **Flexible character identity.** Players customise appearance and starting traits/background; identity continues evolving through choices, experiences, relationships, and progression.
41. **Extensive appearance and equipment customisation.** Aim for detailed appearance editing, layered clothing and armour, dyes, cosmetics, and varied combinations. Delivery may be staged over development.
42. **Layered lore discovery.** Environments establish atmosphere and clues; conversations, exploration, documents, and events reveal deeper context.
43. **Integrated cultural identity.** Visual design, history, customs, beliefs, laws, economics, and relationships reinforce cultural identity.
44. **Consequences at multiple scales.** Actions may affect individuals, communities, factions, and regions according to their significance and circumstances.
45. **Layered, knowledge-dependent reputation.** Reputation varies by individual, faction, settlement, and region. Information travels through plausible channels; NPCs respond to what they witness, know, or believe.
46. **Contextual justice.** Laws, enforcement, evidence, witnesses, social standing, and local power shape plausible consequences. Justice emerges from what authorities can know and do.
47. **Character-centred warfare.** War can reshape regions, but the player experiences it through travel, combat, allies, personal obligations, and local consequences—not a separate grand-strategy game.
48. **Simulation-driven settlements.** Population, prosperity, resources, leadership, safety, and local events change settlements over time. Players encounter these changes directly without necessarily managing settlements.
49. **Persistent world simulation.** NPCs, settlements, factions, and local conditions evolve away from the player, with simulation detail and fidelity scaled to relevance.
50. **Knowledge-based reporting.** News arrives through plausible channels such as witnesses, travellers, allies, letters, and faction contacts. Reports may be delayed, incomplete, biased, or wrong.
51. **World-driven timing.** Opportunities develop, change, or disappear according to events, travel time, NPC decisions, and circumstances. Convey urgency only when the character could reasonably know.
52. **Emergent quest consequences.** Objectives can become impossible, NPCs can act without the player, and failure or abandonment can change relationships and circumstances. Alternative outcomes remain possible where plausible.

## IV. Social systems and the character’s life

53. **Systemic conversations.** Authored dialogue combines with contextual social simulation. Conversations depend on the character’s knowledge, motives, feelings, and willingness to disclose information.
54. **Contextual social interaction.** Persuasion, deception, intimidation, negotiation, and similar actions depend on ability, motives, trust, loyalty, leverage, evidence, culture, risks, and knowledge. Outcomes may be partial or temporary.
55. **Living companion relationships.** Companions combine authored personal arcs with evolving trust, loyalty, shared experiences, conflicts, ambitions, and autonomy. They can change or leave.
56. **Systemic romance.** Romance and intimacy emerge from mutual attraction, compatibility, trust, consent, circumstances, and choices, with authored arcs where appropriate.
57. **Optional family and legacy.** Family, children, and inheritance matter when pursued, especially around retirement or death. A dynasty is not mandatory.
58. **Meaningful life-cycle transitions.** Retirement and death can be important world events. Former characters may remain NPCs where plausible; a successor may continue the story when available, but succession is not guaranteed.
59. **Consequential lifespan and aging.** Aging interacts with health, injuries, experience, ambitions, relationships, and events. Changes vary by character and circumstance rather than following a rigid countdown.

## V. Complexity, fairness, and player understanding

60. **Deep simulation, accessible presentation.** Systems may be intricate underneath while the interface surfaces relevant decisions, understandable feedback, and convenient actions without exposing every calculation.
61. **Fixed challenge and accessibility.** The game offers no selectable gameplay difficulty presets or adaptive difficulty. Challenge follows coherent world and system rules; accessibility options remove unnecessary interaction and presentation barriers without becoming hidden difficulty controls. Preparation, knowledge, tactics, and player decisions provide ways to meet demanding situations.
62. **Contextual learning.** Introduce mechanics when the character encounters relevant situations, with optional explanations and reference material available later.
63. **Explainable simulation.** Communicate important causes, effects, contributing factors, and uncertainty consistent with character knowledge, without exposing every calculation or granting omniscience.
64. **Contextual choices.** Present actions plausible for the character’s circumstances, abilities, knowledge, and relationships. Show known costs and risks; let uncertain consequences emerge naturally.
65. **Contextual feasibility.** Distinguish impossible, risky, difficult, improper, and unauthorised actions. Block true impossibilities but allow plausible attempts even when they may fail or carry consequences.
66. **Consistent simulation over forced balance.** Welcome coherent emergent strategies. Fix bugs, infinite loops, and nonsensical outcomes; balance should support the intended experience, not eliminate every advantage.

## VI. Modding, saves, and technical foundations

67. **Broad moddability.** Provide documented content formats and extension points for substantial new content and selected gameplay systems. Mods should be manageable independently while the base game remains coherent.
68. **Managed mod compatibility.** Mods declare versions/dependencies, validate content, report conflicts, support load ordering where relevant, and provide practical recovery guidance when mods or updates threaten saves.
69. **Protect persistent stories.** Use reliable saves, recoverable backups, schema migrations, compatibility warnings, validation, and safeguards for long-running worlds, character transitions, and modded campaigns.
70. **Scalable simulation and presentation.** Preserve meaningful world behaviour as complexity grows while allocating computation according to what can affect or be perceived by the player. Scale simulation fidelity and update frequency by relevance; scale rendering detail by distance, screen-space importance, visibility, and hardware capability. Use simpler models and materials, impostors, culling, reduced animation work, and abstract off-screen simulation where appropriate. Increase fidelity as entities become relevant, preserving their identity and state across levels of detail. Profile and validate these transitions: simplification must not create conspicuous visual inconsistencies, break causal world behaviour, or discard meaningful state.
71. **Pixel-art visual direction.** The target presentation is distinctly 2D and pixel-art-led, with detailed sprite-based characters, tile-based environments, and a rich, atmospheric world. Retain the established angled/three-quarter spatial camera; use layered composition, depth ordering, silhouette, lighting, colour, and effects to create a sense of depth. A 2.5D impression is welcome without making full 3D character or environment models a prerequisite. Prioritise techniques that suit an indie development budget and supported hardware. Keep visual identity and readability strong. Concept references guide the overall feel and information hierarchy; their exact art, labels, values, and layouts are illustrative rather than literal implementation requirements.
72. **Integrated audiovisual design through Verdandi.** Music, ambience, sound effects, and gameplay feedback reinforce atmosphere, danger, action, and character experiences. Audio development belongs in the separate Rust crate/library named Verdandi, avoiding tight coupling with the main game code.
73. **Hybrid asset pipeline.** Combine handcrafted signature assets with procedural variation, reusable components, and automated validation.
74. **Hybrid animation system.** Combine authored signature animations with reusable locomotion, blending, procedural adjustments, and context-sensitive reactions.
75. **Purposeful modularity.** Separate modules/crates when independence, reuse, testing, or ownership clarity justify it. Avoid unnecessary abstractions and crate boundaries.
76. **Context-sensitive resilience.** Unexpected failures should be clear during development. Validate important state, recover safely from expected errors, preserve diagnostics, and never conceal serious defects or silently corrupt data.
77. **Layered testing.** Use unit tests, integration tests, simulation invariants, reproducible scenarios, and manual playtesting.
78. **In-game development tools.** Provide tools such as interactive inspectors, debug overlays, time controls, entity inspection, and ways to examine live simulation behaviour.
79. **Shared but concealed developer console.** Retain the existing console, accessible through the tilde key or a long press of the pause button. It serves developers, players, and modders while remaining out of ordinary gameplay by default.
80. **Layered command system.** Provide stable core commands and allow systems/mods to extend them. Include discoverability, argument validation, appropriate permissions, and useful error messages.
81. **Managed save migrations.** Version save schemas, use explicit migration paths, validate migrated data, preserve backups, and warn when compatibility cannot be guaranteed.
82. **Strict campaign mod lock-in.** Record each campaign’s exact mod configuration and prevent changes to that configuration during the campaign. Changing the setup requires a new campaign to protect consistency and save integrity.
83. **Explicit mod capability boundaries.** Define permissions by mod type, validate content, warn about powerful capabilities, and isolate risky extensions where practical.
84. **Open, file-based mod distribution.** Support local files and external repositories with documented formats, clear installation expectations, and basic validation. An official marketplace is not required.
85. **Progressive modding tools.** Start with documented formats, examples, templates, and validation. Add dedicated editors when actual workflows justify them.

## VII. Future compatibility, localisation, and accessibility

86. **Single-player first; multiplayer later.** Prioritise single-player. Avoid unnecessary architectural decisions that would make future cooperative multiplayer impossible, but do not treat multiplayer as a current design constraint.
87. **Low-cost multiplayer compatibility.** Maintain sound state ownership and clear system boundaries, using determinism where naturally useful, without premature networking abstractions.
88. **Community-driven localisation.** Prepare localisation-ready foundations and documented translation files so contributors can add languages without changing game logic.
89. **Layered accessibility.** Provide accessible defaults and essential options early, then expand configurable controls, visual/audio alternatives, interface scaling, and adjustable gameplay assistance.

## VIII. Platform and release policy

90. **Scalable hardware tiers.** Define realistic minimum and recommended hardware targets. Adapt rendering, effects, simulation frequency, and optional features to hardware capabilities while preserving the core experience across supported systems.
91. **PC and Android support.** Target desktop (Windows, Linux, and macOS where officially supported) and Android. Adapt controls, layouts, rendering, and presentation to each instead of simply shrinking the desktop interface. iOS is not a project target. Official build targets and validation status remain tracked separately in the roadmap.
92. **Hybrid input support.** Support touch, mouse/keyboard, and controllers where practical, with platform-appropriate layouts and consistent underlying gameplay actions.
93. **Fully offline-capable single-player.** The complete single-player experience—including core gameplay, simulation, and local saves—works without internet access. Connectivity may enable optional conveniences but must not be required for play.
94. **Player-controlled updates.** Players decide when to download and install updates. Update delivery should respect that choice, avoid unexpectedly interrupting play, and work alongside save-compatibility safeguards.

## IX. Product direction and prototype evolution

95. **Vision over prototype preservation.** The intended game and player experience—not the current implementation—is the source of truth. Prototype-era code, data models, generated worlds and maps, placeholder assets, and technical decisions are provisional. Redesign, replace, regenerate, or discard them when they constrain the intended experience. Do not perpetuate a limitation merely because it already exists or because work has been invested in it. Preserve meaningful player progress when practical, but do not confuse that with preserving every prototype artifact. During pre-release development, resetting or regenerating project-owned prototype data is acceptable when it is the better path to the game we intend to build. Backward compatibility is a means to protect players, not a goal that outranks the vision; make consequential breaking changes deliberate and document their effects.

## How to apply these principles

- Treat this document as the current product-direction authority when older prose conflicts with it.
- Distinguish intended direction from implemented status. Do not describe planned real-time spatial combat, strict campaign mod lock-in, or other future goals as already delivered.
- Keep bounded implementation milestones small and testable. A milestone may intentionally defer a long-term principle without contradicting it.
- Preserve useful existing systems when they fit the direction; change or replace them deliberately when they conflict. Avoid maintaining parallel implementations solely to preserve an outdated product description.
- Use the roadmap for current state and delivery order, the development plan for implementation priorities, the architecture document for technical boundaries, and system documents for implementation detail.


The detailed system designs that apply these principles are indexed in the [design specification library](design/README.md). The authoritative list and sequencing of unfinished work remain in the root [roadmap](../ROADMAP.md).
