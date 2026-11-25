# Primary Player Experience: Exploration, Combat, and World Judgment

This specification describes the intended moment-to-moment and long-term player experience. It complements the [game vision](game-vision.md), [design principles](../design-principles.md), [NPC behaviour](npc-behavior.md), [world simulation](world-simulation.md), [real-time spatial combat](real-time-spatial-combat.md), and [challenge and difficulty philosophy](challenge-and-difficulty.md). It defines product direction, not implementation status or a commitment to deliver every capability at once.

## Goal

Create a character-focused dark-fantasy RPG in which exploration and combat are both central pleasures. Players should have a world worth discovering and meaningful freedom to act within it, including the freedom to behave destructively. The world, its inhabitants, and its institutions should respond according to what they know, what they value, and what they can actually do.

The game should not require every fight, crime, discovery, or life choice to belong to an authored quest. A character's own decisions can create stories just as consequential as authored content.

## 1. Exploration and combat are both central

Exploration lets the player discover unfamiliar places, inhabitants, cultures, dangers, conflicts, histories, and opportunities. It should reward curiosity through atmosphere, meaningful discoveries, and situations worth investigating, without requiring every stretch of wilderness to contain a reward or encounter.

Combat is not merely a tax paid to continue exploring, nor only a solution to quest obstacles. The act of fighting should itself be engaging through responsive real-time action, readable threats, timing, positioning, equipment, abilities, injuries, enemy behaviour, and consequential outcomes. Some encounters may be avoidable; others may be sought out. Not every fight needs to be morally weighty or tied to a major narrative.

These pleasures reinforce one another. Exploration reveals what exists and what might be at stake; combat and other actions let the character change those circumstances. Neither should make the other redundant.

## 2. Freedom to act, including destructive choices

Players may pursue authored stories, personal ambitions, relationships, work, travel, conflict, or no defined long-term goal at all. They may help people, ignore them, negotiate, intimidate, steal, attack, kill, flee, or escalate a conflict when the character and situation make those actions plausible.

The game should not impose arbitrary protection on ordinary inhabitants solely to preserve a preferred story or moral outcome. Where an action is physically and contextually possible, the design should generally allow the attempt and let the relevant systems resolve it. True impossibilities may be blocked; risky, unlawful, cruel, or socially unacceptable actions are not automatically impossible.

This freedom does not promise that every action is easy, consequence-free, or supported by a bespoke narrative branch. It means the world responds through its established rules rather than silently refusing plausible player agency.

## 3. The world judges characters through its inhabitants

Characters and groups form opinions based on their values, interests, relationships, experiences, and available information. Responses may include fear, admiration, hatred, respect, distrust, loyalty, avoidance, opportunism, or a desire for revenge.

Judgement is contextual rather than universal. The same character may be feared by one settlement, celebrated by a rival faction, hated by victims' families, respected by a group that benefits from their actions, and unknown in distant regions. A character's reputation is therefore layered across individuals, communities, factions, settlements, and regions.

Reputation and response must be knowledge-dependent:

- Direct witnesses may identify the character and remember what they saw.
- Survivors may provide accounts, seek help, flee, conceal information, or pursue revenge.
- Physical evidence and other clues may support or contradict reports.
- Travellers, messengers, faction contacts, rumours, and investigations may spread information over time.
- Reports may be delayed, incomplete, biased, mistaken, or deliberately deceptive.
- Authorities may suspect a character without having enough evidence to identify or locate them confidently.

The simulation must distinguish what happened in the authoritative world state from what each person or institution knows, suspects, or believes. Unwitnessed actions must not automatically become public knowledge.

## 4. Violence and organized responses

Violence can affect individuals, families, settlements, organizations, factions, trade, local leadership, and regional stability. The scale and form of the response depend on the event and the people or institutions affected.

A response is not a mandatory wanted-level ladder. Depending on local law, authority, resources, priorities, evidence, and relationships, possible reactions include:

- Individuals avoiding, confronting, reporting, or seeking revenge against the character.
- Victims' families, companions, or allies pursuing their own response.
- Local guards investigating, warning, detaining, or attempting to arrest the character.
- Patrols, bounty hunters, mercenaries, or faction forces searching for or pursuing them.
- Leaders organizing a larger force when the threat, political stakes, and available capabilities justify it.
- Rival groups exploiting the event, sheltering the character, recruiting them, or using the incident for their own purposes.

A large-scale hunt—including a force numbering in the hundreds—is a possible outcome, not a scripted guarantee or a universal response to a fixed number of kills. It must arise from the world having a reason and the actual capacity to organize such an operation.

Organized pursuit requires more than spawning enemies near the player. Responding groups need credible information, identification, leadership, orders, travel, coordination, equipment, supplies, and time. Their effectiveness depends on those capabilities and on the quality of what they know. They may lose the trail, act on false information, disagree, be delayed, or abandon the pursuit when priorities change.

## 5. Attrition, escape, and defeat

Prolonged conflict should be able to wear a character down. Where the relevant survival and combat systems support them, injuries, fatigue, depleted supplies, equipment damage, lost allies, and accumulated pressure can make continued resistance increasingly difficult.

This should create meaningful decisions rather than a predetermined execution. The player may fight, retreat, hide, mislead pursuers, seek shelter, negotiate, surrender, or attempt to escape by another route when circumstances allow. Pursuers should be dangerous because of their actual numbers, coordination, equipment, information, and persistence—not because the game secretly scales their statistics.

A character who starts a violent confrontation may ultimately be captured or killed after surviving repeated engagements. That outcome should follow from the situation and established rules, not from an invisible script that decides the player must lose.

The world's challenge remains world-driven. Authorities and enemies do not become stronger simply because the player has performed well, and they do not become weaker to ensure the player survives.

## 6. Persistence, death, and inheritance

Consequential actions should leave the appropriate traces in persistent world state. Depending on the event and supporting systems, these may include deaths, injuries, changed relationships, damaged or abandoned places, altered leadership, faction hostility, public accounts, ongoing investigations, active pursuit, and changes in local or regional conditions.

Character death may end the current life while leaving relevant consequences in the world. A successor may encounter the former character's reputation, enemies, allies, family consequences, property, deeds, and unfinished conflicts where those things plausibly persist. Inheritance must continue to distinguish world-persistent history from private knowledge, temporary conditions, and character-specific state.

A successor should not automatically know everything the previous character knew. They may instead learn about the former character through people, records, reputation, surviving institutions, and direct encounters.

## 7. Save and reload are player-controlled tools

Saving and reloading are outside the character's knowledge and are not part of the simulated world's judgement. Players may preserve a save, experiment with a dangerous choice or destructive spree, and reload to return to the earlier timeline.

The game should not moralize about this behaviour, secretly alter the world because a player reloaded, or treat the player's use of save files as an in-world event. Within any loaded timeline, however, the world should respond consistently to the actions that actually occurred in that timeline.

Save reliability and recovery remain governed by the project's persistence principles. This specification does not prescribe save-slot design, rollback mechanics, or changes to save policy.

## 8. Design boundaries

- Do not reduce reputation to one universal morality or notoriety number.
- Do not make every violent act trigger an automatic, omniscient, identical response.
- Do not promise a large military pursuit unless the world can plausibly gather and sustain it.
- Do not make pursuit a scripted boss encounter or a hidden difficulty adjustment.
- Do not guarantee that every character can escape, defeat, or negotiate with every response.
- Do not make combat enjoyable only when it advances a quest.
- Do not let supporting systems turn this experience into a collection of disconnected simulators.
- Do not claim planned behaviour is already implemented; implementation status and sequencing belong in the roadmap and relevant system plans.

The governing test is whether a player's actions create understandable, persistent, and world-appropriate reactions while preserving meaningful freedom to explore, fight, retreat, fail, die, and begin another life in the same evolving world.
