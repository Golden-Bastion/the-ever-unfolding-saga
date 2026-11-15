# Presentation Direction

> This document defines the intended information hierarchy and interface composition. Shared artistic principles are defined in [Pixel-Art Visual Direction](pixel-art-direction.md); implementation status belongs in the root [roadmap](../../ROADMAP.md), and current screen behavior belongs in [system documentation](../systems/).

## Visual language and composition

The interface is a dark, restrained frame around an expressive, beautiful world. Use near-black and charcoal surfaces, fine borders, restrained warm brass/gold accents, compact iconography, and clear semantic colours for status. The existing onyx `#070707` remains the primary interface background token; it does not define the colour treatment of the game world.

The world should remain the visual focus during spatial gameplay. Do not treat it as a background behind a permanent dashboard. Use silhouettes, layered depth, lighting, material distinction, and animation to keep people, threats, routes, and interactable objects readable. The interface should be legible and deliberate without competing with environmental composition.

The approved visual references apply to interface work as well as world art, but their influence is selective:

- **The Witcher 3:** contextual presentation, restrained information density, readable hierarchy, and the ability for interface elements to recede when they are not needed.
- **Kingdom Come: Deliverance:** practical clarity, believable object and equipment presentation, and interfaces that feel grounded rather than ornamental.
- **Dark Souls Remastered:** restraint, strong contrast, gothic atmosphere, and deliberate use of visual emphasis.

These references are not instructions to copy layouts, icons, typography, or exact visual treatments. TEUS should use their principles to build a consistent interface of its own. Baldur's Gate II is not an approved visual reference for this project.

The following layouts describe intended information hierarchy, not a claim that every element already exists in the current build. Exact labels, numbers, icons, and key bindings may change as systems and usability tests mature.

## Exploration HUD

- Keep essential character status visible without covering the view: a portrait or clear character marker with health and other currently relevant vitals.
- Place current objective/quest guidance in a compact, dismissible area, and show a contextual interaction prompt near the person, object, exit, or point of interest it refers to.
- Make local orientation and time context available with a compact compass/minimap and a readable time-of-day cue where useful.
- Keep secondary shortcuts compact and contextual. Avoid permanent panels for systems that are not relevant to the current activity.
- Use in-world markers and visual feedback to distinguish nearby opportunities from background scenery without turning every object into a labelled icon.
- Keep HUD contrast sufficient over bright snow, pale skies, water, foliage, firelight, and dark interiors. Use controlled backing surfaces or edge contrast where necessary instead of darkening the entire world to accommodate the HUD.

## Real-time combat HUD

- Preserve the spatial view and make combat state legible at a glance: target identity and health, player vitals, available abilities, and important resource/cooldown feedback.
- Communicate enemy attacks and dangerous areas through clear spatial telegraphs, not text alone.
- Show nearby allies and their current state when that information affects a decision. Offer a small set of plausible local commands and access to slow-motion tactical orders; do not present a remote army-command interface.
- Let the combat HUD add detail to the exploration HUD when needed, then recede when the encounter ends.
- Keep telegraphs and combat effects visually distinct from ordinary environmental light and decoration.

## Character hub, inventory, and equipment

- Treat the character screen as a navigable hub rather than one oversized panel. The current tabs are General, Reputation, Inventory, Quests, and Journal; future skills, relationships, work, followers, and legacy can become sections or tabs as their systems are implemented.
- Make identity and progression easy to scan: portrait, name/background, level or milestones, relevant attributes, conditions, and traits.
- Show equipped gear in relation to the character, with identifiable slots and an item grid or list that can be filtered by category. Communicate item properties and carrying limits where they matter.
- Keep equipment, inventory, skills, and records closely connected without forcing every detail onto the spatial HUD.
- Use the same grounded material language as the world for item and equipment art. Avoid decorative rarity treatments that obscure the item's actual form or make the interface visually noisy.

## World map and travel

- Use a readable regional map to present known locations, routes or connections, points of interest, and concise location context. Make travel a deliberate action from this view.
- Let map annotations reflect what the character knows. A location summary may surface services, danger, or useful opportunities when that information is available.
- The map supports travel between local spatial areas; it is not a replacement for spatial exploration or a remote grand-strategy control surface.
- The current interim World Map is a topological graph and is not required to imitate a geographic map until the world-topology foundations support one.

## Shared feedback and mobile adaptation

- Reuse consistent visual patterns for buttons, status/resource bars, icons, item pickup and quest notifications, dialogue, and contextual messages. Use colour alongside iconography or text so meaning is not communicated by colour alone.
- Keep interaction states clear and consistent: default, hover, pressed, selected, disabled, and destructive actions must be distinguishable without relying on colour alone.
- Use animation to communicate transitions and state changes, not as decoration. Respect reduced visual space and avoid motion that delays interaction or obscures information.
- On Android, preserve the same gameplay priorities while reflowing and scaling the HUD for a smaller screen. Provide reachable touch controls for movement and primary actions, with larger contextual/combat controls placed where thumbs can comfortably reach them. The virtual joystick and action buttons should not obscure the character, nearby interactions, or important combat telegraphs.
- On desktop, keep keyboard/mouse guidance available where useful; the reference's WASD, interaction key, ability numbers, and character-menu key are examples, not immutable bindings. Controls should remain discoverable and remappable where supported.
- Adapt composition per platform rather than shrinking a desktop screenshot into a phone viewport. Maintain readable text, clear hit targets, and the ability to see the world while acting.
- Maintain equivalent information and gameplay capability across desktop and Android, while allowing layout and control placement to adapt to each platform.

## Spatial exploration vitals

Spatial exploration now keeps a compact character panel in the header with character name/level, current and maximum health, and a text summary of active conditions. It reuses the common health-gauge primitive and shared dark theme, so the essential status remains visible without occupying a large permanent panel. This does not imply that the planned combat HUD, minimap, or ally command controls exist yet.

## Boundaries

These are information-hierarchy and visual-direction goals, not a specification for the current screen layout or a claim that every listed control already exists. Current screen behavior is described in [system documentation](../systems/); future gameplay mechanics are described in their respective design specifications. The interim World Map's implemented behavior remains documented in [World Map](../systems/world-map.md).
