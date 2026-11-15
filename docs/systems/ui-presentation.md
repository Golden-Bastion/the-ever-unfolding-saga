# UI presentation architecture

## Responsibilities

The game model and renderer-neutral view builders remain authoritative for gameplay meaning. Screen modules translate those views into Bevy UI and convert input into semantic game actions. The shared presentation layer owns reusable layout primitives, visual tokens, interaction feedback, screen transitions, scroll handling, and common input adapters.

The boundary is intentional: screen modules should describe what a screen contains, while the shared presentation layer should define how common controls and surfaces behave and appear.

## Interaction updates

Navigation-only selection changes are not content changes. The shared sync_action_button_selection system applies the active navigation index to existing ChoiceButton entities by adding or removing UiSelected. Screen input handlers for lifecycle menus, interactions, combat, records, and developer tools therefore do not need to mark their complete view dirty just to move a highlight.

Button hover, pressed, and selected colours are interpolated over time by the shared presentation system. The same target colours and timing apply to ordinary and warm-accent action buttons. Standard screen roots use the shared entry transition rather than each screen inventing its own appearance animation.

## Stable screen regions

Character Lab retains its root, workspace, viewer host, and scrollable controls host after first creation. When its data changes, only the children of the viewer and controls hosts are replaced. The controls host itself remains alive, so its current ScrollPosition is not discarded during an appearance edit.

The developer console keeps its screen root, close control, output scroll region, and input control mounted. Its renderer compares the previous and current renderer-neutral ConsoleView and updates output, autocomplete, and input independently. Typing therefore changes the input label and, when needed, autocomplete rows without rebuilding the output history or replacing the input control that owns focus. The output and autocomplete lists have separate child hosts inside one retained scroll region.

Combat retains its screen root and actor, event-log, and action panels between turns. Player/enemy HP gauges and status labels update in place through stable component markers; only the combat-log rows are replaced when new events arrive. Navigation selection still flows through the shared action-button selection system and does not mark the combat view dirty.

Talk, conversation, remains, and remains-result screens retain their outer root and content host while their current screen state remains active. The content host is the only region rebuilt for a new dialogue line, message, result, or refreshed list; the ordinary talk/remains scroll panel and the specialized conversation scene root remain mounted. Changing between interaction modes creates the appropriate shell once for the new mode.

Character Records now gives Inventory Detail, Quest Detail, Meditation, Meditation Result, Chronicle Detail, and Journal Entry screen-specific roots with named scrollable content hosts. Refreshes within the same record screen replace only the host children, keeping the root heading and scroll panel mounted. Moving between record screen types creates the matching shell rather than reusing a heading or content surface from another screen. Journal drafts remain in screen state while their visible content refreshes.

The lifecycle creation, Load, and Options/Character tab views use stable content hosts. Load keeps its scroll viewport, keyed save-row host, feedback region, and footer mounted while save data or messages refresh. Options retains its active tab's scrolling list, separate feedback-message region, and footer. A content refresh replaces only the children of the relevant hosts, keeping the scrolling panel and its live position intact; switching Options tabs resets the list to the new tab's start.

Feedback overlays retain their outer scrollable surface while queued level-up and quest-reward entries advance. Only the current entry's content children are replaced, preserving the overlay entity and scroll state. The shared touch-scroll handling also owns the overlay on Android, so long feedback remains reachable without changing its presentation shell.

The spatial scene continues to update player position, directional animation, camera follow, and discovery tint in place. World-clock changes no longer rebuild the entire spatial scene: they update only the marked atmosphere tint, sky panel, sun/moon position, horizon accent, and time readout. The world topology and ordinary spatial UI are rebuilt only through their existing explicit dirty/re-entry path. The interim World Map presentation and behavior are unchanged by this work.

## Visual language

The shared palette remains onyx-black with restrained warm accents. Panels use a consistent bordered surface, action buttons use compact corners and a common touch-target size, and typography continues to scale through the existing responsive font tokens. Feedback should clarify hierarchy and interaction without adding decorative effects that compete with the world.

The broader approved visual direction is defined in [Pixel-Art Visual Direction](../design/pixel-art-direction.md), while screen composition and information hierarchy are defined in [Presentation Direction](../design/presentation-direction.md). The dark interface frame must remain legible over both bright natural environments and dark interiors; do not solve contrast problems by making every world scene uniformly dark.

## Keyed dynamic rows

Load-screen save rows are keyed by save path; installed-mod rows in Options are keyed by stable mod ID. Character Inventory rows use persistent item IDs, while Quest rows use persistent quest IDs carried through the renderer-neutral quest view. A refresh updates the existing row's text, action labels, selection index, and selected feedback in place. Insertions create only new rows, removals despawn only obsolete rows, and reordering changes the parent-child order without replacing unaffected row entities.

The Load screen retains its viewport, keyed save-row host, message region, and footer across same-screen refreshes. Save rows expose Load, Export, and Delete actions; Import and Back stay in the fixed footer. The shared confirmation dialog is hosted as a modal overlay, and its confirm action uses the danger palette. Options now contains only Mods and Interface; its scroll viewport and row host remain mounted during same-tab refreshes. Message and footer regions are refreshed independently. Switching Options tabs resets the scroll position. The Character hub retains its scroll viewport, row host, and footer; Inventory/Quest refreshes reconcile keyed rows while changes between unrelated tabs replace only the row contents and reset the scroll position.

## Ongoing contract

The retained-shell and keyed-row migration across the current high-churn screens has been completed and owner-validated through playtesting. New dynamic collections and screen regions should adopt the same stable-host and keyed-identity contract as they are introduced or migrated.

Desktop and Android behavior must remain equivalent. Stable hosts should preserve scroll position, focus, selection, and touch-gesture ownership across content updates.
