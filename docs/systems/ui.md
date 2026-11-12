# UI System

The shipped frontend is Bevy. The UI layer is built around renderer-neutral presentation views and semantic interaction events so gameplay systems do not depend on GUI details. The product direction is graphics-first spatial gameplay: spatial views carry movement, exploration, environmental context, and the intended future action-combat experience. Text remains important for dialogue, descriptions, records, history, and contextual feedback, but is not the overriding presentation model. See [the canonical design principles](../design-principles.md).

## Screen architecture

The Bevy frontend uses dedicated flows for start, save selection, character creation, spatial exploration, the unified pause/navigation menu, Options, records, combat, developer console, quit, and death. The spatial-first function assignment and migration gates are tracked in [spatial-first-migration.md](spatial-first-migration.md). The spatial exploration scene is the intended primary activity space. NPC dialogue and remains recovery are contextual overlays hosted by that scene rather than a transition to another gameplay screen. `NavigationState` remains responsible for menu and lifecycle navigation. Dedicated screens capture their invoking screen and restore it on close, with SpatialExploration as the in-session fallback. The unified pause/navigation overlay can also open directly over SpatialExploration without replacing the active scene. Tapping the spatial header menu button, desktop Escape, and Android system Back open it; Back while it is open resumes the same spatial session. Holding the spatial header menu button for 650 ms without dragging opens the developer console on touchscreens. The former Gameplay dashboard and its Leave transition have been removed; ordinary in-session navigation returns to spatial exploration. Returning to SpatialExploration triggers a presentation rebuild from the existing spatial session rather than regenerating the map. Character, Options, the World Map, and safe-only Meditation are available through the pause menu. The World Map is the entry point for fast travel along known routes; physical passages remain independently usable through contextual interactions. Meditate appears only when the authoritative safe-to-meditate check passes and opens the existing meditation flow over the retained spatial context. Investigate appears beside Interact only for an active threat at the current location and opens the existing Combat screen through the authoritative combat entry point.

Lifecycle screens are owned by `bevy_lifecycle.rs`. The spatial pause/navigation overlay and interim World Map are owned by `bevy_gameplay.rs`. The spatial world and contextual target detection are owned by `bevy_spatial.rs`; the in-world dialogue/remains overlay state and presentation are owned by `bevy_interactions.rs`, which retains the previous standalone interaction renderer as a compatibility path. The Character hub, inventory and quest details, meditation, chronicle details, and journal writing are owned by `bevy_records.rs`. Combat is owned by `bevy_combat.rs`, and the developer console is owned by `bevy_console.rs`.

## Presentation boundary

`presentation.rs` contains frontend-independent view models. These models carry domain data such as strings, numbers, identifiers, and collections without depending on Bevy or any terminal UI library.

Bevy presentation adapters turn those views into UI nodes. Shared node construction, semantic visual theme tokens, screen roots, surfaces, overlays, labels, action/menu controls, health gauges, condition indicators, contextual messages, navigation state, and semantic input routing live in `bevy_presentation.rs`.

Tabbed screens keep their frame and shared tab strip stable after their first render. Tab changes replace only the active tab-content subtree, leaving the title, panel, and tab buttons intact. UI-scale reconciliation updates existing layout and touch-target dimensions only when interface preferences or viewport size changes; newly added nodes receive the current EmSize and touch-target values through an incremental pass.

The v0.51 graphical foundation keeps legacy panel helpers for existing screens while adding meaning-oriented primitives for the redesigned interface. New action and menu controls expose explicit interaction-state styling, health gauges keep their display values synchronized with their component state, and responsive spacing/touch-target tokens are centralized so new screens do not invent per-screen sizing rules. The dedicated Options screen reuses these primitives rather than introducing a separate menu architecture. Its Saves, Mods, and Interface tabs are shared across desktop and Android. Interface preferences independently adjust the full UI layout scale and the root font size; both values are persisted in the application's private configuration directory and applied at runtime. UI scaling affects fixed-size and viewport-relative layout dimensions, while font sizing remains independently configurable. The Options screen uses one shared landscape row primitive from `bevy_presentation.rs`: option names stay left and their horizontal controls/values stay right. Mods, saves, and Interface share the same row/button treatment. Each tab's entries live in a dedicated scroll viewport; the screen frame, tab strip, status messages, and bottom action footer remain outside that viewport. Save export actions stay beside their save entries; Save Import, Mod Install, Interface Reset, and Back actions are placed in the fixed footer rather than mixed into the scrolling entries. New option lists should reuse `spawn_option_row`, `spawn_compact_action_button`, `spawn_scrollable_list`, and `spawn_option_footer` rather than creating tab-specific row layouts.

## Input

Native Bevy keyboard, touch, and button interaction is translated into the semantic `InputEvent` model in `input.rs`. Gameplay and screen systems consume these semantic events rather than Bevy-specific keyboard types except where text entry requires native `KeyboardInput` text data.

Android touch positions are logical window coordinates while Bevy UI layout hit-testing uses physical coordinates, so the shared touch handling converts touch positions with the window scale factor before testing UI bounds. This conversion is shared by custom choice and scroll hit-testing.

Lifecycle character-creation fields use Bevy's `EditableText` with real `InputFocus`, tab navigation, cursor/edit handling, and native IME routing. Field selection is visual navigation state, not an implicit request to open the keyboard: native focus is assigned only when a field is activated by touch or explicitly opened from keyboard navigation. Switching creation tabs clears focus from a field that is no longer active. The shared presentation layer still gives fields explicit touch targets. On Android, the Java-hosted native text input is synchronized with the focused field so character commits remain available even when the Bevy IME event stream exposes only key events; dismissing the keyboard clears native focus and it stays dismissed until the field is explicitly focused again. The custom IME-to-keyboard bridge remains only for the developer console, whose input model is still renderer-specific.

Arrow keys, Home/End, Page Up/Page Down, Enter, Escape, Tab, Backspace, Delete, and selected number/vim-style shortcuts remain available where each screen supports them. UI buttons produce the same semantic confirmation events as keyboard input.

## Gameplay and results

The former Gameplay dashboard is retired and no longer has a renderer or navigation destination. Its remaining functions now live in the spatial experience: Map is opened from the pause menu, safe-only Meditation remains there, Investigate appears beside Interact only for an active local threat, and spatial exploration is entered directly through lifecycle completion. Character records, Options, and the developer console remain dedicated screens or tools. Recent history remains in its dedicated record screen. Using a physical passage invokes the established graph-travel rules and loads the connected local map without leaving spatial mode.

NPCs and searchable remains are positioned in the local map and become contextual targets only when the character is close enough and has a clear approach. One Interact prompt names the nearest valid target. Talking opens a themed overlay over the spatial scene, showing identity, faction, memory, availability, conversation choices, and responses. Searching remains opens the corresponding in-world inspection overlay, followed by the recovery result in the same overlay flow. The overlay has a scrollable content region and a fixed action footer. Character movement pauses while it is open; closing it resumes the same spatial session and position. The underlying interaction and recovery rules remain in the NPC interaction and legacy game modules, including quest offers/turn-ins, turn/time advancement, loot transfer, hidden discoveries, and recovery notes.

Combat presents player/enemy status, encounter events, action choices, and result details through Bevy nodes while the authoritative combat system continues to own resolution and state mutation.

The Character hub uses the shared tab strip and fixed-panel/scroll-list layout for General, Reputation, Inventory, Quests, and Journal. Journal brings together the chronological world chronicle and the character's personal notes; manually written notes remain available through the existing write-note flow and are not duplicated as chronicle entries. Inventory items, quests, and chronicle entries open detail views that return to their originating tab. Meditation remains a separate record flow. Results stay on-screen until the user advances or returns to the parent flow.

## Developer console

The developer console uses renderer-neutral `ConsoleView` data. Command editing, history navigation, completion candidates, scrolling, and command execution state remain in the game console modules; `bevy_console.rs` is responsible for graphical presentation and input handling.

The console opens from Gameplay or spatial exploration and returns to the screen that invoked it without a terminal-mode transition. On touchscreens, a normal tap on the spatial header menu button opens the pause/navigation menu; holding that same button for at least 650 ms without dragging opens the developer console. The desktop slash shortcut is also available from both Gameplay and spatial exploration.

## Visual design

The shared Bevy presentation layer uses an onyx `#070707` background with neutral charcoal panels and surfaces, fine warm-metal borders, and restrained brass interaction accents. Legacy panels remain available for existing screens, while `spawn_surface` and `spawn_overlay` provide the non-panel structural primitives used by the v0.51 graphical redesign.

Meaning-oriented helpers include compact icon+text action buttons, compact menu buttons, health gauges with overlaid current/max values, condition indicators, and contextual messages. The shared 32×32 pixel-art icons use a consistent worn-iron, stone, parchment, and muted-brass treatment; they remain individual semantic PNG assets under `data/assets/icons/ui/` so mods can continue replacing icons independently. Health gauges use a blood fill with a darker offset shadow layer. On Android, interactive controls retain a minimum 48dp logical hit area even when the surrounding UI is scaled down; the visual layout and content can still scale independently. Responsive spacing and fixed-size layout dimensions follow the configured UI scale, while typography uses the independently configurable root font size.

The unified gameplay pause/navigation menu is a centered overlay with a soft dimming layer so the underlying gameplay remains visible. Over spatial exploration it contains Resume, the Character hub, New Game, conditional Load Game, Options, and Quit. It also offers Map, which opens the focused World Map while retaining the spatial scene as context, and Meditate when the current authoritative state permits safe meditation. Known destinations can be selected for fast travel only through routes made entirely of visited reciprocal connections; each hop applies the normal travel action. Inventory, Quests, Reputation, and Journal are accessed as tabs inside Character. Opening a child screen from the menu preserves the paused-menu state so normal Back navigation returns to the same menu over spatial exploration. Android system Back and desktop Escape use the semantic Cancel path to open the menu over the retained spatial scene. Back dismisses the menu to spatial exploration. The spatial header menu button opens the unified menu on tap and the developer console on a 650 ms touch hold without dragging. There is no separate Gameplay dashboard or Leave action. The menu surface is scrollable so its full action set remains usable on smaller touch displays. Entering the menu also sets the explicit GameplayRuntimeState pause flag; spatial movement, animation, and camera following stop while the menu is open, while UI/input handling continues normally. The application starts fullscreen on desktop; Android uses its immersive, sensor-landscape Activity configuration.

Text remains a first-class presentation tool for dialogue, description, logs, records, and accessible contextual feedback. Within spatial gameplay, however, the world and player action should be communicated visually and spatially; artwork is not merely optional decoration for an otherwise text-first experience. Text and other fallbacks should keep content usable when assets are missing, without defining the intended overall presentation.

## Target visual composition (concept direction)

The intended interface is a dark, restrained frame around an expressive pixel-art world: near-black/charcoal surfaces, fine borders, warm brass/gold accents, compact iconography, and clear semantic colours for status. Let the environment dominate spatial gameplay rather than treating the world as a background to a dashboard. Use contrast, silhouettes, layered depth, lighting, and animation to keep people, threats, routes, and interactable objects readable.

The following layouts interpret the approved visual reference. They describe the intended information hierarchy, not a claim that every element already exists in the current build. Exact labels, numbers, icons, and key bindings may change as systems and usability tests mature.

### Exploration HUD

- Keep essential character status visible without covering the view: a portrait or clear character marker with health and other currently relevant vitals.
- Place current objective/quest guidance in a compact, dismissible area, and show a contextual interaction prompt near the person, object, exit, or point of interest it refers to.
- Make local orientation and time context available with a compact compass/minimap and a readable time-of-day cue where useful.
- Keep secondary shortcuts compact and contextual. Avoid permanent panels for systems that are not relevant to the current activity.
- Use in-world markers and visual feedback to distinguish nearby opportunities from background scenery without turning every object into a labelled icon.

### Real-time combat HUD

- Preserve the spatial view and make combat state legible at a glance: target identity and health, player vitals, available abilities, and important resource/cooldown feedback.
- Communicate enemy attacks and dangerous areas through clear spatial telegraphs, not text alone.
- Show nearby allies and their current state when that information affects a decision. Offer a small set of plausible local commands and access to slow-motion tactical orders; do not present a remote army-command interface.
- Let the combat HUD add detail to the exploration HUD when needed, then recede when the encounter ends.

### Character hub, inventory, and equipment

- Treat the character screen as a navigable hub rather than one oversized panel. The current tabs are General, Reputation, Inventory, Quests, and Journal; future skills, relationships, work, followers, and legacy can become sections or tabs as their systems are implemented.
- Make identity and progression easy to scan: portrait, name/background, level or milestones, relevant attributes, conditions, and traits.
- Show equipped gear in relation to the character, with identifiable slots and an item grid or list that can be filtered by category. Communicate item properties and carrying limits where they matter.
- Keep equipment, inventory, skills, and records closely connected without forcing every detail onto the spatial HUD.

### World map and travel

- Use a readable regional map to present known locations, routes or connections, points of interest, and concise location context. Make travel a deliberate action from this view.
- Let map annotations reflect what the character knows. A location summary may surface services, danger, or useful opportunities when that information is available.
- The map supports travel between local spatial areas; it is not a replacement for spatial exploration or a remote grand-strategy control surface.

### Shared feedback and mobile adaptation

- Reuse consistent visual patterns for buttons, status/resource bars, icons, item pickup and quest notifications, dialogue, and contextual messages. Use colour alongside iconography or text so meaning is not communicated by colour alone.
- On Android, preserve the same gameplay priorities while reflowing and scaling the HUD for a smaller screen. Provide reachable touch controls for movement and primary actions, with larger contextual/combat controls placed where thumbs can comfortably reach them. The virtual joystick and action buttons should not obscure the character, nearby interactions, or important combat telegraphs.
- On desktop, keep keyboard/mouse guidance available where useful; the reference's WASD, interaction key, ability numbers, and character-menu key are examples, not immutable bindings. Controls should remain discoverable and remappable where supported.
- Adapt composition per platform rather than shrinking a desktop screenshot into a phone viewport. Maintain readable text, clear hit targets, and the ability to see the world while acting.

### Spatial exploration vitals

Spatial exploration now keeps a compact character panel in the header with character name/level, current and maximum health, and a text summary of active conditions. It reuses the common health-gauge primitive and shared dark theme, so the essential status remains visible without occupying a large permanent panel. This does not imply that the planned combat HUD, minimap, or ally command controls exist yet.

## v0.51 graphical foundation

Issue #278 establishes the reusable presentation primitives for the redesigned gameplay HUD and menu surfaces. Issue #279 applies those primitives to the gameplay surface with a world-first composition, responsive primary/contextual actions, dynamic time-of-day sky treatment, and the compact player HUD. Issue #317 unifies the former secondary-navigation and pause flows into the gameplay pause/navigation menu, while #227 supplies the dedicated tabbed Options screen.

## Design direction

Keep screen responsibilities separate from gameplay mechanics. New UI work should reuse frontend-neutral view models and semantic input events rather than introducing renderer-specific dependencies into gameplay rules. Adapt layouts and controls to desktop and mobile, supporting touch, mouse/keyboard, and controllers where practical. Keep spatial gameplay uncluttered, use contextual actions for nearby interactions, and reserve detailed management/information for hierarchical record screens. Avoid maintaining parallel terminal and graphical implementations or undertaking a wholesale UI redesign without a specific need.


## Spatial interaction presentation direction

Spatial interaction discovery exposes stable action/target descriptors independently of the UI. The current contextual prompt continues to activate the nearest eligible descriptor until the radial wheel in #405 is implemented. Conversation view data now carries generated portrait-layer selections for both the player and NPC; the existing overlay remains temporary, and the dedicated close-up two-character dialogue scene is future work. The weighted generator is renderer-neutral; the frontend is responsible for loading and compositing its resolved layers.
