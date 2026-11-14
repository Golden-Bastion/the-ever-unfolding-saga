# UI presentation architecture

## Responsibilities

The game model and renderer-neutral view builders remain authoritative for gameplay meaning. Screen modules translate those views into Bevy UI and convert input into semantic game actions. The shared presentation layer owns reusable layout primitives, visual tokens, interaction feedback, screen transitions, scroll handling, and common input adapters.

The boundary is intentional: screen modules should describe what a screen contains, while the shared presentation layer should define how common controls and surfaces behave and appear.

## Interaction updates

Navigation-only selection changes are not content changes. The shared sync_action_button_selection system applies the active navigation index to existing ChoiceButton entities by adding or removing UiSelected. Screen input handlers for lifecycle menus, interactions, combat, records, and developer tools therefore do not need to mark their complete view dirty just to move a highlight.

Button hover, pressed, and selected colours are interpolated over time by the shared presentation system. The same target colours and timing apply to ordinary and warm-accent action buttons. Standard screen roots use the shared entry transition rather than each screen inventing its own appearance animation.

## Stable screen regions

Character Lab retains its root, workspace, viewer host, and scrollable controls host after first creation. When its data changes, only the children of the viewer and controls hosts are replaced. The controls host itself remains alive, so its current ScrollPosition is not discarded during an appearance edit.

The lifecycle creation and Options/Character tab views already use stable content hosts for their tab frames. Scroll-position snapshots remain a compatibility fallback for screens that still recreate scrollable panels.

## Visual language

The shared palette remains onyx-black with restrained warm accents. Panels use a consistent bordered surface, action buttons use compact corners and a common touch-target size, and typography continues to scale through the existing responsive font tokens. Feedback should clarify hierarchy and interaction without adding decorative effects that compete with the world.

## Remaining migration

This work establishes the first retained-presentation slice. It does not yet make every screen retained: several screen modules still destroy and recreate their full UI trees when their underlying content changes. Continue migrating those screens toward:

1. A stable root and stable layout frame owned by each ScreenId.
2. Named content regions for independently changing parts of a screen.
3. In-place updates for values, gauges, labels, and selection state.
4. Keyed row reconciliation for dynamic lists, preserving entity identity where items remain.
5. Explicit lifecycle transitions that animate out before a screen is removed, without delaying input or gameplay state changes.

Keep desktop and Android behavior equivalent. Stable hosts must preserve scroll position, focus, selection, and touch gesture ownership across content updates.
