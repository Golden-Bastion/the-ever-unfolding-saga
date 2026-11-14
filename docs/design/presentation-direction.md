# Presentation Direction

> Design record migrated from the former future-composition section of [docs/systems/ui.md](../systems/ui.md). This document preserves interface and visual-composition intent; implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md).

## Visual language and composition

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


## Boundaries

These are information-hierarchy and visual-direction goals, not a specification for the current screen layout or a claim that every listed control already exists. Current screen behavior is described in [system documentation](../systems/); future gameplay mechanics are described in their respective design specifications. The interim World Map's implemented behavior remains documented in [World Map](../systems/world-map.md).
