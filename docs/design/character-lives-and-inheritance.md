# Character Lives, Progression, Retirement, Death, and Inheritance

> Design record migrated from [GitHub issue #368](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/368). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

This design extends the existing character progression, death, corpse recovery, world persistence, and character-inheritance systems. It should coordinate with #364 and #367, rather than introduce a separate campaign or simulation model.

## Goal

Make different character lives meaningful in the same persistent world. A player should be able to pursue different ambitions, develop capabilities through their chosen activities, and eventually lose or voluntarily relinquish control of a character without erasing that character's history or the world's consequences.

Death and retirement both end active control of a character, but they are not the same outcome:

- **Death** ends the character's life in the world and leaves a corpse and lasting record.
- **Retirement** ends the character's active player-controlled career while the character remains alive in the world as an NPC. A later successor can encounter and interact with them by travelling to their location.

The central principle is: **the world remembers each life, but a successor does not automatically become that character or inherit all their personal knowledge and power.**

## Existing foundation

The current code already provides:

- A Character model with name/title, HP, Might, Insight, Endurance, inventory, conditions, experience, level, notes, and exploration state.
- Experience awards and automatic level advancement in character progression.
- Fatal combat, corpse creation, corpse inventory recovery, and death records in world history.
- Save/load with versioning and default values for compatibility.
- World inheritance that preserves most world state, corpses, NPCs, local maps, and history while creating a fresh character and resetting personal progression and faction reputation.

The design should extend these mechanisms. Important gaps include meaningful level-up choices, structured character-life records that outlive the bounded general history log, a distinction between personal knowledge and world knowledge, and a supported transition from active character to retired NPC.

## 1. Character creation and identity

A character has a persistent identity and a life-specific progression record. Initial creation should support:

- Name and initial title/epithet.
- A data-driven background that describes starting circumstances and gives modest, non-exclusive benefits.
- Attribute allocation and initial circumstances.
- Starting equipment and optional contextual details, such as a rumor or contact, when the relevant systems exist.

Use the current attributes—Might, Insight, and Endurance—as the initial foundation. Proposed starting balance: begin each attribute at 1, distribute two additional points, and cap starting attributes at 3. Treat these numbers as initial balance values that can be tested and tuned, not a permanent restriction on future progression.

Backgrounds should not lock a character into a class or activity. A former soldier could become a scholar or merchant; a healer could become a ruler or monster hunter. Titles may change in response to deeds, but the original background remains part of the life record.

Keep initial traits modest. Do not add large lists of personality traits until dialogue, relationships, or other systems can respond to them mechanically.

## 2. Character progression

Retain experience and levels, but make each level-up a meaningful choice instead of an automatic number increase alone.

- **Attributes** represent broad capabilities and should affect relevant actions.
- **Skills** represent domain-specific expertise, introduced when their activities have real gameplay support.
- **Growth choices** may improve an attribute, develop an available skill, or select a capability when that system exists.

Experience should reward meaningful outcomes across multiple styles of play. Existing combat and quest awards remain useful; exploration, investigation, social, economic, and leadership accomplishments should become valid sources as their associated mechanics arrive.

Skills may improve through meaningful use, training, teachers, discoveries, or deliberate practice. Repeating trivial actions indefinitely must not produce unlimited progression. Level thresholds and awards should start from the existing framework and be tuned through testing rather than replaced with an unnecessarily complex formula upfront.

Progression should broaden a player's options without making preparation, knowledge, local danger, and world context irrelevant. The game should not automatically scale every challenge to the character's level.

Implement progression in stages: first make level-ups meaningful and attributes useful, then add skills alongside their activities. Do not create a large set of empty skill tracks before the gameplay exists.

## 3. Self-directed lives and ambitions

Characters may have an optional long-term ambition and optional short-term aims. Examples include monster hunting, exploration, family life, trade, accumulating wealth, service to a faction, settlement-building, leadership, scholarship, craft mastery, or protecting a community.

These ambitions describe a direction, not a mandatory quest chain or character class. A character may change or abandon an ambition; wandering, responding to opportunities, or simply surviving must remain valid forms of play.

Goals can help surface relevant opportunities and track achievements, but each ambition should only be offered as a meaningful path once sufficient supporting activities exist. The system should not promise complete trade, family, or settlement playstyles before those mechanics have been implemented.

A character's life record should reflect what they actually did, not just the ambition they selected. Ambition and activity are connected but distinct: a player may become known for deeds that were not part of their original plan.

## 4. Death and permanent life records

Death must end the individual character's active life while leaving the world intact.

The death transition should resolve the fatal event, mark the character dead, transfer carried inventory to the corpse exactly once, record cause/location/world time, and create a permanent life record. It must be safe against duplicate execution or interrupted save transitions.

Add a structured, persistent record for each character life. It should retain at least:

- Unique character/life identity, name, title, and background.
- Start and end dates in world time.
- The life outcome (including death or retirement) and cause/location where relevant.
- Major achievements, completed ambitions, important decisions, and significant events.
- References to relevant people, places, possessions, and world outcomes where those systems support them.

The existing general world-history log can remain bounded. Life records must not disappear merely because older entries have been pruned. The corpse and life record should be cross-referenced so that the player can connect a former character's biography to their remains and belongings.

The end-of-life presentation should summarize meaningful deeds, important relationships or faction standing, the outcome and circumstances, and the material or social legacy left behind.

The first implementation should support the existing fatal-combat route. Natural death, illness, old age, and other causes should be introduced only alongside the health and life-span systems needed to support them coherently.

## 5. Retirement and retired characters as NPCs

Retirement is a supported alternative to death, not a disguised death state.

- A living character may voluntarily retire, ending active player control while remaining alive in the persistent world.
- Retirement should establish a location where the character resides. Initially this can be a suitable existing settlement or location; a full housing/property system is not a prerequisite.
- The retired character should be represented as a persistent NPC associated with their original identity and life record. Do not create an unrelated duplicate while leaving an active, controllable copy behind.
- When a successor travels to the retired character's location, they should be able to encounter and interact with them through the NPC/interaction systems available at that stage.
- The retired NPC should retain recognizable identity, relevant history, and appropriate personal knowledge. Their dialogue or behavior should be able to acknowledge their former life and the successor's relationship to that legacy where supported.
- Retirement must not automatically duplicate or transfer possessions, skills, reputation, quests, or knowledge to the successor. The retired character's belongings remain associated with that character or their established location until an explicit world rule changes ownership.
- Retirement records the end of the active player-controlled career, but the person's in-world story can continue. If later world simulation causes a retired character to move, retire further, or die, those are subsequent world events and should update the persistent record rather than erase the retirement event.

The first retirement implementation does not require sophisticated autonomous life simulation. The retired NPC may begin with a stable location and limited interactions; more detailed goals, relationships, schedules, aging, and independent decisions can be added through the multi-fidelity simulation architecture in #367.

After retirement, the player may begin a successor in the same world. Retirement should not require the retired character to be related to the successor; family ties and explicit heirs can be introduced later.

## 6. Inheritance and continuity across lives

Creating a successor must preserve consequential world state while giving the new character their own identity.

### World state that persists

- Geography, routes, local-map physical changes, and established discoveries represented in persistent world state.
- World time, major events, and consequences of previous choices.
- NPCs, factions, and their memories of past events.
- Corpses and their recoverable contents.
- Permanent character-life records and the world chronology.
- Lasting quest outcomes and world changes.

This extends the existing inheritance behavior rather than replacing it.

### Personal state that does not transfer automatically

A successor begins with their own identity/background, attributes, level/experience, conditions, inventory, ambitions, and personal relationships. They do not simply inherit the previous character's levels, skills, current equipment, active quest log, faction reputation, or undisclosed personal knowledge.

Equipment remains where it was left—in a corpse, storage, or another represented location—and can be recovered through the relevant world interactions. Retirement likewise does not duplicate a retired NPC's belongings.

Distinguish three categories as spatial and knowledge systems mature:

1. **Physical world state:** terrain, structures, routes, and other persistent changes.
2. **Recorded knowledge:** information preserved by a map, journal, letter, public record, monument, or another persistent medium.
3. **Individual knowledge:** personal observations, secrets, rumors, and leads that have not been recorded or otherwise shared.

Physical state persists. Recorded knowledge remains available through its actual medium. Individual knowledge is not magically transferred to a successor. This may eventually require a clear separation between shared physical local-map state and each character's personal discovery state; new systems should avoid entangling the two.

Factions should remember previous deeds without automatically granting the successor the previous character's personal reputation. Family, inherited property, inherited office, or recognition based on name/legacy may later provide explicit exceptions through dedicated systems, not implicit stat copying.

World chronology is authoritative across lives. A new character's personal action counter may start over, but dates and world events must never rewind or become ambiguous because a character's turn count resets.

## 7. Balance, integrity, and compatibility

The following are cross-system requirements:

- No single activity, especially combat, should dominate progression solely because other paths do not award experience.
- Repetitive low-value actions must not yield unlimited progression.
- Death has personal costs, but must not roll back the world.
- Retirement ends active control without erasing the retired person or duplicating them as an NPC.
- Items exist in one place at a time; death, retirement, succession, and save/load must not duplicate equipment.
- Reputation and knowledge remain distinct from recorded historical recognition.
- The world should retain meaningful danger and should not automatically scale every encounter to character power.
- New persisted fields should use versioned, backward-compatible save handling and sensible defaults for older saves.
- Death/retirement and succession transitions must remain correct across save/load and repeated inheritances.

## Suggested implementation sequence

1. Add persistent character-life records and connect death/corpse creation to those records.
2. Make progression choices meaningful and introduce data-driven starting backgrounds.
3. Add optional ambitions and achievements, backed only by implemented activities.
4. Add the retirement-to-NPC transition, including stable identity, residence, successor encounters, and persistence.
5. Formalize physical world state versus recorded and individual knowledge during inheritance.
6. Validate balance, backward-compatible saves, death/retirement transitions, corpse recovery, and repeated generations.

These steps are an implementation order, not a ceiling on simulation depth. Use the appropriate levels of fidelity defined in #367; initially stable/event-driven retired NPCs are compatible with richer simulation later.

## Acceptance criteria

- Characters with different backgrounds can pursue different lives without class-locked paths.
- Meaningful level-ups offer useful choices, and progression is not dependent on combat alone.
- A life record remains available after general world-history entries have been pruned.
- Death produces exactly one corpse and one life outcome; carried inventory is not duplicated.
- A retired character is no longer the active player character but remains a persistent NPC in the world.
- A successor can travel to the retired character's location and encounter/interact with them, with identity and history intact.
- Retirement does not create an accidental copy of the active character, or duplicate their belongings.
- New characters do not automatically acquire a previous character's progression, personal reputation, active quests, or unrecorded secrets.
- Previous lives' world changes, NPC/faction memories, world chronology, and life records persist across multiple generations.
- Save/load and legacy-save compatibility are maintained throughout the staged implementation.
- Tests cover distinct play styles, repetitive progression exploits, death, retirement, successor interactions, corpse recovery, and repeated inheritance.
