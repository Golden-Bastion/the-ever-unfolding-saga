# Changelog

This file records work completed in the current development line. It is not a list of planned work. Earlier lines are preserved in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md), while unfinished work remains in [ROADMAP.md](ROADMAP.md).

## v0.53.x — World topology and persistent spatial-world foundations

The entries below cover the v0.53.x development line beginning with v0.53.0. This line extends the local spatial prototype with independently persistent encounters and lays the groundwork for coherent world topology and a more reactive spatial world.

### v0.53.0: Independent persistent spatial encounter groups

- Added the optional `spawn_group` field to campaign encounter profiles. Profiles in the same group are weighted alternatives; different groups are selected independently and can produce multiple persistent encounters on one local map.
- Kept profiles without a group in one default group, preserving existing campaign behavior and old mod definitions. Legacy local-map records without group metadata also reserve that default group.
- Snapshotted the selected group with each encounter record. Existing active or defeated encounters retain their identity, health, position, and quest attribution; additional groups can materialize later when their conditions become eligible.
- Removed the single-active-encounter placement restriction while retaining unique, reachable, collision-safe map positions. Nearby encounter interactions and combat continue to address each encounter by its stable record ID.
- Added regression coverage for independent group selection, weighted alternatives, delayed group eligibility, legacy-save and mod compatibility, multiple nearby interaction targets, and exact encounter selection in combat.
- Roaming, respawning, awareness/pursuit, and off-screen simulation remain separate follow-up work.

## v0.52.x — Storage, UI, and spatial-gameplay foundations

The entries below cover the v0.52.x development line through v0.52.62. This line established the storage and mod-management foundation, rebuilt core UI surfaces, introduced spatial exploration and authoring tools, migrated in-session play to the spatial world, and began the retained-presentation architecture.


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

### v0.52.26: Spatial pause menu access
- Reused the unified Gameplay pause/navigation menu as an overlay above spatial exploration, preserving the active map and continuous player position instead of returning to the transitional dashboard.
- Routed desktop Escape and Android system Back to open the menu; Back while the menu is open resumes the same spatial session. Character and Options child screens return to the paused spatial menu, and Quit cancellation returns to that same context.
- Kept the spatial Leave control as a separate explicit transition to the compatibility dashboard.
- Paused spatial movement, animation advancement, camera following, and Android virtual-joystick input while the menu is open.
- Added regression tests for spatial pause entry/exit, parent-screen retention, and input-screen gating.
- Bumped the project and Android fallback versions to 0.52.26. Desktop and Android landscape playtesting remains outstanding.

### v0.52.27: Spatial destination-list travel
- Added a Travel entry to the existing spatial pause menu without adding a permanent HUD control.
- Reused the current destination selector and authoritative location-graph travel action; physical passages remain available through spatial interactions.
- Shared the map-preservation and destination-map restoration path between physical passage travel and destination-list travel.
- Destination selection remains an overlay over spatial exploration. Back returns to the pause menu; successful travel enters the destination's spatial map, while invalid routes leave the source context intact.
- Added regression coverage for source-map persistence, destination-map restoration, and stale-route rejection.
- Bumped the project and Android fallback versions to 0.52.27. Desktop and Android landscape playtesting remains outstanding.


### v0.52.28: Safe meditation entry from spatial exploration
- Added Meditate to the existing spatial pause menu only when the authoritative game state permits safe meditation.
- Opened the existing dedicated meditation screen from the spatial pause menu and retained the spatial scene as its return context; no artificial movement or permanent HUD control is required.
- Kept meditation target selection, safety validation, recovery, time advancement, and persistence in the existing action and records systems.
- Added regression coverage for safe-only visibility, non-spatial exclusion, menu ordering, and conditional menu sizing.
- Bumped the project and Android fallback versions to 0.52.28. Desktop and Android landscape playtesting remains outstanding.


### v0.52.29: Contextual investigation in spatial exploration
- Added an Investigate control beside Interact while an active threat belongs to the current location.
- Reused the authoritative `bevy_combat::begin` entry point and existing Combat screen; no spatial threat actor or parallel combat rules were introduced.
- Added regression coverage for inactive, current-location, and other-location threat availability.
- Bumped Rust and Android version metadata to 0.52.29. Desktop and Android landscape playtesting remains outstanding.


### v0.52.30: Spatial time-of-day presentation
- Reused the authoritative 12-part world clock and existing mod-aware sun/moon atlas in a compact spatial sky indicator beside the location heading.
- Added a small 24-hour readout using the simulation's two-hour time steps; the display does not imply minute-level precision.
- Applied restrained warm dawn/dusk and cool night colour washes to the spatial environment beneath the HUD, preserving the readability of controls and status surfaces.
- Rebuilt the spatial presentation when the authoritative day/time changes so the sky indicator and environmental treatment cannot drift apart.
- Added regression tests for the 24-hour time mapping and day-phase lighting bands.
- Bumped Rust and Android version metadata to 0.52.30. Desktop and Android landscape visual validation remains outstanding.


### v0.52.31: Spatial lifecycle hand-off and character-context verification
- Routed completed character creation, inheritance, and successful save loading directly into spatial exploration instead of the transitional Gameplay dashboard.
- Kept save-warning confirmation in the lifecycle screen until the player explicitly accepts the warning and loading completes.
- Preserved the explicit spatial Leave route to the compatibility dashboard; the dashboard itself is not retired in this increment.
- Added lifecycle bridge regression tests for successful completion, pending save warnings, and invalid routing conditions.
- Added Character hub regression coverage for all five tabs and nested inventory, quest, chronicle, and journal Back paths preserving the original spatial parent.
- Bumped Rust and Android version metadata to 0.52.31. Desktop and Android landscape validation remains deferred until the spatial migration work is complete.

### v0.52.32: Transient spatial location identity
- Removed the redundant generic exploration label from the normal spatial header, leaving the compact time indicator on its own; the World Lab retains its seed heading.
- Added a four-second arrival notice when entering spatial exploration or traveling to another location; the normal spatial status feedback becomes visible again when the notice expires.
- Preserved travel outcome feedback separately from the temporary location announcement.
- Kept region and current-location hierarchy out of the permanent HUD and recorded a focused World Map showing both names as future work.
- Added regression coverage for initial entry, notice expiration, sandbox heading identity, and both physical-passage and destination-list travel announcements.
- Bumped Rust and Android version metadata to 0.52.32. Integrated desktop and Android landscape validation remains deferred until the spatial migration work is complete.


### v0.52.33: Spatial World Map and fast travel
- Added a focused World Map to the spatial pause menu and compatibility Gameplay hub, showing current region/location, previously visited locations, immediate frontier exits, and authoritative graph connections.
- Made visited map nodes directly selectable as fast-travel targets when connected through reciprocal routes made entirely of visited locations. Each hop uses the normal travel action, preserving time, turns, history, quests, and arrival effects; death, danger, or consequential arrival events interrupt the journey at that stop.
- Kept unexplored frontier nodes visible but non-selectable for fast travel; they must be reached through ordinary spatial gameplay and physical passages.
- Removed the separate destination-only Travel screen, its renderer-neutral navigation view/module, and the old pause-menu Travel entry. Normal physical-passage travel remains in spatial gameplay.
- Kept graph layout presentation-only and deterministic; no geographical coordinates or map positions were added to world data or saves.
- Documented the interim map's view/layout/fast-travel boundaries and the future continuous geography-backed map requirements under issue #365.
- Added map discovery/fast-travel tests, multi-hop and interruption coverage, and deterministic graph-layout regression coverage.
- Bumped Rust and Android version metadata to 0.52.33. Desktop and Android landscape visual validation remains deferred until the spatial migration work is complete.

### v0.52.34: Spatial pause control and developer-console access
- Moved the existing menu button from the compatibility Gameplay HUD into the spatial exploration header, beside the separate Leave control.
- Preserved tap-to-pause behavior and the 650 ms touchscreen long-press gesture for opening the developer console; dragging cancels the hold gesture.
- Routed the long-press console entry through the existing dedicated-screen navigation so closing the console restores the same spatial session and map.
- Kept the desktop slash shortcut available from spatial exploration and retained desktop Escape / Android Back pause-menu navigation.
- Added regression coverage for menu-button screen eligibility, spatial console return context, and developer-console shortcut availability.
- Bumped Rust and Android fallback version metadata to 0.52.34.


### v0.52.35: Directional player idle/walk animation and asset contract
- Replaced the one-frame idle/four-frame walk atlas contract with paired idle and walk rows for north, east, south, and west.
- Defined 32×48 logical frames in a 6-column × 8-row atlas (192×384 total), with four looping idle frames and six looping walk frames per direction.
- Added a mod-aware JSON manifest for frame dimensions, clip coordinates, per-clip durations, and looping behavior; the renderer validates the manifest and derives its atlas layout from it.
- Made idle animation advance while the player is stationary and prevented repeated stop/pause calls from resetting the idle cycle.
- Scaled animated sprites to their manifest-defined frame size while preserving the existing static fallback dimensions, player ground anchor, and collision footprint.
- Added unit coverage for manifest parsing/validation, row mapping, idle looping, walk looping, movement-speed scaling, and invalid frame deltas.
- Kept the PNG placeholder empty pending external artwork generation; the static player sprite remains the runtime fallback.
- Bumped Rust and Android fallback version metadata to 0.52.35.

### v0.52.36: Fix spatial exploration startup query conflict
- Made the mutable floor-tile and wall-side `Sprite` queries explicitly exclude the `SpatialPlayer`, ensuring all three mutable sprite query sets are disjoint.
- Added a Bevy app smoke test that initializes and runs `update_spatial_world`, guarding against regressions in query access validation.
- Bumped Rust and Android fallback version metadata to 0.52.36.

### v0.52.37–v0.52.45: Spatial gameplay, authoring, and integration

- Moved spatial interaction discovery into renderer-neutral gameplay logic and introduced stable action/target identities for NPCs, remains, discoveries, and physical passages.
- Added a player-anchored radial interaction wheel with responsive layout, keyboard/touch navigation, and selected-entry feedback; introduced close-up dialogue scenes with composited player and NPC portrait layers.
- Added authored, proximity-revealed spatial discoveries and persistent transitions between connected local maps through the authoritative travel action.
- Built Character Lab and World Lab foundations, including deterministic local-map generation, isolated playtesting, map editing, point-of-interest and passage/actor authoring, and reusable versioned fixtures.
- Improved spatial movement, collision resilience, camera behavior, and player sprite animation; supplied the directional 32×48 idle/walk atlas and data-driven animation manifest while retaining the static fallback.
- Migrated lifecycle hand-offs, pause/menu access, meditation, investigation, time-of-day presentation, character records, and return navigation into the spatial-first flow; retired the legacy Gameplay dashboard and its Leave route.
- Added navigation controls to the interim World Map while preserving its topological graph, existing fast-travel rules, and presentation-only viewport state.
- Fixed character-appearance control wiring and Android Developer Tools touch/render invalidation behavior.

### v0.52.46–v0.52.49: Retained UI presentation architecture

- Established shared animated button feedback, screen-entry transitions, stable content hosts, and retained scroll handling as reusable presentation primitives.
- Kept Character Lab, developer console, combat, interaction/dialogue, Character Records, Options, feedback overlays, and spatial time-of-day visuals in stable presentation shells, updating only the relevant content regions.
- Added regression coverage for retained host identity, scroll-position preservation, independent content invalidation, and spatial sky presentation.
- Corrected the sun/moon travel direction so the sun rises on the left and sets on the right of the sky presentation.
- Left the interim World Map unchanged during the retained-presentation work.

### Documentation structure

- Established a dedicated design-specification library and separated the active roadmap, current changelog, and prior changelog history so future plans, completed work, and technical documentation have distinct homes.

- Formalized the approved visual direction: grounded gothic fantasy balanced with environmental beauty and regional variety, flexible three-quarter perspective, realistic character proportions, restrained material-led palettes, and selective influence from The Witcher 3, Kingdom Come: Deliverance, and Dark Souls Remastered. Expanded the interface presentation standard to use the same references without copying their designs.
- Recorded the retained-shell and keyed-row UI migration as complete following owner playtesting, and reconciled the roadmap and UI architecture documentation accordingly.

### v0.52.50: South-facing player sprite palette correction

- Recolored the 1,427 olive pixels in the south-facing idle and walk rows (atlas rows 4–5, zero-indexed) to a restrained warm-charcoal palette while preserving per-pixel luminance to rounding, alpha, frame positions, all unselected pixels, and the animation manifest.
- Bumped Rust and Android fallback version metadata to 0.52.50.

### v0.52.53: Fix Android World Lab fixture export compilation

- Routed World Lab fixture export through the lifecycle-owned Android storage bridge and exposed the export helper to the developer-tools module, fixing the Android native-library compile error.
- Bumped Rust and Android fallback version metadata to 0.52.53.

### v0.52.52: Unified save management and destructive confirmation

- Moved save discovery, metadata, import, and export into the existing Load screen and removed the duplicate Saves tab from Options; Options now contains only Mods and Interface.
- Kept Load accessible when no saves exist so new installations can import saves from the title screen or in-game pause menu.
- Added Load, Export, and Delete actions to each retained, path-keyed save row. Import and Back remain in the fixed footer, and Android save-import staging is now owned by the lifecycle Load flow.
- Added a reusable modal confirmation primitive and a red danger palette for destructive actions. Save deletion permanently removes a file only after the player confirms.
- Added regression coverage for retained save-row identity, action-index updates, empty-catalogue import access, and confirmed/cancelled deletion.
- Bumped Rust and Android version metadata to 0.52.52.

### v0.52.51: Keyed dynamic UI row reconciliation

- Added stable keyed reconciliation for Options save rows and installed-mod rows, using save paths and stable mod IDs as identity. Existing row entities and controls now survive refreshes, while text, action labels, navigation indices, and selection feedback update in place.
- Applied the same reconciliation model to Character Inventory and Quest rows, keyed by persistent item and quest IDs. Added the quest ID to the renderer-neutral QuestView so row identity is independent of list position and display text.
- Kept the Options scroll viewport, tab-specific row host, Character hub scroll viewport, and Character footer mounted during same-tab data refreshes. Insertions, removals, and reordering affect only the rows that need changing.
- Kept Options messages and footer actions as independently refreshed regions; tab changes reset scroll and replace only the old tab's content.
- Added regression coverage for save-path and item/quest identity, row reordering, retained entity identity, obsolete-row removal, and updated action indices.
- Bumped Rust and Android version metadata to 0.52.51.

### v0.52.54: World Lab layout and shared button sizing

- Reworked World Lab and its map editor into a shared two-pane workspace: the map preview stays on the left, while generation settings, editor state, status, and actions stay on the right. Character Lab now uses the same workspace and panel layout helpers.
- Changed shared action buttons to use content-first widths instead of stretching across their parent groups. Each action fits its label, and the action group fits its widest button; compact controls retain their minimum touch dimensions.
- Kept map-grid preview rows unwrapped so their character dimensions remain intact.
- Added regression coverage for the shared lab layout and intrinsic action-button sizing.
- Bumped Rust and Android version metadata to 0.52.54.

### v0.52.55: Correct shared action-button text sizing

- Fixed shared standard, detail, and compact action buttons so text is measured intrinsically inside a definite available-width constraint instead of percentage constraints nested inside auto-sized nodes.
- Kept each button content-sized, capped its width to the available action group, and changed overflow wrapping to word boundaries so labels no longer break after one or two characters.
- Added structural regression coverage for all three shared text-button variants and their wrapping policy.
- Bumped Rust and Android version metadata to 0.52.55.

### v0.52.56: Equal-width shared action-button groups

- Added a width-bounded outer frame and intrinsic-width inner stack for shared vertical action groups.
- The stack sizes itself to its widest button within the available 520-logical-pixel bound, then stretches every button to that common width without cyclic percentage sizing or runtime width feedback.
- Kept compact option/footer controls and absolute-positioned map actions outside this shared vertical sizing behavior.
- Added regression coverage for both explicit keyed action groups and automatically organized button groups.
- Bumped Rust and Android version metadata to 0.52.56.

### v0.52.57: Restore full-width pause-menu actions

- Added an explicit full-width action stack for the pause menu, filling its bounded action column while preserving the existing button and text layout.
- Kept intrinsic-width sizing as the default for all other shared vertical action groups.
- Inset the developer-console close control from the upper-right edge to keep it clear of mobile display cutouts.
- Removed the redundant empty fast-travel notice from the World Map while retaining its Back action.
- Bumped Rust and Android version metadata to 0.52.57.


### v0.52.58: Renderer-neutral room and interior-door topology

- Added stable room identities with explicit walkable-cell membership and canonical grid-edge doors connecting adjacent rooms; interior doors do not consume floor cells, and map-edge travel passages remain separate.
- Added open, closed, and locked door states. Open doors permit movement and pathfinding; closed and locked doors block edge traversal. Locked doors can be explicitly unlocked to the closed state, while key ownership and lock-picking remain future gameplay rules.
- Integrated door-aware continuous spatial collision and contextual open, close, and locked-door interactions.
- Converted the authored Old Shrine chamber to explicit chamber/approach rooms with an open threshold door.
- Extended World Lab with room-cell assignment and interior-door authoring, including open/closed/locked initial states; fixture export validates topology.
- Added regression coverage for topology validation, canonical edges, door state transitions, contextual door actions, and closed-door pathfinding.
- Made room and door topology backward-compatible with earlier saves by defaulting missing fields to empty collections; existing saved map geometry is preserved and no inferred topology is added. Advanced the local-map generator and spatial fixture format versions, and bumped the save envelope to version 4 and Rust/Android version metadata to 0.52.58.


### v0.52.59: Ashen Gate and Hollow Market layouts with room-aware camera

- Added deterministic authored local layouts for Ashen Gate and Hollow Market, including the broken gate threshold, market stalls, unlit lantern posts, and explicit approach/court room membership.
- Added location-authored spatial discoveries for the cracked gate, dead road, abandoned stall, and unlit lantern.
- Routed graph-backed map exits through each authored threshold so additional connections do not cut arbitrary openings through the layouts. Hollow Market places its Old Shrine route beyond the market court and the Ashen Gate return route near the entry. Both thresholds begin open and remain subject to the shared open/close door rules.
- Added room-bound camera targeting and eased camera movement when crossing open interior doors. The camera retains map-wide clamping and falls back to the existing behavior in unassigned spaces and legacy maps without room topology.
- Advanced the local-map generator version to 5. Existing saved local maps remain authoritative and are not regenerated.
- Added deterministic layout, topology, exit reachability, door-blocking, and room-transition regression tests.
- Reused the current table and pillar art as first-pass market stall and lantern-post stand-ins; dedicated gate/market assets remain future art work.
- Bumped Rust and Android version metadata to 0.52.59.


### v0.52.60: Character-specific evidence progression

- Added character-owned learned evidence IDs, separate from shared physical point-of-interest discovery and interaction state. Successors retain the world and its physical traces but begin without the previous character's personal deductions.
- Added optional prerequisite evidence IDs to authored spatial discoveries. Examine actions are available only after all prerequisites are known by the current character; previously examined physical sites can be examined anew by a later character.
- Added content validation for missing, empty, repeated, self-referential, and cyclic evidence prerequisites.
- Added evidence-linked quest objectives and optional `required_evidence_ids` to quest templates. Campaign seeding appends evidence goals alongside the quest's standard objectives, and quest progress follows current-character knowledge.
- Preserved old-save readability with defaults for character evidence, POI prerequisites, and optional quest objective labels; the save envelope remains version 4.
- Added regression coverage for evidence examination, prerequisite gates, quest objective synchronization/seeding, inheritance boundaries, content validation, and legacy serialization.
- Bumped Rust and Android application metadata to 0.52.60.

### v0.52.61: Authored Gate-to-Shrine evidence investigation

- Connected the six existing Ashen Gate, Hollow Market, and Old Shrine discoveries into a prerequisite-gated investigation, following recent tracks from the gate to a disturbed market stall, a missing lantern wick, and matching soot at the shrine altar.
- Made the final Black Altar finding an evidence objective for "Quiet the Old Shrine"; the authored prerequisite chain ensures the preceding clues must be understood first.
- Added campaign-content and map-placement regression coverage for the clue order and prerequisite metadata, and documented the shipped investigation.
- Bumped Rust and Android application metadata to 0.52.61.

### v0.52.62: Data-driven persistent spatial encounter spawning (#470)

- Extended campaign encounter definitions with stable profile, location, and optional quest IDs; weighted selection; deterministic chance; and conditions for time of day, danger, world day, completed quests, and character conditions.
- Added deterministic on-entry profile selection derived from persistent map/location identity and world time, without consuming the general gameplay RNG stream. The selected combat profile is snapshotted into the local map and is not silently rerolled or respawned after materialization.
- Made local encounter records authoritative for nearby confrontation and combat startup while retaining the legacy global-threat path for older location-name-only content. Victory only clears a global threat associated with the defeated location.
- Linked alternative encounter profiles to the same authored quest objective, persisted quest attribution with the local encounter record so later content changes cannot sever that link, and added stable-ID mod extension/override behavior with legacy mod compatibility.
- Added focused coverage for weighted and conditional selection, deterministic materialization, no-reroll persistence, mod merging and validation, legacy deserialization/threat gating, local confrontation without a global threat, and quest credit for alternative profiles.
- Documented the current single-record-per-map boundary and left roaming entities, concurrent encounters, and real-time spatial combat to their later development work.
- Bumped Rust and Android application metadata to 0.52.62.
