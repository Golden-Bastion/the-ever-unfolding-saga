# Spatial-first function map

This document audits the useful functions currently exposed by the Gameplay dashboard and pause/navigation flow, then assigns each one a proposed home in the spatial-first experience. It is the implementation map for issue [#421](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/421).

**Status:** function audit complete; the unified pause menu, destination-list travel, safe-only meditation entry, and contextual threat investigation are available over spatial exploration; the remaining dashboard functions are still being migrated.

The placements below follow the project's established direction: keep frequent actions contextual, make deliberate activities convenient, and put detailed information and infrequent system operations in focused screens. They are architectural assignments, not approval of a specific button layout or visual design.

## Current spatial shell

The current spatial screen provides:

- No generic heading is shown during normal spatial play; the compact time-of-day indicator stands on its own. A location name appears only in a brief arrival notice on entry or travel.
- Character name, level, health, and active-condition summary.
- A status/message strip for movement and discovery feedback, with a temporary arrival notice for the current location.
- An Interact control that opens the player-anchored radial action wheel when eligible targets exist.
- An explicit Leave control.
- Desktop movement through WASD/arrow keys and interaction through E/Confirm.
- Android movement through the virtual joystick and touch activation of the visible controls.

Desktop Escape and Android system Back now open the existing unified pause menu over the spatial scene. Back while the menu is open dismisses it and resumes the same spatial session. The menu retains its existing Resume, Character, New Game, conditional Load Game, Options, and Quit actions; its child screens continue returning to the paused spatial context. The visible Leave control remains a separate, explicit transition to the compatibility Gameplay dashboard. While the menu is open, spatial movement, player animation, camera follow, and Android joystick input are paused.

The spatial screen already rebuilds its presentation after returning from a dedicated screen, using the retained local map and continuous character position. NavigationState now captures and restores the invoking screen. Those foundations make spatial entry points possible without moving the player or regenerating the map.

## Function-to-surface assignment

| Existing function | Current owner and behavior | Proposed spatial home | Migration status |
|---|---|---|---|
| Current location and region | WorldView and LocationView; the dashboard shows location context. | Keep the location name out of the persistent spatial header and announce it briefly on entry or travel. A future focused World Map should show the current region and current location together. | Temporary location announcement implemented; World Map remains future work. |
| Time and day/night | WorldView and the world clock expose day and 12 two-hour time portions; the dashboard already maps those portions to a sky palette and the mod-aware sun/moon atlas. | Reuse the same sky phases in a compact horizon-and-celestial display with a 24-hour readout. Apply restrained warm dawn/dusk and cool night colour washes to the spatial environment, keeping the HUD above the wash. | Implemented in v0.52.30; both the environmental treatment and compact sky/clock are driven by authoritative world time. |
| Character vitals | bevy_gameplay.rs renders the dashboard vitals; bevy_spatial.rs already reads authoritative character data for the spatial vitals panel. | Keep the compact spatial vitals panel. Do not create a second HUD copy or independent values. | Present in spatial exploration. |
| Transient feedback and messages | bevy_feedback.rs presents level-up and quest-reward feedback and remembers the screen to which it should return. Spatial movement/discovery/arrival uses the spatial status message. | Keep short local feedback in the spatial status area; use the existing feedback overlay for consequential results and choices. | Existing mechanisms support the spatial context. |
| Travel to a connected local area | bevy_spatial.rs exposes physical passages as contextual targets and transitions through the existing world graph. | Keep physical passages in the world and discoverable through the interaction wheel. | Present in spatial exploration. |
| Travel through the destination list | navigation::build_view builds the current location and its connected destinations; the Gameplay Travel action opens that selection flow. | Make the existing destination selector available from the spatial pause menu. Keep travel out of the persistent HUD and preserve physical passages as an in-world alternative. | Available from the spatial pause menu; uses the existing graph and travel action. |
| Explore the current location | The Gameplay Explore action calls bevy_spatial::enter. | Treat this as a transition into the primary gameplay context, not as a permanent action inside that context. New character creation, inheritance, and successful save loading should enter the appropriate spatial context directly. | Lifecycle completion now enters spatial exploration directly; the explicit Leave control still opens the compatibility dashboard. |
| Investigate an active threat | The dashboard exposes Investigate only when the authoritative threat state allows it; activation uses bevy_combat::begin and opens the existing Combat screen. | Show an Investigate control beside Interact only while an active threat belongs to the current location. Continue to call the authoritative combat entry point; do not invent a separate investigation/combat ruleset or a spatial threat actor solely to host a button. | Available beside Interact while an eligible threat is active. |
| Meditate | bevy_records.rs owns the Meditation screen and checks the authoritative safe-to-meditate rule and available targets. | Offer Meditate in the spatial pause menu only while the authoritative safe-to-meditate check passes. Open the existing dedicated screen and return to the same paused spatial context. | Available through the spatial pause menu when safe. |
| Character records | bevy_records.rs owns the Character hub and its General, Reputation, Inventory, Quests, and Journal tabs. | Open the existing Character hub as a dedicated screen from the spatial pause menu. Its Back action returns to the same paused spatial scene. Do not split Inventory, Quests, or Journal into unrelated permanent HUD panels. | Available through the spatial pause menu; regression tests cover all five tabs and nested detail/journal back paths retaining the spatial parent. |
| Talk, search remains, examine, and enter passages | bevy_spatial.rs and interaction_actions.rs discover eligible nearby action/target pairs; the radial wheel presents them. bevy_interactions.rs handles dialogue and remains overlays. | Keep these as proximity- and eligibility-driven contextual actions. They should not return as detached dashboard buttons. | Present in spatial exploration. |
| Options | bevy_options.rs owns the Saves, Mods, and Interface tabs. The screen returns to the screen that opened it. | Open the existing Options screen from the spatial pause menu. Keep its detailed controls in the dedicated screen. | Available through the spatial pause menu. |
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
2. **Travel entry (implemented in v0.52.27).** The spatial pause menu opens the existing destination selector. Selection uses the existing travel action, persists the source map, and enters the destination's spatial map; Back returns to the pause menu. Physical passages remain available in the world.
3. **Meditation entry (implemented in v0.52.28).** The spatial pause menu shows Meditate only when the authoritative safe-to-meditate check passes; the existing screen and target rules remain unchanged.
4. **Character entry (verified in v0.52.31).** Keep the Character hub and all nested inventory, quest, chronicle, and journal flows attached to their spatial invoking context; regression tests cover every top-level tab and nested Back path.
5. **Threat/investigation presentation (implemented in v0.52.29).** Show Investigate beside Interact only while an active threat belongs to the current location; start the existing Combat flow through `bevy_combat::begin`.
6. **Time and day/night presentation (implemented in v0.52.30).** Reuse the existing 12-part world clock and sun/moon atlas for a compact spatial sky display and 24-hour readout. Apply a subtle environment-wide dawn/dusk/night colour wash beneath the HUD; do not add a second time simulation or copy the dashboard's large sky composition.
7. **Lifecycle hand-off (implemented in v0.52.31).** New character creation, inheritance, and successful save loading enter spatial exploration directly. Keep the explicit Leave route as a temporary compatibility path to the Gameplay dashboard.
8. **Region and location context (partially implemented).** Announce the current location briefly on entry or travel rather than displaying its name permanently. Keep the region/location hierarchy for a future focused World Map that shows both names; do not add a permanent region panel to the spatial HUD.
9. **Retirement review.** Remove the dashboard and its transition-only Leave path only after the function audit has been implemented and validated on desktop and Android landscape.

## Remaining migration work

The time-of-day presentation and lifecycle hand-off are implemented. The existing world clock remains authoritative; no time advancement, save format, or gameplay rule has been changed. New lives and successful save loads now enter the spatial scene directly, while the explicit Leave control still provides a temporary route to the compatibility dashboard. Location names now appear as brief arrival notices rather than permanent header text.

The dashboard is not retired. A future focused World Map is the planned home for showing the current region and current location together; that map is not implemented yet. Keep region detail out of the persistent HUD. The final dashboard-retirement review must account for all remaining functions and wait until desktop/Android landscape validation is performed after implementation work is complete.
