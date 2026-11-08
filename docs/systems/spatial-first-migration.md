# Spatial-first function map

This document audits the useful functions currently exposed by the Gameplay dashboard and pause/navigation flow, then assigns each one a proposed home in the spatial-first experience. It is the implementation map for issue [#421](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/421).

**Status:** function audit complete; the existing unified pause menu is now available over spatial exploration; the remaining dashboard functions are still being migrated.

The placements below follow the project's established direction: keep frequent actions contextual, make deliberate activities convenient, and put detailed information and infrequent system operations in focused screens. They are architectural assignments, not approval of a specific button layout or visual design.

## Current spatial shell

The current spatial screen provides:

- Current location name and a local-exploration heading.
- Character name, level, health, and active-condition summary.
- A status/message strip for movement, discovery, and arrival feedback.
- An Interact control that opens the player-anchored radial action wheel when eligible targets exist.
- An explicit Leave control.
- Desktop movement through WASD/arrow keys and interaction through E/Confirm.
- Android movement through the virtual joystick and touch activation of the visible controls.

Desktop Escape and Android system Back now open the existing unified pause menu over the spatial scene. Back while the menu is open dismisses it and resumes the same spatial session. The menu retains its existing Resume, Character, New Game, conditional Load Game, Options, and Quit actions; its child screens continue returning to the paused spatial context. The visible Leave control remains a separate, explicit transition to the compatibility Gameplay dashboard. While the menu is open, spatial movement, player animation, camera follow, and Android joystick input are paused.

The spatial screen already rebuilds its presentation after returning from a dedicated screen, using the retained local map and continuous character position. NavigationState now captures and restores the invoking screen. Those foundations make spatial entry points possible without moving the player or regenerating the map.

## Function-to-surface assignment

| Existing function | Current owner and behavior | Proposed spatial home | Migration status |
|---|---|---|---|
| Current location and region | WorldView and LocationView; the dashboard shows location context. | Spatial header and local environment. The spatial header already identifies the current location; region/detail can remain secondary context rather than a permanent panel. | Partially present. |
| Time and day/night | WorldView provides time, day, and time_points; the dashboard derives its sky treatment from those values. | Environmental presentation or a compact time cue, not another large information panel. | Not represented in the spatial HUD yet. |
| Character vitals | bevy_gameplay.rs renders the dashboard vitals; bevy_spatial.rs already reads authoritative character data for the spatial vitals panel. | Keep the compact spatial vitals panel. Do not create a second HUD copy or independent values. | Present in spatial exploration. |
| Transient feedback and messages | bevy_feedback.rs presents level-up and quest-reward feedback and remembers the screen to which it should return. Spatial movement/discovery/arrival uses the spatial status message. | Keep short local feedback in the spatial status area; use the existing feedback overlay for consequential results and choices. | Existing mechanisms support the spatial context. |
| Travel to a connected local area | bevy_spatial.rs exposes physical passages as contextual targets and transitions through the existing world graph. | Keep physical passages in the world and discoverable through the interaction wheel. | Present in spatial exploration. |
| Travel through the destination list | navigation::build_view builds the current location and its connected destinations; the Gameplay Travel action opens that selection flow. | Keep a deliberate destination-selection surface accessible from spatial play. It may be reached through the system/quick menu or a compact map/travel entry point; the precise affordance remains a design decision. | Destination selection is not yet reachable from the spatial screen. |
| Explore the current location | The Gameplay Explore action calls bevy_spatial::enter. | Treat this as a transition into the primary gameplay context, not as a permanent action inside that context. After the dashboard is retired, loading/starting an active character should enter the appropriate spatial context directly. | Current entry still depends on the dashboard. |
| Investigate an active threat | The dashboard exposes Investigate only when the authoritative threat state allows it; activation uses bevy_combat::begin and opens the existing Combat screen. | Surface it contextually when the threat is active. Continue to call the authoritative combat entry point; do not invent a separate investigation/combat ruleset or a spatial threat actor solely to host a button. | Current action exists outside spatial play; exact spatial presentation remains open. |
| Meditate | bevy_records.rs owns the Meditation screen and checks the authoritative safe-to-meditate rule and available targets. | Provide a convenient spatial entry when meditation is valid. Safe-location convenience is a requirement: do not require walking to an artificial point just to meditate. | Dedicated screen and rules exist; spatial entry is missing. |
| Character records | bevy_records.rs owns the Character hub and its General, Reputation, Inventory, Quests, and Journal tabs. | Open the existing Character hub as a dedicated screen from spatial play. Its Back action should return to the same map and player position. Do not split Inventory, Quests, or Journal into unrelated permanent HUD panels. | Return routing is implemented; spatial entry is missing. |
| Talk, search remains, examine, and enter passages | bevy_spatial.rs and interaction_actions.rs discover eligible nearby action/target pairs; the radial wheel presents them. bevy_interactions.rs handles dialogue and remains overlays. | Keep these as proximity- and eligibility-driven contextual actions. They should not return as detached dashboard buttons. | Present in spatial exploration. |
| Options | bevy_options.rs owns the Saves, Mods, and Interface tabs. The screen returns to the screen that opened it. | Open the existing Options screen from the spatial system menu. Keep its detailed controls in the dedicated screen. | Return routing is implemented; spatial entry is missing. |
| Resume, New Game, Load Game, Quit | GameplayScreen::Pause owns these actions. New/load/quit continue through the lifecycle flows. | Reuse the existing pause menu as an overlay over spatial play. Resume or Back returns to the same spatial session; the other actions retain their existing lifecycle behavior. | Available from spatial exploration through Escape/Android Back. |
| Developer console | bevy_console.rs opens from the Gameplay flow; the touchscreen hold gesture is attached to the Gameplay pause control. | Keep developer access separate from the player-facing system menu. Preserve its developer-only entry behavior while updating any necessary spatial shortcut separately. | Not a player gameplay function; spatial shortcut is not implemented. |
| Leave spatial exploration | The spatial header has an explicit Leave control. | Keep it independent from Back navigation. It remains a temporary exit to the Gameplay dashboard while that dashboard is still the compatibility shell. Its eventual removal belongs to the final dashboard-retirement review. | Explicit action retained; Back no longer triggers it. |

## Authoritative behavior that must remain unchanged

The spatial interface must route into existing gameplay behavior rather than reproduce it:

- Travel continues to use the location graph and existing travel actions. Physical passages remain distinct from destination-list navigation.
- Meditation continues to enforce the safe-location and target rules in the action layer.
- Investigation continues to begin through the existing combat adapter and authoritative combat system.
- Character records continue to use the existing Character hub and tab/detail navigation.
- Options, save import/export, mods, and interface preferences remain owned by the Options flow.
- New Game, Load Game, and Quit continue through lifecycle state transitions.
- NPC dialogue, remains recovery, points of interest, and passage eligibility continue to use the existing spatial interaction discovery and interaction systems.
- World time, quests, history, reputation, and persistence remain authoritative in the game state. The spatial presentation does not create parallel copies of these values.

## Recommended implementation order

1. **Spatial system-menu access (implemented in v0.52.26).** Escape/Android Back opens the existing pause menu over the spatial scene; Back dismisses it to the same map and player position. The Leave control remains a separate explicit action.
2. **Travel entry.** Expose the existing destination selection from spatial play while keeping physical passages as an in-world option. Do not add a second travel model.
3. **Meditation entry.** Make the existing safe meditation flow accessible from spatial play without requiring unnecessary movement.
4. **Character entry.** Open the existing Character hub from spatial play and verify all tabs and detail screens return to the exact spatial context.
5. **Threat/investigation presentation.** Decide how an active threat is communicated in spatial exploration, then expose the existing Investigate action when eligible.
6. **Lifecycle hand-off.** Only after the spatial menu and entry points cover the required functions should new/load/start/quit flows be adjusted to make spatial exploration the normal active-character shell.
7. **Retirement review.** Remove the dashboard and its transition-only Leave path only after the function audit has been implemented and validated on desktop and Android landscape.

## Remaining design decisions before further HUD work

The following points affect how the remaining dashboard functions are exposed and should be settled before implementing their spatial entry points:

- **Travel discovery:** should destination-list travel live inside the system/quick menu, or have its own compact map/travel control in the spatial HUD? Physical passages remain available either way.
- **Meditation discovery:** should a safe-only Meditate action be directly available in the spatial action area, or live one level inside a contextual/quick menu? It must remain convenient in safe locations.
- **Threat presentation:** should an active threat produce a contextual Investigate prompt, or be represented through another spatial indicator? The action should only appear when the existing threat state makes it valid.
- **Time presentation:** should time-of-day be communicated through environmental lighting/sky treatment, a small cue, or both? The dashboard's large sky composition should not simply be copied over the spatial view.

The pause-menu integration does not retire the dashboard. It remains the compatibility fallback until the other required functions have implemented and validated spatial replacements.
