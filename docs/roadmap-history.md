# The Ever-Unfolding Saga

## Roadmap

## Pre-v0.8.0

- the Rust project exists
- the game name is finalized
- the design plan and roadmap are written
- the codebase has a clean folder structure

- save/load is implemented through a versioned JSON save file
- world state model
- entity IDs
- basic character creation
- text UI
- a simple travel/history/death loop is playable
- the recovery action is meditate, and it heals plus saves when the player is safe

- the player can move through a generated world
- the player can die
- the death screen offers world reset or world inheritance
- safe meditation and saving are tied to world danger
- a basic threat interaction exists
- threat resolution now has a real combat loop
- defeating a threat can clear danger and leave a trophy item behind
- item pickups now notify the player directly
- major action results now pause on screen so the text does not get skipped

- corpses are created when a character dies
- items drop into the corpse inventory instead of disappearing
- corpse remains persist in the world after inheritance
- locations show visible remains
- the player can search remains and recover items
- history records death, looting, and aftermath events
- danger can be cleared by defeating a threat, changing the world state
- faction reputation is tracked and changes when the player completes the shrine quest
- NPCs remember important events and react differently on later visits
- hidden quests stay out of the quest log until they are actually offered
- the first simple quest is active and rewards the player

- the region expanded from 3 to 8 connected locations
- three distinct dangerous locations now have different enemy profiles
- additional enemy types and location-specific trophies were added
- the item pool now contains multiple trophies and faction rewards
- the faction roster expanded to three factions
- the NPC roster expanded to five named NPCs with persistent memories
- the quest system now supports multiple active quest chains using the same offer/turn-in rules
- location arrival scenes provide distinct atmospheric events and descriptions
- campaign bootstrap now backfills new content into older saved worlds instead of requiring a reset
- campaign content definitions are centralized in the bootstrap layer to prepare for the later data-loading phase

## v0.8.1 Quest-system maintenance

- new characters no longer inherit the previous character's personal quest log
- completed quests are stored as persistent world deeds so later characters do not receive duplicate quests
- existing completed quest records are migrated into persistent world deeds during bootstrap/inheritance
- the main location description no longer prints a duplicate quest summary; Quest Log remains the dedicated quest view
- required quest items are consumed when a quest is successfully turned in
- fixed campaign quest bootstrapping so the shrine quest is not recreated on every load

## v0.8.2 Quest interaction and reputation maintenance

- quest offering and turn-in now use an explicit Talk action with NPC selection
- same-location quest completion now works immediately after the threat is defeated
- NPCs no longer automatically offer or complete quests merely because the player entered a location
- new characters start with zero faction reputation while persistent faction memories remain
- quest completion grants +5 faction reputation and the associated faction reward contributes another +5 while held
- inherited reward items restore their +5 faction contribution when the new character recovers them from a previous life

## v0.9.0 Data-loading foundation

- added a runtime-loaded campaign content file under `data/base_content.json`
- seeded the world from loaded content instead of hardcoded location bootstrap
- added validation for duplicate content IDs and broken content references
- moved NPC, faction, quest, atmosphere, and encounter definitions into the content pack
- added stable content IDs to the base campaign data
- kept existing save data compatible through defaulted quest fields

## v0.10.0 Mod loading foundation

- added mod discovery under `data/mods`
- loaded mod content after the base content pack
- merged content by stable IDs and location keys so mods can replace or extend data safely
- kept broken mod files from crashing the game by reporting load warnings
- moved quest identity handling onto stable content IDs in the runtime paths

## v0.11.0 Optional visuals foundation

- added optional ASCII portraits for NPCs
- added optional location scene art
- added optional item illustrations for rewards and trophies
- kept visuals fully optional so the game still works as text-only when assets are missing

## v0.12.0 Polish and balance pass

- simplified the main status screen to remove repeated world-mode and history-count clutter
- removed repeated location and inventory ASCII art from the always-visible screens while retaining contextual visuals during interaction and item acquisition
- tightened combat output so attacks and damage resolve in one readable sequence instead of pausing after every hit
- added explicit combat victory and quest reward summaries
- added clearer save-load validation warnings for broken runtime references and inconsistent completed quest deeds
- adjusted dangerous encounters to reduce early-game damage spikes while preserving different threat durability
- reduced repetitive travel/location presentation by keeping the recurring status display focused on actionable information

## v0.12.1 Quit-screen polish

- replaced the plain quit confirmation with a randomized atmospheric exit screen
- added four original dark farewell variants with contextual ASCII art
- kept quitting non-destructive: `N`, `No`, or Enter returns to the game without changing state

## v0.13.0 Gameplay polish and balance completion

- added an atmospheric end-of-life screen with a concise summary of death location, remembered deeds, faction standing, and items left on the corpse
- made previous-life traces discoverable through corpse recovery and NPC memories without restoring the dead character's quest log
- added a small set of randomized, non-quest travel events to make repeated journeys less predictable without changing important quest outcomes
- added explicit corpse/legacy feedback so inherited equipment is distinguished from information that must be rediscovered
- retained the existing faction reputation split and prevented new characters from inheriting old reputation

## v0.14.1 Character progression, NPC availability, and Time foundation

- added Might, Insight, and Endurance attributes with level-based progression
- added experience gains from combat, quests, and discovery, with player-chosen level improvements
- added a character sheet for progression and condition visibility
- added Wounded, Exhausted, and Well-rested conditions with gameplay effects
- added a persistent world time cycle with named day/night portions, day count, and an east-to-west ASCII sun/moon track
- replaced the time track with a compact two-line celestial cycle and a clear east-to-west indicator line
- kept the sun and moon readable with plain Unicode symbols while hiding the internal time variable from player-facing text
- improved unavailable NPC feedback so it explains whether it is too early or too late and hints when to check again
- made travel, combat, searching, journaling, talking, and meditation advance hidden time portions
- made meditation duration player-selected and tied healing directly to time spent
- added time-sensitive NPC availability and travel atmosphere changes
- kept progression and conditions character-specific while world time persists through inheritance
- added v1 save migration for the new progression/time fields
- added small time-sensitive travel variation to exercise the new systems

## v0.14.2 Location scene art presentation

- rendered optional location scene art during arrival scenes so the player sees the place before the descriptive text
- kept scene art fully optional and text-only fallback intact when no art is defined
- reused the existing atmosphere and NPC scene flow so visual content layers cleanly onto the text systems

## v0.16.0 Keyboard-driven interaction pass

- replaced the blocking number-entry menus with keyboard-driven selection using arrows, Enter, Esc, and number shortcuts
- added raw-mode text input so prompts work directly inside the TUI without falling back to scrollback interaction
- changed pause handling to a single-key confirmation instead of Enter-only input
- updated the on-screen control hint to match the new keyboard flow

## v0.17.0 Responsive ratatui renderer

- migrated the terminal renderer to ratatui
- added width/height-aware compact rendering for narrow vertical screens
- kept the existing gameplay flows working inside the new screen shell
- bumped the project version to v0.17.0

## v0.17.1 Mobile portrait UI cleanup

- expanded compact-mode detection so tall, narrow terminals no longer get forced into the desktop split layout
- made prompt overlays use more of the available screen space on compact terminals
- added scrolling window behavior for menu overlays so longer option lists stay readable on smaller screens

## v0.17.2 Monochrome prompt cleanup

- switched the UI styling to monochrome gray/white borders and highlights
- moved the pause prompt out of the center overlay so result text stays visible
- reduced the choice popup footprint so it does not bury the rest of the screen as aggressively

## v0.17.3 Docked prompt layout cleanup

- moved prompt and confirmation dialogs into a reserved bottom panel instead of drawing them over the rest of the screen
- kept the main dashboard visible while choices, pause prompts, and quit confirmations are active
- tightened the prompt layout so messages and results stay readable behind the prompt flow

## v0.17.4 Turn-based result cleanup

- removed the duplicate top-of-screen game-state summary so the header no longer repeats the Status panel
- changed the Messages panel into a short-lived Result panel that is cleared when the player starts a new choice
- kept action outcomes visible only for the current turn instead of letting old text pile up indefinitely
- tightened the header layout to give landscape screens more usable space

## v0.17.5 Main-screen cleanup

- removed the empty top header box instead of leaving it as a hollow frame
- removed debug-style location exits and people listings from the main dashboard
- removed the character name and faction reputation from the main dashboard
- moved faction reputation into the character sheet
- moved location arrival art and atmosphere into the Location panel instead of the Result panel
- fixed the quit confirmation prompt so it no longer prints a stray `>` on separate lines

## v0.17.6 UI polish and health gauges

- replaced plain-text player HP rendering with a ratatui `LineGauge`
- added an enemy health `LineGauge` during combat
- used blood/dark-red player health and dark-purple enemy health fills while retaining the monochrome UI elsewhere
- kept combat health in the dashboard state so it updates while combat choices are rendered
- normalized panel rendering and collapsed border overlap across compact and wide layouts
- removed redundant combat HP text output and simplified the result display
- bumped the project version to v0.17.6

## v0.18.0 Event system foundation

- added a reusable event runtime in `src/events.rs`
- added stable event IDs, triggers, weights, chance gates, conditions, effects, and cooldowns to campaign content
- migrated the four existing travel events into `data/base_content.json`
- persisted event cooldowns as part of the world state with serde defaults for older saves
- added content validation for event IDs, triggers, weights, effects, chance ranges, and condition references
- added unit tests covering event conditions and cooldown eligibility
- removed the old hardcoded random travel-event branch from `game.rs`

## v0.18.1 Event validation and persistence test coverage

- reject invalid individual event definitions instead of loading them into the runtime
- emit warnings identifying each rejected event and the exact validation reasons while continuing to load valid content
- reject duplicate event IDs without overwriting previously accepted events
- added save/load coverage for persistent event cooldowns
- added world-inheritance coverage confirming event cooldown state persists with the inherited world
- added content filtering tests for invalid events and duplicate event IDs

## v0.19.0 Event & world memory integration

- cached campaign content in runtime state so event processing no longer reloads content for every trigger
- added structured history entries for narrative and event records, including event ID, location, and outcome data
- recorded executed events automatically in persistent world history
- added history-aware event conditions through prior event IDs
- validated prior-event references and rejected invalid event definitions without preventing valid content from loading
- rehydrated runtime campaign content after save loading without storing campaign content in save files
- added tests for structured event history, history-based conditions, runtime content rehydration, and existing persistence behavior

## v0.19.1 Save compression and character-specific filenames

- save files are gzip-compressed while retaining the existing JSON payload format
- save filenames now include a sanitized character name (`ever_unfolding_saga_save_<character>.json.gz`)
- legacy uncompressed `ever_unfolding_saga_save.json` files remain loadable
- startup discovers character-specific saves and preserves the selected save path
- added tests for compressed round-tripping, legacy loading, invalid gzip data, filename sanitization, and character-specific paths

## v0.20.0 State-aware event conditions

- added faction reputation conditions with minimum and maximum reputation thresholds
- added required inventory-item conditions
- added required active-condition checks
- validated referenced factions and invalid reputation ranges during content loading
- kept event conditions backward-compatible through serde defaults
- fixed mod event validation so a mod event may safely reference an existing base event in `prior_event_id` conditions
- added focused runtime and content-validation tests for reputation, inventory, condition, and cross-content event requirements
- added the `event.market-recognition` campaign event to demonstrate reputation-gated narrative content

## v0.20.1 Test-failure repair and runtime flow restoration

- restored the startup/load control flow that was accidentally truncated in v0.20.0
- restored campaign bootstrap for factions, NPCs, and quests using stable content references
- restored main-menu dispatch, loaded-state validation, NPC/faction/quest lookup helpers, and character creation/inheritance prompts
- fixed save-path handling when a character dies and a new character inherits or starts a new world
- removed the unused diagnostic macro and resolved the warnings reported by the v0.20.0 test run

## v0.20.2 Persistence test fixes

- normalized gzip decompression failures to `io::ErrorKind::InvalidData` so malformed compressed saves have a stable public error category
- treated filename components containing no alphanumeric characters as `unnamed` after sanitization
- removed two unused runtime helpers reported by the test build as dead code

## v0.22.0 Start and quit screen overhaul

- added a dedicated start screen instead of immediately entering new-game creation or save loading
- made `New Game`, `Load Game`, and `Quit` explicit start-screen choices
- only show `Load Game` when a compatible save or legacy save exists
- added a dedicated save-selection screen instead of automatic loading or Y/N load confirmation
- kept character/world creation behind the start-screen flow rather than making it the initial screen
- added dedicated full-screen menu rendering so start, load, creation, and quit flows do not render the gameplay dashboard underneath them
- changed quit handling to use the existing randomized farewell sentences as the actual choices instead of Y/N confirmation
- removed save-on-quit behavior; saving remains tied to safe meditation
- made death-screen quitting non-destructive as well

## v0.22.1 Dark start-screen art variants

- added four randomized dark start-screen ASCII variants
- paired each start-screen artwork variant with an approved atmospheric sentence
- kept start-screen art limited to the start flow; quit and death artwork remain unchanged for their own patches

## v0.22.2 Dark quit-screen art variants

- expanded the quit-screen artwork pool with five additional dark ASCII variants
- added matching farewell choices for gate, extinguished shrine, empty road, graveyard, and final-look imagery
- kept quit behavior non-destructive; selecting the leave choice exits without saving
- left the death-screen artwork untouched for the dedicated death-screen patch

## v0.22.3 Dark death-screen art variants

- added five new dark death-screen ASCII variants
- added distinct death-themed sentences for the new variants
- retained the existing three death-screen variants for a total of eight randomized variants
- kept the existing death summary, inheritance flow, and non-destructive quit behavior unchanged

## v0.23.0 Game Screen Architecture

- extracted start, load, character creation, quit, and death screen logic from `game.rs` into `game/screens.rs`
- reduced `game.rs` responsibilities to game orchestration, state management, and gameplay logic
- kept all gameplay behavior and screen flows unchanged
- no gameplay mechanics or data structures were changed

### Architectural Direction

This begins the gradual decomposition of the large `game.rs` module by responsibility.

Future refactoring should continue only where there is a clear separation of responsibility rather than splitting files purely to reduce line counts.

```
src/
├── main.rs
├── game.rs              # game orchestration / main loop
├── game/
│   ├── combat.rs
│   ├── actions.rs
│   ├── screens.rs
│   ├── character.rs
│   └── world.rs
├── content.rs
├── content/
│   ├── loader.rs
│   ├── definitions.rs
│   └── seeding.rs
├── events.rs
├── model.rs
├── persistence.rs
└── ui.rs
```

## v0.24.0 Gameplay Action Architecture

- extracted main-menu action definitions and dispatch targets from `game.rs` into `game/actions.rs`
- moved travel, meditation, NPC interaction, quest handling, inventory, journal, corpse recovery, progression, time/condition helpers, and death handling into the action module
- extracted the combat encounter loop and combat-specific helpers into `game/combat.rs`
- reduced `game.rs` to orchestration, campaign bootstrap/validation, dashboard rendering, and location-scene presentation
- kept save compatibility and existing gameplay behavior unchanged
- kept the new modules dependent on the existing model, persistence, event, and UI systems instead of introducing parallel state systems

## v0.25.0 Content Architecture

- split the monolithic `content.rs` into `content/definitions.rs`, `content/loader.rs`, and `content/seeding.rs`
- kept `content.rs` as a compatibility-facing module facade and preserved the existing content-loading entry point
- moved campaign data definitions and content validation into the definitions module
- moved world seeding and content-to-world translation into the seeding module
- moved base/mod loading, merging, event filtering, path discovery, and content-loading tests into the loader module
- corrected the embedded fallback content path for the new loader module location
- preserved campaign content formats, mod behavior, event validation, and save compatibility
- bumped the project version to v0.25.0

## v0.25.1 Game Runtime Architecture Cleanup

- moved campaign bootstrap and loaded-state validation from `game.rs` into `game/world.rs`
- moved dashboard rendering and location-scene presentation from `game.rs` into `game/presentation.rs`
- kept `game.rs` focused on application flow, menu dispatch, and orchestration
- reused the cached campaign content during location-scene presentation instead of reloading the content pack on every newly visited location
- preserved gameplay behavior, save compatibility, and the existing screen flow
- bumped the project version to v0.25.1

## v0.26.0 Game Runtime Loop Architecture

- moved the main gameplay loop from `game.rs` into `game/runtime.rs`
- kept `game.rs` as the top-level game entry point and compatibility facade
- kept runtime orchestration dependent on the existing action, combat, screen, world, and presentation modules
- preserved gameplay behavior, save compatibility, and menu dispatch behavior
- bumped the project version to v0.26.0

## v0.27.0 Game Facade Cleanup

- removed the redundant loaded-state validation wrapper from `game.rs` and re-exported the world validation function directly
- kept `game.rs` as a minimal top-level game entry facade
- preserved gameplay behavior, save compatibility, and screen flow
- bumped the project version to v0.27.0

## v0.27.1 World Bootstrap Decoupling

- removed the `game/world.rs` dependency on `game/actions.rs`
- kept faction lookup inside the world/bootstrap responsibility
- preserved campaign bootstrap, save compatibility, and gameplay behavior
- bumped the project version to v0.27.1

## v0.28.0 Gameplay Action Dispatcher Architecture

- moved `GameAction` dispatch out of `game/runtime.rs` into `game/dispatcher.rs`
- kept the runtime loop focused on lifecycle handling, presentation, menu selection, and turn flow
- preserved existing action, combat, screen, world, save, and quit behavior
- bumped the project version to v0.28.0

## v0.29.0: Gameplay Action Architecture Cleanup I
- Time-display formatting now has its own gameplay time module instead of living inside `game/actions.rs`

## v0.30.0: Gameplay Interaction Architecture Cleanup II
- NPC selection, dialogue, quest offering/turn-in, faction memory, faction reputation, and NPC availability now live in `game/interactions.rs` instead of `game/actions.rs`

## v0.31.0: Gameplay Character Architecture Cleanup III
- Character progression and character-sheet presentation now live in `game/character.rs` instead of `game/actions.rs`.
- `game/actions.rs` retains thin compatibility entry points for progression while the implementation lives in the character module.

## v0.32.0: Gameplay Legacy Architecture Cleanup IV
- Character progression and character-sheet presentation live in `game/character.rs`.
- NPC dialogue, quest interaction, faction memory/reputation, and NPC availability live in `game/interactions.rs`.
- Death, corpse creation, corpse recovery, previous-life item recovery, and legacy item-gain presentation live in `game/legacy.rs`.
- `game/actions.rs` now focuses on ordinary gameplay actions, shared time/condition helpers, menu definitions, inventory/quest views, and journaling.

## v0.33.0: Gameplay Action Architecture Cleanup V
- `game/menu.rs` owns `GameAction`, menu entries, and main-menu construction.
- `game/actions.rs` now focuses on ordinary gameplay actions and shared time/condition helpers, plus inventory, quest, journaling, and character-sheet actions.
- Runtime menu construction and dispatch now depend on the dedicated menu layer rather than `game/actions.rs`.

## v0.34.0: Gameplay Action Architecture Cleanup VI
- `game/actions.rs` now focuses on player-facing gameplay actions rather than temporal state mutation.
- Temporal progression and condition lifecycle helpers now live in `game/state_effects.rs`.
- Travel, meditation, and journaling use the shared state-effects layer.

## v0.35.0: Gameplay Action Architecture Cleanup VII
- `game/actions.rs` now focuses on travel and meditation gameplay actions.
- Inventory display, quest-log display, and journal writing now live in `game/records.rs` as a cohesive player-record concern.
- Character-sheet dispatch now goes directly through `game/character.rs` instead of being wrapped by the action layer.

## v0.36.0: Echoes of the Ashen Road expansion
- Added the Resonant Forge and Hollow Caravan as the expansion's opening locations.
- Expanded the road through Broken Stage, Library of Loops, Silence Spire, Endless Corridor, and the Pit.
- Added expansion encounters, NPCs, faction-linked quests, atmospheres, item art, and location-specific travel events.
- Adapted the old music/metal concepts around an in-world traveling musical tradition, haunted performances, resonance, memory, repetition, and silence rather than modern-world imagery.
- Delivered the expansion through the existing stable-ID mod/content system without changing the gameplay engine or save format.
- Kept the deeper locations in a companion content pack so they can extend and override the opening expansion content through the same loader rules.

## v0.37.0: Developer console foundation
- Added an overlay developer console with scrollable output, command history, and a `/` shortcut from the normal gameplay menu.
- Added command access to world navigation, content/mod inspection, quests, factions, NPCs, inventory, character stats, conditions, time, history, save, and content reload.
- Added Tab autocomplete with arrow-key navigation, Enter selection, Esc cancellation, and stable runtime entity IDs with names/titles shown as hints.
### v0.37.1: Developer console stability patch
- Added an opaque, clean terminal handoff when leaving the developer console so the main gameplay layout is redrawn without stale console buffer contents.
### v0.37.2: Gameplay screen rendering fix
- Reworked the main gameplay menu to render through the shared UI terminal instead of creating a second terminal that overlaid the dashboard.
- Prevented menu choices, results, and journal updates from being visually mixed with the previous gameplay frame.

## v0.38.0: Content loading and world seeding hardening
- Unified base-content and mod discovery around a single resolved `data/` root so the loader cannot silently combine files from different roots.
- Added explicit data-root candidate diagnostics and warnings for missing, unreadable, or malformed external content.
- Strengthened campaign seeding so existing worlds reconcile newly loaded locations, metadata, exits, and persistent campaign entities instead of relying on first-creation state only.
- Exposed the content loading report through the content module for developer diagnostics and future tooling.
- CI reliability: automatic rustfmt commits are retained, while formatting no longer gates test execution; CI also runs when workflow, Cargo, or data files change.

## v0.39.0: UI patching and `logkeys` dev command
- Developer-console output and completion menus support bounded scrolling and keep overflowing content navigable, including wrapped output lines.
- Developer-console implementation is split into focused entrypoint, UI/state, and command modules for easier maintenance.
- Added the `logkeys [true|false]` developer-console command. When enabled, it buffers terminal key events outside the developer console with timestamps, event kinds, key codes, and modifiers; console input itself is excluded.
- Fixed in-game ASCII art flattening by preserving leading whitespace in menu, location, and journal renderers. (#57)
- Fixed transparent/stacked game screens by clearing the full terminal frame before rendering standalone menu and developer-console screens. (#60)
### v0.39.1: patch console and actions layout
- Restored the in-game action menu to the final dashboard panel instead of drawing it as a detached overlay.
- Restored clean developer-console screen lifecycle: the console opens on a cleared screen and returns to the preserved game alternate screen without stacking frames.

## v0.40.0: quest depth and world consequences

- Expanded quests from single-item turn-ins into explicit persisted objective state.
- Existing campaign quests now track visit, defeat, and item-acquisition objectives as a single quest chain.
- Objective progress updates through travel and combat and is visible in the quest log and NPC turn-in flow.
- Quest completion now records a structured quest history event and updates the existing faction and NPC memory systems, allowing later event conditions to react to completed deeds.
- Existing quest/save data is migrated through serde defaults and deterministic quest normalization without changing the save-file version.
- Loaded quest objectives are validated for empty targets, invalid requirements, and impossible progress before runtime use.

## v0.41.0: dedicated quest representation
- Replaced the log-style quest review with a dedicated player-facing fullscreen quest screen.
- Quest index entries clearly distinguish `ACTIVE`, `READY`, and `COMPLETED` states.
- Quest details show the description, objective progress, and reward without mixing into the gameplay journal.
- Returning from the quest screen restores the normal gameplay dashboard without changing quest or gameplay state.
- Kept existing hidden-quest filtering, objective tracking, completion behavior, and save compatibility unchanged.
### v0.41.1: quest navigation bugfix
- Fixed quest details so their Back action returns to the Quests screen instead of exiting directly to gameplay.
- Preserved the existing Quests screen Back action that returns to the normal gameplay dashboard.

## v0.42.0: world navigation presentation
- Replaced the direct Travel menu flow with a dedicated fullscreen World Navigation screen.
- Shows the player's current location, region, description, danger state, and available routes in one focused view.
- Reuses existing location scene art on the navigation screen when it is available.
- Route choices identify dangerous destinations and include a short destination description.
- Keeps the existing travel effects, including time advancement, conditions, threats, quest synchronization, history, and travel events.
- Preserved the existing Back behavior so leaving navigation returns to the normal gameplay dashboard without changing state.

## v0.43.0: inventory and character presentation
- Replaced the plain inventory dump with a dedicated Inventory screen that lets the player select held items and view each item's description and available art.
- Replaced the plain character sheet with a dedicated Character screen containing General, Reputation, and Journal tabs.
- General shows core character stats, health, experience, effective attributes, and active conditions.
- Reputation shows faction standing and remembered faction dealings; Journal shows recorded character notes.
- Reworked meditation to choose the next named time-of-day stopping point instead of entering a numeric duration, while preserving healing, condition updates, history, and automatic saving.
- Preserved the existing screen flow so backing out of the dedicated screens returns to normal gameplay.
### v0.43.1: inventory and meditation presentation refinement
- Inventory item selection now uses a clean list screen with no item art or details shown before selection.
- Selected inventory items open a dedicated detail screen containing the item's description and available art.
- Meditation now keeps the time-of-day selection on a dedicated presentation screen and shows the completed meditation, elapsed portions, recovery, and resulting time on a dedicated result screen.
- Unsafe meditation attempts also use the dedicated Meditation screen instead of the action log.

## v0.44.0: dedicated combat encounter presentation
- Replaced combat's generic action/log presentation with a dedicated combat encounter screen.
- Combat now presents the player and enemy together with current turn, encounter location, enemy power, available actions, and recent combat events.
- Player and enemy health remain displayed through the existing LineGauge presentation while combat is active.
- Combat results for victory, defeat, and fleeing are presented on the encounter screen before returning to normal gameplay or the death flow.
- Preserved existing combat rules, history recording, rewards, quest updates, and threat cleanup behavior.

## v0.45.0: World presentation
- add Dedicated World / The Ever-Unfolding Saga screen
- add dedicated History screen
- preserve World screen parent navigation
- give Talk flow a dedicated screen state
- keep action cursor visible on compact screens

## v0.46.0: presentation foundation
- Added frontend-independent presentation/view models for character, item, location, threat, history, faction, NPC, quest, and combatant data.
- Kept presentation models free of ratatui/crossterm dependencies and gameplay rules so later frontends can consume the same data.
- Added shared terminal UI primitives for compact-layout detection, panel rendering, scrolling text, message panels, health gauges, frame clearing, and common layout behavior.
- Migrated the World/Chronicle screen to build a frontend-independent `WorldView` before terminal rendering.
- Reused shared UI primitives for the World screen's responsive layout, panels, and health gauge while preserving gameplay behavior and menu dispatch.
- Migrated History, Navigation, and Talk flows to build frontend-independent presentation views before terminal-specific display.
- Preserved existing history ordering, navigation choices, NPC availability, dialogue options, quest interactions, and time advancement behavior.
- Migrated Quest Log, Inventory, and Meditation flows to build frontend-independent presentation views before terminal-specific display.
- Preserved existing quest filtering/details, inventory selection/details and item art, meditation safety checks, target selection, recovery, save behavior, and result presentation.
- Migrated the Character Sheet and Combat flows to build frontend-independent presentation views before terminal rendering.
- Preserved character progression, condition/reputation/journal presentation, combat actions, events, result states, rewards, history recording, and threat cleanup behavior.
- Added a frontend-neutral `InputEvent` boundary and routed gameplay-facing keyboard interactions through semantic events instead of direct `crossterm::KeyCode` handling.
- Preserved existing keyboard controls, menu navigation, combat selection, history navigation, and developer-console editing/completion behavior.
- Added focused presentation-boundary tests covering renderer-neutral view composition and representative Character Sheet view-model generation.
- Migrated lifecycle/start/load/character-creation/quit/death flows to frontend-independent screen and choice views.
- Migrated corpse/legacy recovery flow to frontend-independent remains and recovery result views while preserving recovery rules and item behavior.
- Migrated developer-console rendering to consume a renderer-neutral `ConsoleView` while retaining semantic `InputEvent` handling.
- Completed issue #146, removing the remaining lifecycle, legacy/recovery, and developer-console presentation coupling from gameplay-facing flow modules.

## v0.47.0: procedural world foundation
- Added deterministic procedural world generation with explicit region/location/extra-edge configuration and a connected generated world graph.
- Added deterministic authored-content placement on generated worlds while preserving authored location identities, metadata, exits, and gameplay relationships.
- Persisted the generated world's seed and generation configuration as world-level metadata with backward-compatible defaults for older saves.
- Switched new-game initialization to generated worlds and preserved generated structure plus runtime mutations across save/load without re-seeding generated locations.
- Added focused generation, content-placement, save/load, and initialization coverage for the new world lifecycle.

## v0.48.0: procedural content and emerging world
- Added deterministic procedural generation primitives for regional themes, climates, prosperity, danger, population, resources, and tags.
- Added deterministic location characteristics derived from surrounding regional context, including location kinds, population, resources, danger, and tags.
- Added deterministic generated location names, context-driven factions, and NPCs using reusable name/content pools.
- Populated generated settlements and other populated locations with runtime-valid NPCs assigned to context-matched generated factions while preserving authored entities.
- Made generated entity population idempotent across repeated campaign bootstrap calls and covered deterministic generation with focused tests.
### v0.48.1
- Generated deterministic faction relationships from shared resources, danger, prosperity, and environmental differences, recording rivalry, alliance, trade, influence, or dependency in persistent faction memory.
- Integrated relationship generation into generated-world bootstrap and covered deterministic, idempotent, valid, and serialization-preserving relationship state.
- Integrated authored campaign NPCs and factions as deterministic anchors in generated regions without replacing their authored identities, faction assignments, locations, or quest references.
- Allowed authored factions to participate in the same deterministic generated relationship system as generated factions, while retaining authored-authored relationships unchanged.
- Covered authored anchor placement, authored quest reference integrity, cross-authored/generated relationships, and deterministic integration with focused tests.
- Hardened gameplay and persistence with negative Insight XP handling, persistent deterministic event RNG, bounded history, save-size/decompression limits, mod path confinement, and save serialization without cloning the full GameState.
- Generated persistent quests from faction relationships and dangerous locations, using existing quest objectives and NPC/faction runtime references.
- Generated deterministic travel events from the same world relationships and dangerous locations, reusing the existing event trigger/cooldown/history flow.
- Added consequence-driven world evolution: completing generated opportunities can pacify dangerous locations, record persistent faction memory, and create deterministic follow-up opportunities without regenerating the world graph.
- Covered generated opportunities, valid references, evolution, serialization, and time-driven consequence propagation with focused tests.

## v0.49.0: Bevy full migration
- Established Bevy as the default and shipped graphical runtime while keeping gameplay rules and presentation models independent of the frontend.
- Added a reusable Bevy presentation layer with shared screen/panel/label primitives, choice buttons, health-style gauges, centralized visual theme constants, semantic navigation state, and a semantic input queue.
- Added Bevy keyboard translation for the frontend-neutral input events and Bevy UI interaction translation without exposing Bevy types to gameplay systems.
- Migrated lifecycle flows for starting, loading, character creation, death, inheritance, and quit confirmation to Bevy.
- Migrated the primary gameplay/world dashboard and world navigation/travel flow to Bevy while continuing to reuse authoritative gameplay rules.
- Migrated NPC dialogue and remains recovery into dedicated Bevy interaction screens while keeping dialogue, quest interaction, corpse recovery, and related state changes in the authoritative game modules.
- Migrated character, reputation, journal, inventory, quest, meditation, and history screens to Bevy, including nested detail/result flows and journal entry editing.
- Migrated combat presentation and interaction to Bevy while keeping combat resolution, state changes, outcomes, and reward handling in the authoritative game combat system.
- Migrated the developer console frontend to Bevy while retaining renderer-neutral command state, completion, history, scrolling, save, output, and close behavior.
- Removed Ratatui/crossterm dependencies and terminal-only screen, rendering, input, and game-loop modules from the shipped application.
- Persisted runtime-generated event definitions separately from authored campaign events so procedural event progression survives save/load.
- Removed per-trigger cloning of the complete campaign event vector while preserving deterministic event selection and RNG sequencing.
- Routed event-driven condition application through the shared condition refresh semantics to prevent duplicate same-named conditions.
- Recorded processed procedural world-evolution transitions in structured event history so resolved evolution quests are not reconciled repeatedly on later time advances.
### v0.49.1: Bevy player feedback restoration
- Removed migration-only `InputEvent::Other` and unused shared `ScrollViewport` compatibility types from the frontend boundary.
- Removed the obsolete terminal UI compatibility façade and the terminal-only `logkeys` developer command; gameplay modules no longer depend on the legacy presentation layer.
- Restored player-facing progression feedback in Bevy, including interactive level-up attribute selection and quest reward details for gained/consumed items and reputation changes.
- Made Bevy the required runtime dependency instead of retaining an obsolete optional feature gate now that the terminal frontend has been removed.

## v0.50.x: Android Packaging/Porting
### v0.50.3: Responsive Bevy presentation for Android landscape
- Made the shared Bevy screen root use viewport-relative spacing so the same layout scales across phone, tablet, and resizable desktop landscape resolutions.
- Made shared panels flexible so dashboard, navigation, lifecycle, record, interaction, and result content can shrink into the available viewport while retaining internal scrolling for longer content.
- Made shared labels and button text wrap within their available width instead of allowing long content to force the layout wider than the screen.
- Converted shared title/body typography and the progression feedback overlay to viewport-relative font and spacing units while preserving the existing dark, text-first visual identity.
- Kept interactive controls at a 48px logical minimum height to provide a stable touch target while responsive spacing and typography adapt around them.
- Reused the same shared presentation implementation for desktop and Android; Android remains landscape-only through the existing activity configuration.
### v0.50.4: Android touch, soft-keyboard, and system Back input
- Added reusable contextual touch behavior to the shared Bevy presentation layer while preserving the frontend-neutral semantic input queues and desktop keyboard mappings.
- Ordinary Bevy choice buttons remain direct-touch actions, while character-creation fields and the developer-console input gain contextual touch focus controls.
- Added a compact contextual Tab button beside the developer-console input without reserving a permanent control row.
- Added touch-drag scrolling for shared scrollable panels so long gameplay, records, dialogue, console, and lifecycle content remains usable on touch-only devices.
- Added shared Android soft-keyboard IME integration; committed IME text is routed through the existing keyboard text-input path, and text focus controls keyboard visibility without Android-specific gameplay logic.
- Added Android Back key handling through the shared semantic Cancel input, including suppression of the navigation action when the soft keyboard is being dismissed.
- Completed semantic delete handling in the developer console while retaining existing physical keyboard behavior.
### v0.50.5: Cross-platform game-root selection and Android screen audit
- Added native desktop game-root use/recreate/alternate-folder selection.
- Hardened protected base-content replacement and corrected lifecycle Load Back navigation.
- Corrected Combat semantic Cancel for Android system Back and audited all existing Bevy screens against the shared responsive/touch architecture.
### v0.50.6: Android APK CI packaging and signing
- Added CI packaging for installable signed Android APKs using the existing Gradle/GameActivity project.
- Preserved the existing Android x86_64, AArch64, and ARMv7 targets with per-ABI native library packaging.
- Added workflow-dispatch support for building a selected Android target without a release tag.
- Added secure release signing through GitHub Actions secrets, with short-lived CI-only signing for manual test builds when persistent secrets are unavailable.
- Added APK signature verification and Android build-log artifacts.
- Release-tag Android assets are now APKs; the legacy standalone Android executable tarballs are removed from the published release while desktop release artifacts remain unchanged.
### v0.50.7: Android launch crash fix
- Added Bevy's Android entry-point macro so GameActivity can invoke the native application entry point.
- Added an Android CI guard that fails the build when `android_main` is not exported by the native library.
### v0.50.8: Android SAF storage access
- Replaced Android All Files Access with user-selected Storage Access Framework directory access.
- Kept Rust filesystem I/O on the app-scoped Android game root while mirroring editable mods and saves to the selected shared folder.
- Persisted the selected folder permission across launches and synchronized shared data into the local game root on startup.
- Migrated existing local mods/saves into an empty newly selected shared folder without overwriting an existing shared game dataset.
- Kept the bundled protected base content out of the user-editable shared storage mirror.
### v0.50.9: Android armv7 CI symbol-check fix
- Corrected Android CI native-library validation to inspect defined `android_main` symbols instead of requiring dynamic export visibility, allowing the armv7 build to proceed to APK packaging.
### v0.50.10: Android entry-point CI guard fix
- Replaced the stripped-library `android_main` symbol check with source/configuration validation that is compatible with the release profile.
- Kept the native Android build and added a non-empty native-library output check before APK packaging.
### v0.50.11: Android GameActivity entry-point fix
- Moved Bevy's Android entry-point function into the library target that produces the packaged native `.so`.
- Restored artifact-level CI validation of the exported `android_main` symbol using the Android NDK's `llvm-nm`.
- Kept a non-empty native-library output check before APK packaging.
### v0.50.12: Android GameActivity entry-point export fix
- Replaced Bevy's generated private Android entry-point wrapper with a public `android_main` symbol in the library target that produces the packaged native `.so`.
- Kept artifact-level CI validation of the exported `android_main` symbol using the Android NDK's `llvm-nm`.
- Kept a non-empty native-library output check before APK packaging.
### v0.50.14: Android storage and touch input fixes
- Corrected Android bundled-asset paths so the packaged `data/` content is copied into the app-local game root before native gameplay starts.
- Added Android-side diagnostics for IME events and the `BrowserBack` key event so device input-delivery behavior can be verified without changing the existing text or navigation consumers.
- Added tap-versus-drag gating for graphical choice buttons so touching and dragging a scrollable panel no longer immediately confirms a choice.
- Updated the Android Gradle fallback version metadata to match the project version.
- Fixed the touch-choice interaction query so touch-button state can be cleared without a Rust mutability error under the current stable toolchain.
- Android device verification of storage synchronization, IME delivery, system Back delivery, and touch gesture behavior remains outstanding.
### v0.50.15: Android shared-storage synchronization fix
- Extended Android local-to-shared SAF synchronization from `data/mods/` to the complete local `data/` directory, including bundled root-level files such as `base_content.json` and future root-level data files.
- Kept shared-to-local synchronization limited to editable `data/mods/` and `saves/`, preserving the bundled base content as locally protected content.
- Added an Android `onPause()` storage flush while retaining the `onStop()` flush, so newly written saves and input diagnostics are mirrored to the selected shared folder earlier in the activity lifecycle.
- Hardened recursive local-directory mirroring so a single file-copy failure is logged without aborting synchronization of the remaining files in the same directory.
- Bumped the project and Android fallback version metadata to 0.50.15.
- Physical Android verification of complete data mirroring, save-file visibility, and input-log visibility remains outstanding.
### v0.50.16: Android touch and text-input flow fix
- Normalized custom Android touch hit-testing to Bevy UI's physical coordinate space so taps target the control actually touched on scaled displays.
- Applied the same coordinate conversion to custom touch-scroll hit-testing.
- Made character-creation fields explicit graphical buttons instead of dynamically converting matching text labels into controls after rendering.
- Kept text-entry focus tied to explicit lifecycle field controls and retained the shared IME/keyboard input path.
- Added Android keyboard-text diagnostics so device verification can distinguish IME delivery from keyboard-event routing.
- Added focused coverage for the shared touch-coordinate conversion.
- Physical Android verification of the corrected touch targeting, character-creation field selection, IME text entry, keyboard dismissal, and post-IME interaction remains outstanding.
### v0.50.17: Native Bevy editable text input
- Migrated character-creation fields to Bevy 0.19.1's `EditableText` and `InputFocus` pipeline instead of maintaining a custom lifecycle text-input implementation.
- Enabled Bevy's UI widgets and tab-navigation support so the three character-creation fields use native text editing, focus traversal, cursor handling, deletion, and IME routing.
- Removed the lifecycle-specific IME commit-to-`KeyboardInput` bridge and duplicate semantic Backspace editing path; the custom IME bridge remains only for the developer console.
- Kept the shared Android touch coordinate conversion and explicit field touch targets, with touch selection now assigning real Bevy input focus.
- Added Android Back/IME dismissal recovery that clears native editable focus before semantic navigation resumes.
- Physical Android verification of distinct start-menu targets, character-creation field selection, native IME text entry, keyboard dismissal/recovery, and touch scrolling remains outstanding.
### v0.50.18: Bevy editable-text startup fix
- Registered the `Pointer<Release>` message storage required by Bevy 0.19.1's `EditableTextInputPlugin` without enabling Bevy's picking plugin stack.
- Preserved the project's custom touch interaction architecture while preventing the native editable-text pipeline from failing ECS system validation at startup.
- Bumped the project and Android fallback version metadata to 0.50.18.
### v0.50.19: Save path correction
- Save files now use the dedicated `saves/` directory under the selected game root instead of being written directly into the root.
- Load discovery scans `saves/` for current character-specific compressed saves while retaining root-level discovery for the legacy uncompressed save filename.
- Bumped the project and Android fallback version metadata to 0.50.19.

## v0.51.x: UI overhaul
### v0.51.13: Android launch presentation and desktop display configuration
- Added the canonical purple pixel-art app icon as the Android launcher icon, Android launch splash artwork, and desktop window icon.
- Added a tabbed Options Display section with desktop Fullscreen, Windowed 1920 × 1080, and Windowed 1280 × 720 presets.
- Applied desktop display changes at runtime while keeping windowed presets resizable.
- Persisted the selected desktop display mode in the desktop configuration directory.
- Kept the Display tab present but non-configurable on Android.
### v0.51.14: Android quit/relaunch lifecycle fix
- Kept the explicit Android task/process termination required to avoid reusing Bevy's process-global Android activity handle after Quit.
- Deferred the process kill until after the Java Activity's destroy callback has fully unwound, avoiding an immediate process death inside `GameActivity.onDestroy()`.
- Hardened the Android Quit path to target the post-Quit relaunch splash regression by allowing Activity teardown to unwind before process termination.
### v0.51.15: Atmospheric quit-confirmation variants
- Tightened the semantic relationship between each quit question, its leave/stay responses, and its ASCII art motif.
- Preserved whole-bundle random selection so each atmospheric presentation remains internally matched.
- Expanded the quit-confirmation set from 9 to 16 variants, including substantially darker themes of fading names, unseen voices, doubled heartbeats, mirrors that refuse to reflect departures, and an unseen presence.
- Bumped the project and Android fallback version metadata to 0.51.15.
### v0.51.16: Shared graphical presentation regression fix
- Restored intrinsic height for standalone graphical text labels and muted labels.
- Consolidated standard action buttons into a bottom-left, stacked group with a 520px maximum width.
- Kept the shared touch target contract while preventing action buttons from flex-growing to fill the available panel height.
- Bumped the project and Android fallback version metadata to 0.51.16.
### v0.51.17: Final Android input compatibility fixes
- Routed Android system Back button and gesture handling through the AndroidX OnBackPressedDispatcher into the shared semantic Cancel path.
- Added desktop mouse Back-button mapping to the same semantic navigation path.
- Synchronized Android GameActivity text-input state with the character-creation EditableText fields and developer console because the existing Bevy IME event path was not delivering committed character text on the tested device.
- Preserved keyboard dismissal behavior by suppressing navigation while an Android text field remains focused.
- Bumped the project and Android fallback version metadata to 0.51.17.
### v0.51.19: Android native text-input bridge
- Replaced the Android GameActivity text-state synchronization path with a Java-hosted native EditText that stays outside the game's rendered UI.
- Forwarded Android text changes, editor submit actions, and keyboard dismissal to Rust through small JNI callbacks, with target routing for character creation and the developer console.
- Disabled Bevy's Android Window::ime_enabled path so the Android text-entry fix no longer touches GameActivity's shared GameTextInput state.
- Kept desktop text input and the existing Android system Back/navigation behavior intact.
- Bumped the project and Android fallback version metadata to 0.51.19.
### v0.51.20: Android build regression fix
- Corrected the Android JNI callback's error handling for `jni 0.22.4`, which returns an `EnvOutcome` from `EnvUnowned::with_env` rather than a standard `Result`.
- Bumped the project and Android fallback version metadata to 0.51.20.
### v0.51.21: Android IME visibility fix
- Prevented Bevy/winit from changing Android IME visibility while the Java text-input bridge is active, avoiding an immediate native `hideSoftInput` request.
- Hardened keyboard presentation with AndroidX `WindowInsetsControllerCompat.show(Type.ime())` and retained `InputMethodManager.showSoftInput()` as a compatibility fallback.
- Bumped the project and Android fallback version metadata to 0.51.21.
### v0.51.22: Android text-edit cursor safety
- Replaced full-snapshot Android lifecycle text synchronization with direct text-buffer replacement and a queued cursor-to-end operation, avoiding insertion at a stale cursor offset.
- Added regression coverage for repeated replacement, stale pending edits, and multibyte Unicode text.
- Disabled the Android IME's fullscreen extract editor for the hidden native EditText to keep native input presentation aligned with the in-game field.
- Bumped the project and Android fallback version metadata to 0.51.22.
### v0.51.23: Developer console, Android input, and Ashen Gate pixel-art banner — complete
- Marked only the active console input row for touch input and autocomplete, preventing executed command-history lines from becoming duplicate input fields.
- Added an upper-corner Close button that exits the console through the same session reset path as the existing close behavior.
- Stabilized Android text entry across character creation and the developer console, including native keyboard visibility, cursor-safe text updates, IME presentation, JNI compatibility, and shared Back navigation.
- Completed the release verification for desktop and Android console input and close behavior.
- Replaced Ashen Gate's ASCII scene art with a full-width environmental panorama while leaving the location card and surrounding gameplay UI unchanged (#350).
- Replaced the JPEG intermediate with a 600×125 PNG and nearest-neighbor image sampling for crisp pixel-art edges; retained the ASCII fallback for other locations and banner-load failures (#351).
- Bumped the project and Android fallback version metadata to 0.51.23.

## v0.52.x: Storage, interface, and spatial foundations

### v0.52.0: Internal storage and mod-management redesign (#385, #386)
- Replaced selectable desktop game roots, Android SAF folder selection and mirroring, and multi-root content discovery with one application-managed internal data root per platform. Migration from the removed roots and shared folders was intentionally excluded.
- Added explicit save import/export and ZIP mod installation through Options, with archive validation, staged installation, collision protection, and controls to enable, disable, remove, and reprioritize installed mods.
- Made installed mods the only source of custom assets. Content-bearing and asset-only mods are supported; bundled assets remain protected, and mod asset replacements resolve deterministically by priority.
- Validated cross-mod event references after enabled content is merged. Save metadata records active mod IDs, names, and versions, and loading warns about missing, disabled, or mismatched dependencies.
- Deferred mod changes until the next process initialization and refreshed Android's packaged asset cache while keeping installed mod packages user-managed.

### v0.52.1: Shared UI scaling and fullscreen presentation (#388)
- Made desktop start fullscreen and removed the old windowed-resolution presets; Android retains immersive landscape presentation.
- Added persistent, independently configurable UI scaling and font sizing shared by desktop and Android through Interface Options.
- Reconciled scale changes with preference and viewport updates, including newly created UI nodes.

### v0.52.2: Reusable Options rows (#389)
- Reworked Saves, Mods, and Interface around one reusable landscape row pattern, keeping labels on the left and controls with current values on the right.
- Placed save export and mod enable/disable, delete, and priority controls beside their corresponding entries; Interface rows use decrement, value, and increment controls.
- Gave each tab's entries an independent scroll area and kept import, install, reset, and Back actions in a persistent footer.
- Kept explanatory copy focused on the save-import no-overwrite guarantee and current status notices.

### v0.52.3: Save metadata, New Game flow, and release distribution (#390–#393, #395–#396, #399–#400)
- Added character, world/progression, save-format, and saved mod-dependency metadata to save rows in Options and Load, with filename fallback when metadata is unavailable.
- Split New Game setup into World, Character, and Start tabs. The Start tab reviews the chosen names before Begin Life; data-driven character backgrounds remain deferred to #368.
- Replaced lifecycle button glyphs with bundled icons.
- Rebranded the game as **The Ever-Unfolding Saga**, updated the Android app name to TEUS, and added a version label to the start screen.
- Added optional GitHub Release mirroring to itch.io through Butler. Android release publishing waits for the full set of APK assets before uploading each platform/architecture build.

### v0.52.4: Character record hub (#401)
- Consolidated character records into one tabbed hub with General, Reputation, Inventory, Quests, and Journal views.
- Combined the world chronicle and personal notes in Journal while preserving note writing without duplicating the generated history entry.

### v0.52.5: Onyx interface and dark-fantasy iconography (#402)
- Unified the shared interface around an onyx `#070707` background, neutral charcoal surfaces, and a coherent pixel-art icon set that mods can override individually.
- Moved the sun and moon presentation to larger world-art sprites cropped from `data/assets/world/celestial/sun-moon.png`; they are not UI icons.

### v0.52.6: UI transitions, text focus, and memory presentation
- Kept the New Game, Character, and Options screen shells and tab strips in place while refreshing only the selected tab's content.
- Reconciled UI scaling on preference and viewport changes, with a separate incremental pass for newly spawned UI nodes.
- Decoupled New Game field selection from editable focus so the native keyboard opens only after explicit field activation; Android keyboard dismissal no longer triggers automatic refocus.
- Filtered authored-anchor, generated-relationship, and generated-faction metadata from player-facing faction and NPC memories without removing the persistent procedural records.
- Bumped the project version to 0.52.6.

### v0.52.9: Spatial interaction and modular portrait foundations (PR #404; device validation pending)
- Extended Explore to enter local spatial play at both safe and dangerous locations and removed detached Talk and Search Remains dashboard actions.
- Added deterministic NPC and searchable-remains placements persisted with local maps, with backward-compatible defaults for older saves.
- Added stable action/target pairs for nearby NPCs, remains, points of interest, and passages. The current nearest-target control and in-world interaction overlay reuse existing dialogue, quest, time, loot, and recovery rules while keeping the player in the spatial session.
- Added a mod-extensible, deterministic weighted portrait-part catalogue and bundled 15 transparent 64×96 pixel-art layers across body, face, hair, headgear, and accessory slots. Existing ASCII portraits remain the visible fallback until the dialogue scene composes the layers.
- Added focused coverage for actor placement, interaction discovery, portrait determinism and compatibility, mod catalogue overrides, and the bundled assets. Formatting, Clippy, all 183 unit tests, REUSE lint, and third-party license checks passed on implementation commit `46cdd86`.
- Manual desktop and Android landscape playtesting remains required before #404 is complete. The radial interaction wheel, reusable scene transitions, and close-up dialogue scene are follow-up work in #405.

### v0.52.21: World Lab passage and actor authoring completion (#413, PR #419)
- Completed the initial Developer Tools milestone with a standalone Developer Tools hub and a portrait-focused Character Workshop using the same modular portrait compositor and mod-aware asset loader as dialogue.
- Added World Lab passage placement, repositioning, and removal against real destinations in a deterministic disposable world graph; edits keep reciprocal location exits synchronized.
- Added actor placement, repositioning, and removal using actual generated NPC and searchable-remains identities, with sandbox-owned identity validation in the production spatial renderer.
- Extended fixture persistence and restoration for passage and actor references, reconstructing the deterministic world graph and actor catalogue from the fixture seed; validation rejects duplicate destinations and invalid or duplicate actor identities.
- Kept World Lab isolated from campaign state and saves, and preserved the existing production movement, collision, rendering, and interaction paths for playtesting.
- Added focused model, editor, renderer-integration, and fixture regression tests. Formatting, Clippy, all 230 unit tests, REUSE lint, and third-party license checks passed on PR #419's validated commit.
- Full-body character authoring remains future work until the required full-body visual assets and renderer support exist; it is not a prerequisite for this milestone.

### v0.52.22: Player portraits and contextual target prompts (#405)
- Added an Appearance tab to new-character creation with a live modular-portrait preview and controls for selecting a part in each available layer slot.
- Persisted stable portrait-part IDs on the player character. Missing appearance data in older saves resolves to a deterministic catalogue default; inherited lives retain the player's configured appearance.
- Kept NPC portraits on the existing deterministic weighted generator while resolving the player portrait exclusively from the player's selected part IDs.
- Changed the spatial interaction control to identify the nearest NPC, remains, point of interest, or destination by name. When more than one distinct target is available, the prompt names the nearest target and indicates the additional target count; the radial wheel remains the action selector.
- Added regression coverage for appearance resolution, legacy save defaults, inheritance, player/NPC portrait separation, and all spatial prompt target categories.
- The project owner manually play-tested the existing #405 interaction/dialogue work; this change completes the remaining requested refinements. Formatting, Clippy, unit tests, REUSE lint, and third-party license checks are tracked on the associated pull request.

### v0.52.23: Spatial movement and camera consistency
- Subdivided spatial movement into short collision-checked steps with a bounded frame-time contribution, improving movement continuity during brief low-frame-rate intervals while preserving wall/prop collision and axis-separated sliding.
- Changed camera following to exponential smoothing so the response is consistent across frame rates, and clamp the final smoothed camera position to map bounds.
- Added regression tests for 30/60 FPS camera consistency, post-smoothing camera bounds, movement recovery after a short frame drop, and collision during a slow frame.
- Recorded the staged spatial-first gameplay direction in #421. The existing Gameplay dashboard remains until its gameplay and system functions have validated replacements in the spatial experience.
- Bumped the project version to 0.52.23.


### v0.52.24: Directional player animation foundation
- Added a renderer-independent animation player with configurable idle/walk clip definitions, frame durations, looping behavior, retained facing, and frame-delta-based playback.
- Defined the player sprite sheet as a row-major 5×4 atlas of 16×24 frames: north, east, south, and west rows, each with one idle frame followed by four looping walk frames.
- Integrated four-way screen-facing selection and movement-speed-scaled idle/walk playback into spatial exploration without changing movement, collision, or world-state rules. Blocked movement may change facing but remains idle.
- Added mod-aware sheet loading and exact atlas-dimension validation; missing, empty, or invalid art falls back to the existing static player sprite.
- Added the intentionally empty `data/assets/spatial/player_oathbound_animated.png` placeholder and documented its stable path and frame layout. Visible directional artwork and desktop/Android visual verification remain outstanding.
- Added focused unit coverage for direction selection, frame order, configurable clips, looping, speed scaling, stopping/facing retention, frame-rate independence, and invalid deltas.
- Bumped the project version to 0.52.24.

### v0.52.25: Spatial screen context preservation
- Added shared navigation helpers that capture the current screen when opening a dedicated screen and restore that caller when the dedicated screen closes.
- Applied context-aware routing to Character, Meditation, Options, combat, console, and standalone interaction flows while retaining Gameplay as the compatibility fallback.
- Added spatial-scene re-entry detection so its rendered entities and HUD are rebuilt after another dedicated screen replaces the screen root; the existing local map and continuous player position remain authoritative.
- Kept the current HUD and action layout unchanged. Spatial access to records, meditation, combat, and system navigation remains a separate design/implementation step under #421.
- Bumped the project version to 0.52.25.
