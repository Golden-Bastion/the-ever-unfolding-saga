# Real-Time Spatial Combat

> Design record migrated from [GitHub issue #252](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/252). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

Future design/specification issue. This issue turns the combat portion of #228 into a concrete system design that can later be implemented incrementally. It should remain a planning document until the combat architecture and rules are sufficiently defined.

## Goal

Design a real-time, top-down 2D combat system built around player decisions and timing rather than manual pixel-perfect movement.

The player should control attack style, attack timing, defense, target decisions, and combo execution. The combat system should handle positioning and combat choreography so the player and enemies remain in active motion around one another.

The system must preserve the game's RPG/stat foundations and remain compatible with the existing authoritative gameplay model.

## Core combat model

- Entering a combat encounter establishes a target, initially auto-locking to the nearest appropriate enemy.
- The player remains free to move normally while exploring the environment outside combat.
- During combat, positioning is assisted/automated by the combat system. The player should be moved around the target as part of attacks, reactions, disengagements, and other combat choreography rather than requiring direct manual positioning.
- The fight should feel like a continuous exchange instead of two stationary entities taking turns until one reaches zero HP.
- The player should still have meaningful agency through attack choice, timing, defense, target selection, and combo continuation.

## Initial attack styles

The first weapon is a sword, with three attack styles:

### Strong

- Slower, committed attack.
- Usually a directional slash or thrust.
- Higher damage/impact than Fast.
- Intended to be useful against tougher opponents, openings, or defensive states.
- Higher commitment and therefore greater risk when mistimed.

### Fast

- Faster and weaker than Strong.
- Slash or thrust that also changes the PC's position.
- Used as a hit-and-run / repositioning attack.
- Should support maintaining combat tempo and chaining into further actions.

### Group

- Arcing slash aimed toward a group of targets.
- Moderate/specialized rather than simply a midpoint between Strong and Fast.
- Intended to threaten multiple nearby enemies.
- Can be inefficient against some single tough targets and poor against situations where its arc/timing is unfavorable.
- Particularly valuable against swarms of weaker enemies.

The three styles should have distinct timing, movement, animation, hit coverage, risk, and combo behavior.

## Timing and combos

- Attacks should have synchronized startup, active/hit, and recovery phases.
- Each attack should provide a timing window for continuing a combo.
- The UI should visibly and/or audibly hint at the appropriate moment to press/click.
- Desktop input should support mouse left-click for attacking.
- Android should use three touch buttons, one for each attack style.
- Better timing should produce better combat outcomes and presentation.
- Higher combos (roughly 3–4+ as an initial design direction) may increase damage and/or unlock stronger effects, animations, music layers, and severity.
- Flawless/precise execution should provide an incentive to play skillfully instead of repeatedly using the safest basic attack.
- High-level combo execution should be rewarding without becoming mandatory for basic survivability.

## Defense

Combat should support a block/parry model alongside attacks.

The final input, timing window, movement behavior, and interaction between defense and the three attack styles still need to be designed.

Defense should be skill/timing based and should interact with enemy attack timing, positioning, and combat capability without becoming excessively difficult or trivial.

## Enemy encounters and AI

Encounter composition should not be hard-coded to a single fixed enemy count.

Support, as appropriate:

- lone enemies;
- small groups of roughly 2–3 stronger enemies;
- swarms of weak enemies;
- enemies that enter or emerge during an encounter;
- bosses or other unique major encounters.

The practical simultaneous-entity limits should be driven by gameplay readability, presentation, and Android performance rather than an arbitrary universal cap.

Enemy behavior should be designed around readable combat roles and varied attack/defense rhythms so encounters can be difficult without simply scaling HP/damage.

Potential role families include aggressive, defensive, skirmishing, swarm-oriented, heavy, and boss behaviors, but these are design directions rather than a fixed implementation requirement.

## Hit and injury model

- The combat system should support individual body-part states without requiring separate physical hitboxes for each body part.
- Body-part outcomes may be inferred from attack direction, defender orientation, attack type, and severity.
- Hit results should expose severity rather than only subtracting generic HP.
- Design toward a progression such as ordinary wound, serious wound, severe injury, and catastrophic injury.
- Severe injuries should be mechanically meaningful, not purely visual.
- Examples of future consequences could include reduced effectiveness of an injured arm/hand or impaired movement from leg injuries.
- This should leave room for later systems such as prosthetics, magic, treatment, and other ways of restoring lost capabilities.

Exact thresholds and formulas should be defined during the detailed combat-balance/design pass.

## Combat presentation

Combat events should be structured so presentation is driven by gameplay results rather than by text strings.

A future event model should support concepts such as:

- AttackStarted
- AttackConnected
- DamageTaken
- Blocked
- Parried
- HeavyHit
- PlayerWounded
- CatastrophicInjury
- EnemyDefeated

The presentation layer can then react with animation, movement, sound, music, blood, screen effects, and textual feedback while the simulation remains authoritative.

## Blood and injury effects

### Ground blood

- Blood should appear in world space around impacts.
- Blood can accumulate during an encounter.
- An area may eventually become heavily stained without requiring unlimited new layers.
- Blood remains in the location for several in-game days after combat.
- It should then dry, fade, or otherwise decay according to world/platform rules.
- The system should avoid unbounded per-impact visual entities and should support pooling, aggregation, saturation, or other scalable approaches.

### Screen blood

- Hits on the PC can produce temporary screen-space blood/splatter.
- The screen should progressively gain a bloody/red edge effect as the PC's injury worsens.
- Individual hits may temporarily intensify the effect.
- Persistent screen effects should represent the PC's current injury state rather than merely the fact that any hit occurred.

### Severe injuries and gore

- Ordinary hits should have restrained feedback.
- Higher-severity hits should produce stronger reactions and presentation.
- Extreme attacks may visually depict severe wounds, ripping apart, dismemberment, or other catastrophic outcomes.
- These effects should be grounded in an underlying severity/injury result instead of being independent cosmetic logic.

## Corpses and aftermath

Defeated enemies should leave persistent environmental consequences.

- Corpses should remain in the location for several in-game days.
- Later world simulation/presentation may imply them being eaten by animals, looted, removed, or otherwise disturbed.
- Not every aftermath step needs to be literally simulated; some can be represented as visual state changes or later events.
- Corpses and blood should therefore be treated as part of short-lived world state rather than effects that disappear when the encounter UI closes.

## Music/audio integration

The combat system should expose enough structured state/events for the future audio system (#230) to react to combat quality and intensity.

Examples include:

- entering combat;
- escalating combat intensity;
- successful combo continuation;
- high/flawless combo states;
- heavy/severe hits;
- victory;
- defeat.

Combo progression may layer or otherwise intensify combat music.

The combat implementation should not directly own audio-file management.

## Platform considerations

### Desktop

- Mouse left-click attacks.
- Keyboard/other existing semantic input remains available for navigation and other actions.

### Android

- Landscape presentation.
- Three persistent/clear touch attack controls for Strong, Fast, and Group.
- Timing should be readable on touchscreens.
- Visual effects, particles, persistent blood, corpses, and entity counts should have platform-aware presentation budgets.
- Android may use faster visual decay and/or stronger aggregation of world effects while preserving equivalent underlying gameplay state.

## Architecture requirements

The combat design should keep gameplay rules separate from Bevy presentation.

Conceptually:

Combat decision
-> attack/movement state
-> animation/timing
-> hit resolution
-> damage/injury result
-> structured combat presentation events

Presentation/audio systems consume those results and handle:

- animation;
- movement/choreography visuals;
- hit reactions;
- blood and gore;
- screen effects;
- text feedback;
- sound/music.

Do not make renderer-specific code authoritative over combat outcomes.

Attack definitions should become data-driven enough that the system can eventually support additional sword moves and other weapon types without rewriting the combat state machine.

## Design work required before implementation

Define and document:

- combat state machine and exchange/phase model;
- exact Strong/Fast/Group attack timings;
- combo progression and failure/reset rules;
- how directional attacks determine direction;
- exact defense controls and timing;
- enemy attack patterns and AI role behaviors;
- target-lock and retargeting rules;
- automatic positioning/choreography constraints;
- interaction between positioning assistance and character stats;
- injury severity calculation and thresholds;
- body-part consequence rules;
- death/catastrophic injury states;
- persistent blood/corpse world-state representation and decay;
- animation timing contracts;
- structured combat presentation events;
- desktop and Android input mapping;
- performance budgets and scalability rules;
- initial asset requirements and sprite/animation conventions;
- how combat transitions back into the existing textual/gameplay flow.

## Non-goals

- No immediate implementation of the complete spatial combat system.
- No replacement of the existing RPG/stat systems.
- No requirement to make every enemy use the same attack set.
- No requirement to fully simulate animal scavenging/looting/removal in the first implementation.
- No requirement to finalize the art pipeline before the combat architecture is designed.

## Relationship to #228

#228 remains the broad concept for the future hybrid world/exploration/investigation/action-combat direction.

This issue should contain the detailed combat specification and can later be split into implementation milestones/tasks once the design is stable.

## Detailed combat specification — initial implementation contract

The following turns the design requirements above into an implementation-oriented baseline. These values are initial tuning values, not permanent balance values; they should be data-driven so they can be adjusted without changing the combat state machine.

### Combat state machine

A combat encounter is a continuous state machine rather than a sequence of menu turns.

```
Engage
  -> Approach / Position
  -> Ready
  -> Attack Startup
  -> Attack Active
  -> Attack Recovery
  -> Combo Window
       -> next attack / defense / reposition
       -> Recovery complete -> Ready
  -> Enemy Attack Telegraph
       -> Enemy Attack Active
       -> Resolution
       -> Ready / Player Reaction
  -> Victory / Defeat / Disengage
```

Only one player action may be authoritative at a time. Presentation may animate several entities simultaneously, but gameplay resolution remains driven by the combat state and its timers.

The combat controller should maintain, at minimum:

- current target;
- player combat position relative to target;
- current player action;
- current action phase and phase timer;
- current enemy action/telegraph;
- combo count and combo quality;
- current defense state;
- pending/last structured combat result;
- encounter members and their combat state.

### Attack timing

The first sword should use three data-driven attack definitions.

| Style | Startup | Active | Recovery | Primary role |
| --- | ---: | ---: | ---: | --- |
| Strong | 0.45 s | 0.12 s | 0.55 s | committed damage/opening attack |
| Fast | 0.18 s | 0.10 s | 0.32 s | tempo/repositioning |
| Group | 0.32 s | 0.16 s | 0.48 s | multi-target control |

These are starting values for tuning, not animation requirements. Animation timing should consume the same phase contract so gameplay and presentation cannot drift apart.

Each attack also exposes:

- effective range;
- hit arc/cone;
- movement displacement;
- impact/poise value;
- damage modifier;
- combo transition eligibility;
- combo-window duration;
- interruption rules;
- defense interaction.

### Attack direction and positioning

The player does not need pixel-perfect directional movement during combat.

The combat system maintains a target-relative position and automatically moves the player toward a valid combat position when an action requires it. The player chooses the meaningful combat decision; the system handles the small positional choreography needed to execute it.

For the initial sword:

- Strong uses the current target-relative facing and commits toward the target.
- Fast moves through or around the target enough to change combat position and creates the strongest repositioning effect.
- Group pivots around the current combat position and sweeps a broad arc rather than requiring manual aiming.

The player's current relative position/facing can influence which body region is exposed and the presentation direction of an attack, but no manual pixel-perfect movement is required.

Target-relative movement must have constraints so the system never teleports the player, crosses impossible geometry, or repeatedly oscillates around a target.

### Combo model

The combo counter represents successful continuation of attacks, not merely repeated button presses.

After an attack reaches its valid continuation point, a short combo window opens. The next attack pressed inside that window continues the combo. Pressing outside the window, being interrupted, or taking a decisive defensive reaction resets the combo.

Initial quality bands:

- Missed timing: no continuation; normal recovery.
- Good timing: continue combo normally.
- Precise timing: continue combo and increase combo quality.
- Sustained precise timing at roughly 3–4+ hits: unlock stronger presentation and a modest combat benefit.

The first implementation should avoid making high-combo execution mandatory for ordinary survival. Combo rewards should improve efficiency and presentation rather than becoming the only viable strategy.

Attack transitions should be data-driven. The initial sword may allow all three styles as follow-ups, but individual transitions can later be restricted when a move or weapon makes a transition inappropriate.

### Defense

Defense is an active timing decision rather than a permanent passive damage reduction.

The initial model is:

- Block: reduces incoming damage while the defense window is active.
- Parry: a short timing window around the enemy's impact that can negate the hit and create an opening.
- Failed parry: normal hit resolution applies; there is no free defense for mistiming.
- Successful parry: briefly interrupts or exposes the attacker and permits a strong counterattack window.

Initial tuning targets:

- defense startup: about 0.10 s;
- parry window: about 0.12 s around impact;
- post-defense recovery: about 0.30 s.

The exact controls are platform-specific but map to the same semantic actions. Desktop can use a keyboard/mouse defense input; Android should expose a persistent defense control beside the three attack controls.

### Enemy attacks and readable AI

Enemies should expose an attack sequence through the same timing model:

`Telegraph -> Startup -> Active -> Recovery`.

The telegraph is part of the gameplay contract, not merely an animation. Its duration and visual/audio representation communicate when a defense or repositioning decision is possible.

Enemy roles should modify timing, range, pressure, and behavior rather than simply multiplying damage/HP.

The first implementation should be able to represent at least:

- aggressive attacker;
- defensive/heavy attacker;
- swarm/light attacker.

Boss-specific behavior can extend the same model later.

### Targeting and encounter composition

Entering combat selects an initial target using the nearest valid hostile target.

During combat:

- the current target remains locked while valid;
- defeated/invalid targets are removed from consideration;
- target selection can be changed explicitly when multiple enemies are present;
- the system should avoid unexpectedly changing targets simply because another enemy becomes closer;
- if the current target disappears, retarget using deterministic encounter rules.

Single-target encounters should remain simple. Multi-target encounters use the same state model, with Group attacks resolving against every valid target inside their defined arc/range.

### Hit resolution and injury

Combat resolution should produce a structured result instead of directly generating presentation text.

A result should contain enough information to determine:

- attacker and defender;
- attack style;
- hit/miss;
- blocked/parried;
- damage;
- impact/severity;
- relative attack direction;
- affected body region;
- resulting injury state;
- defeat/catastrophic state when applicable.

Body-region inference should remain simulation-level logic. It does not require separate physical hitboxes for every limb.

Initial severity bands remain:

`None -> Ordinary -> Serious -> Severe -> Catastrophic`.

Exact thresholds should be derived from weapon power, character capability, defense, attack quality, and existing RPG attributes during balancing rather than hard-coded into the presentation layer.

### Structured combat events

The combat system should emit typed/structured events such as:

- `CombatStarted`
- `TargetChanged`
- `AttackStarted`
- `AttackPhaseChanged`
- `AttackConnected`
- `AttackMissed`
- `Blocked`
- `Parried`
- `DamageTaken`
- `InjuryApplied`
- `ComboChanged`
- `EnemyDefeated`
- `CombatVictory`
- `CombatDefeat`
- `CombatDisengaged`

Textual messages, animations, particles, blood, screen effects, and audio should consume these events rather than being generated by combat rules.

### Presentation/UI boundary

The future combat interface should replace the current three-item action menu with a persistent combat control surface.

The graphical gameplay HUD from #279 should provide the stable world/HUD shell; combat may temporarily take control of the primary-action region while preserving the player HUD and world context where useful.

The initial combat control surface should contain:

- Strong;
- Fast;
- Group;
- Defense;
- target/retarget control when multiple enemies exist;
- a compact timing/combo indicator;
- contextual combat feedback.

Desktop and Android use the same semantic combat actions. Android gets three clear attack buttons plus defense; desktop can use mouse/keyboard bindings without changing the underlying action model.

The combat screen should not require the old Ratatui-style stacked panels. Combat presentation should be spatial and action-oriented, with the world/actors occupying the visual focus and the controls kept compact.

### Relationship to the current implementation

The current `game::combat::resolve_action` turn-based `Attack / Guard / Flee` loop is the compatibility baseline that must remain functional until the real-time system is implemented. It should not be incrementally distorted into a hybrid of the two models.

When implementation begins, introduce the new combat state machine behind the existing gameplay/domain boundary, then migrate the Bevy combat presentation/input to consume the new state/events. The existing textual combat result semantics should be preserved where they remain useful.

This makes the migration a deliberate replacement of the combat encounter model rather than an accumulation of special cases in `bevy_combat.rs`.

### Implementation sequence

The eventual implementation should be split into independently testable stages:

1. Data-driven attack/defense definitions and timing state machine.
2. Single-target continuous combat with Strong/Fast/Group and deterministic enemy attacks.
3. Target-relative positioning/choreography.
4. Combo timing and quality.
5. Structured combat results/events.
6. Multi-target encounters and Group attack resolution.
7. Injury/body-region consequences.
8. Bevy combat presentation and desktop/Android controls.
9. Blood, corpses, hit reactions, and other persistent presentation.
10. Audio integration through the future audio event boundary (#230).

Each stage should preserve the authoritative gameplay/presentation separation and include focused tests for the rules it introduces.
