# Integrated Livelihoods, Preparation, and Tactical Command

> Design record migrated from [GitHub issue #370](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/370). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

The Ever-Unfolding Saga should support different ways of living in and influencing its persistent world without becoming a collection of separate simulation games.

Farming, gathering, crafting, alchemy, trade, expedition preparation, and commanding groups in battle should build upon shared systems for resources, inventory, world time, NPCs, assignments, organizations, combat, and persistence.

These are long-term design directions, not claims that all the described systems currently exist or commitments to implement them all at full depth. The goal is to establish a coherent foundation that can be expanded incrementally while keeping exploration, individual characters, and lasting consequences at the center of the game.

## Design principles

- **One world, many possible lives.** Players should be able to pursue different activities within the same world rather than entering separate gameplay modes that barely interact.
- **Shared systems before specialized systems.** Activities should reuse common representations of resources, time, NPC capabilities, work assignments, groups, and persistent world state wherever practical.
- **Meaningful depth without unnecessary simulation.** An activity should present useful decisions and produce consequential outcomes without requiring every underlying action to be simulated individually.
- **Preparation rather than continuous needs management.** Supplies, equipment, medicines, information, and allies should matter when undertaking expeditions or entering dangerous situations. Constantly maintaining hunger, thirst, and similar meters should not become a mandatory gameplay loop.
- **Consequences persist.** Production, trade, battles, relationships, losses, and other significant outcomes should be compatible with world simulation, saving, character death, and inheritance.
- **Incremental implementation.** Establish small, complete examples before expanding the number of resources, recipes, activities, orders, or unit types.

## 1. Integrated livelihoods and economic activities

The world should support a connected resource loop:

1. **Acquisition:** gather materials, hunt, harvest crops, and discover resources through exploration.
2. **Transformation:** turn suitable resources into useful goods through crafting and alchemy, including potions and other preparations.
3. **Exchange:** sell surplus goods, purchase supplies, and trade resources between locations.
4. **Application:** use the resulting goods, wealth, and capabilities for exploration, combat, expeditions, and other ambitions.

The same resource should retain its identity and usefulness across these activities. For example, gathered herbs could be sold directly, used to produce potions, or carried on an expedition. Crafted goods could serve as equipment, supplies, or merchandise.

### Infrastructure construction and upkeep

Roads, trails, bridges, docks, ferries, and other route infrastructure are consumers of this shared resource and work model, not a separate economy. The infrastructure records in the world-topology system already represent route association, construction state, condition, access, and optional ownership, but they do not yet provide player-facing construction, repair, or transfer workflows.

When infrastructure lifecycle gameplay is implemented in v0.55.x:

- Construction and repair costs must draw from the same stackable resources and inventories/storage used by gathering, crafting, and trade.
- Costs must be validated and deducted atomically; failed or interrupted work must not silently consume materials or duplicate completed work.
- Work should account for time and plausible worker capability, tools, and assignments when those systems support them.
- Infrastructure may be funded or operated by a player, settlement, faction, organization, or other supported actor, but construction and operation must not imply ownership.
- Maintenance should respond to meaningful condition changes and consequences rather than impose arbitrary recurring chores.

Do not implement a parallel infrastructure stockpile, resource catalogue, crafting recipe system, or worker simulator solely to make route construction playable. The shared resource/inventory and work foundations are prerequisites.

### Farming and productive work

Farming should be a supported livelihood rather than a separate farming simulator.

- The player may participate in productive work directly.
- Work may be delegated to NPCs through the shared worker or assignment system.
- Production should depend on appropriate conditions and elapsed world time.
- Harvested resources should enter the same inventory, storage, crafting, and trading systems used elsewhere.
- Farming should not require micromanaging individual plants or maintaining a continuous daily checklist.

### Trade, crafting, and alchemy

Trading should offer meaningful choices based on location, availability, and relevant circumstances without immediately requiring a fully simulated market economy.

Crafting and alchemy should produce useful, differentiated results that interact with exploration, preparation, trade, and other systems. Avoid unnecessary resource and recipe proliferation that adds complexity without meaningful decisions.

The economic systems should be extensible, but their initial scope should remain limited to a small set of useful, interconnected activities.

## 2. Preparation for dangerous activities

Preparation should connect a character's livelihood and accumulated resources to the challenges they undertake.

Relevant factors may include:

- Equipment, tools, and consumables.
- Medicines, potions, and other crafted preparations.
- Food and supplies carried for an expedition, without mandatory continuous hunger simulation.
- Hired help, companions, and organized groups.
- Information, scouting, and knowledge of the destination.
- Appropriate logistics and arrangements for larger operations.

Preparation should improve a character's prospects without guaranteeing success. Its value should emerge through the existing exploration, combat, NPC, and world-simulation systems rather than through a separate preparation simulator.

## 3. Tactical command and larger-scale warfare

Larger battles should extend the spatial exploration and combat experience instead of introducing an entirely separate grand-strategy interface.

### Tactical command

A command control (potentially a button or equivalent interface) could pause the local simulation and allow the player to issue orders to groups they are authorized to command.

Initial orders could include:

- Move to a position or follow a route.
- Defend an area or hold position.
- Attack a target or advance toward an objective.
- Flank using a player-defined route or direction.
- Retreat, regroup, or otherwise reposition.

After issuing orders, the player resumes direct control of their character while allied units attempt to carry them out.

Orders express intentions rather than guarantee outcomes. Execution should depend on relevant factors such as unit capabilities, leadership, morale, equipment, terrain, communication, opposition, and available resources.

Command authority should be grounded in the existing or planned social and organizational systems. Proximity or friendliness alone should not automatically grant control over every allied unit.

The paused command interface is intended to support deliberate tactical decisions in a single-player game. Its effect on battle difficulty and micromanagement should be evaluated during implementation.

### Simulation at different scales

Not every combatant needs to be represented and managed at the same level of detail.

- **Player character:** directly controlled using the spatial combat system.
- **Nearby companions and fighters:** individually represented where practical, with their behavior affected by current orders and circumstances.
- **Larger groups:** commanded as squads or formations, with individual members acting under group orders where appropriate.
- **Battles outside the player's immediate presence:** resolved through world simulation at an appropriate level of abstraction.

Group-level simulation should simplify control and computation without discarding persistent consequences such as casualties, injuries, morale changes, and equipment losses.

### Warfare away from the player

The world should continue to experience conflicts when the player is elsewhere.

Off-screen battle outcomes should be determined by relevant known factors, potentially including force composition, equipment, leadership, morale, preparation, supplies, terrain, and objectives. Randomness may represent uncertainty and unforeseen events, but outcomes should not be arbitrary.

The player should not receive omniscient tactical access to battles they cannot observe. Information should depend on their presence, available intelligence, and reports or other communication methods.

Absence should limit direct intervention, not necessarily all strategic influence. Standing orders, delegated authority, and subordinate judgment may allow a force to act according to previously established objectives.

## 4. Scope boundaries

These systems should remain extensions of The Ever-Unfolding Saga's shared world and character experience, not obligations to reproduce entire genres.

**Not currently intended as separate full simulators:**

- A detailed farming-management game.
- A comprehensive business or market-management game.
- A standalone grand-strategy or kingdom-management game.
- A survival game centered on continuously managing hunger, thirst, and similar needs.
- A full dynasty simulator.

Inheritance remains an important part of the game's identity, but supporting descendants, succession, property, relationships, and lasting legacies does not require simulating every aspect of a dynasty.

Likewise, the world may contain farms, businesses, armies, and families without requiring the player to manage every detail of each institution.

A feature should justify its complexity by reinforcing the central experience, interacting meaningfully with existing systems, or producing distinctive consequences. The mere possibility of adding more depth is not sufficient justification for doing so.

## 5. Relationship to existing systems and implementation

These directions should reuse and remain consistent with the existing or planned systems for:

- World time, events, and variable simulation fidelity.
- Items, inventory, resources, and character capabilities.
- NPC behavior, recruitment, agreements, assignments, and organizations.
- Spatial exploration, combat, and local-world state.
- Persistence, death, retirement, and inheritance.

Relevant design issues include:

- #364 — Long-term game vision: exploration, self-directed lives, and inherited history.
- #365 — World topology, local exploration, and hybrid spatial travel.
- #366 — Procedural content generation: distinctive regions, locations, and opportunities.
- #367 — World simulation and evolution: time, events, and persistent consequences.
- #368 — Character lives, progression, retirement, death, and inheritance.
- #369 — Social influence, recruitment, assignments, and organizations.

Implementation should proceed incrementally. A sensible progression is to establish shared resource and work systems, demonstrate a small connected economic loop, make preparation useful in existing gameplay, and then extend spatial combat to small-group command before pursuing larger-scale battles.

This direction must not be interpreted as a reason to delay or displace the current spatial exploration prototype and its immediate milestones.

## Design validation criteria

The direction should be considered coherent when:

- The same resources and inventory representations are useful across gathering, farming, crafting, alchemy, and trade.
- Productive NPC work can use shared time and assignment mechanisms rather than requiring a separate worker simulation for every activity.
- Preparation has concrete effects on expeditions and combat without creating a mandatory continuous-needs loop.
- Tactical command builds upon the spatial combat experience and existing NPC and organizational logic.
- Off-screen conflicts produce understandable outcomes and persistent consequences without requiring full tactical simulation everywhere.
- These activities remain optional ways of engaging with the world rather than requirements imposed on every player.
- Save/load, death, and inheritance preserve relevant outcomes without duplicating actors, resources, assignments, or combat results.

## Non-goal

This issue documents a long-term design direction and its boundaries. It does not require immediate implementation of every activity described here, nor does it prescribe a complete economic model, a definitive combat-command interface, or maximum simulation detail for every system.
