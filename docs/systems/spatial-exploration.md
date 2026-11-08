# Spatial Exploration

## Responsibility boundaries

`spatial.rs` owns renderer-neutral local-map data, terrain walkability and prop footprints, stable point-of-interest identity, deterministic generation, content-selected built-in room layouts, map exits, discovery state, persistent `SpatialEntityPlacement` records for world-owned NPCs/corpses, and the character's logical exploration session. It must not depend on Bevy rendering or UI types. The new actor-placement field has a serde default so maps from older saves remain loadable.

`bevy_spatial.rs` adapts that model to the Bevy frontend. It owns angled projection, terrain, prop and actor-marker rendering, depth ordering, movement input from keyboard and touch controls, collision checks, camera follow, nearby-target resolution, and synchronization with the active game state. Actor placements are added deterministically on map entry and only use reachable walkable cells. The saved world remains authoritative for persistent maps; the active character owns the exact continuous position. `bevy_interactions.rs` renders the corresponding dialogue and remains overlays without replacing the spatial scene.

## Coordinate conventions

- Grid coordinates are continuous during movement. A tile's center is represented by `(x + 0.5, y + 0.5)`.
- Grid/world coordinates are authoritative for movement, collision, discovery, and persistence. Projected screen-space coordinates are presentation-only.
- The fixed three-quarter projection maps the two grid axes onto opposing diagonal screen directions. Input directions are converted through the inverse projection so keyboard and touch share the same movement rules.
- Sprite transforms are positioned from the ground-contact point plus a visual vertical offset. Actor depth follows the continuous ground position instead of snapping to a whole tile.
- Floor, ash, and rubble diamonds are placed on a dedicated background depth layer so ground surfaces cannot draw over actors. Wall tops are sorted from their ground-contact line; wall faces are sorted from their front/base edge. This lets the wall occlude a character behind it while a character moving beyond the wall's base renders in front. When an actor and wall top share the same ground depth, the actor wins the tie so a wall's entire top diamond does not hide a character in an adjacent lateral position.
- Wall tops extend upward beyond the floor diamond. Camera bounds account for that asymmetric vertical extent rather than expanding both vertical edges equally.

## Movement and interactions

Movement uses a small ground-space collision footprint and resolves the two grid axes separately, allowing movement to slide along a blocked edge. Walls and static prop footprints block movement; floor, ash, and rubble remain walkable. Pillars occupy one cell and tables occupy two adjacent cells in one grid row. Prop footprints are stored with the persistent local map, and the same walkability query is used for interaction line-of-sight checks.

Local spatial exploration is entered through Explore at both safe and dangerous locations. The generic generated "Unsettling Remains" discovery POI is removed when entering a safe location so that prototype content is not falsely presented as a real corpse; other authored POIs are preserved. NPCs at the location and corpses with searchable inventory receive stable positions from reachable walkable cells. Existing positions are never rerolled on re-entry, and stale placements are harmless because rendering and targeting verify the actor's current location/state. Newly assigned positions avoid the entry, POIs, exits, props, walls, and other actor placements.

Contextual interaction discovery returns every eligible nearby NPC, corpse, POI, and edge passage as a renderer-neutral action/target pair. Each pair retains stable target identity, including individual POI and passage IDs; candidates are ordered by projected distance with deterministic tie-breaking. The Interact button and E/Enter input open a radial wheel anchored to the player's projected screen position. Every eligible pair appears as its own action entry, ordered by nearest projected distance; labels include the target where needed. Arrow keys or WASD move the selection toward the closest radial entry, Enter/E confirms, and Escape/Backspace closes. Android uses the same touchable entries and a central close control, with the movement joystick suspended while the wheel is open. The wheel animates into and out of view, keeps the spatial scene visible, and never replaces the renderer-neutral action/target identity with a display index. The selected pair is re-resolved against current eligibility before execution, so range, actor existence, inventory, and clear-approach rules remain authoritative. Future combat must not treat this interaction range or line of sight as a substitute for attack collision or target eligibility.

Talking transitions from the spatial view into a dedicated close-up dialogue scene while preserving the active spatial session underneath. The scene places the player and NPC together, composes each portrait from the deterministic, renderer-neutral portrait layers, and presents memory, availability, responses, and choices in a dedicated dialogue area. Portrait PNGs are resolved through the same mod-aware asset loader as other game art; layers retain their declared draw order, missing parts are omitted, and the legacy NPC portrait text is shown only when no usable image layer can be loaded. A shared scene-root transition animates entry and exit. Movement remains suspended until the exit transition completes, then resumes at the exact previous exploration position. Searching a corpse retains its contextual inspection/recovery overlay and existing loot, hidden-discovery, and note presentation. Dialogue, time advancement, quest effects, and inventory mutation continue to use the authoritative game rules rather than renderer-owned logic.

## Authored room layouts

Campaign location content may specify the optional `spatial_layout` identifier. The first built-in template, `shrine_chamber`, is assigned to Old Shrine and creates a compact rectangular chamber with intersecting masonry walls, one open doorway, a pillar, and a table. The room is renderer-neutral: terrain cells and prop footprints feed the same movement, collision, and line-of-sight rules as procedural geometry.

Layout selection affects newly generated maps only. Once saved, a local map's terrain and prop state remain authoritative; existing saved maps are not silently regenerated to apply a new layout. Older content without `spatial_layout` remains valid, and unrecognized layout identifiers fall back to the normal procedural map. This is a first authored template, not arbitrary mod-defined room geometry or a complete room/door graph.

## Physical map exits

A local map's `exits` are derived from its world location's existing connected-location IDs. Each passage stores a stable target location ID rather than relying on a name or checking the destination's danger flag. Passage placement is deterministic, passages are positioned on edge tiles, and generated walkable corridors connect them to the map's entry area.

A contextual passage action appears only when the character is within interaction range and has a clear approach. Pressing the contextual interaction key uses the nearest available passage, or examines the point of interest when no passage is in range. Directly activating a passage action applies the same range and approach validation.

For now, using a physical passage returns to the normal gameplay dashboard, matching the existing `Leave` action. It does not yet transfer the character to the destination location or keep the player continuously inside spatial mode. The destination ID is nevertheless retained in persistent map state so passage use can later enter a supported destination, with a suitable fallback for locations that do not yet have spatial maps.

Older saved local maps without `exits`, `props`, or `interactables` fields remain deserializable. Missing/empty exit data is initialized from the current world location's connections; prop-less saved maps retain their existing layout rather than being regenerated; missing actor placements are added on entry without regenerating the map.

## Visual assets

The first spatial pixel-art set is packaged as protected base-game assets and includes wood-floor, ash, rubble, and wall-top tiles, separate left/right wall-face sprites, a character sprite, a passage marker, and dedicated pillar/table props. These PNGs are loaded at runtime through the mod-aware asset resolver; they are not a writable global asset collection. A mod can replace a base image by providing the same relative path under its own `assets/` directory. The enabled mod with the highest priority wins, with stable mod ID as the deterministic tie-break. The wood floor is a first-pass treatment for the Old Shrine prototype, not a claim that every location should use wood.

The interface uses named PNG icons from the bundled `icons/ui/` collection for actions, menu controls, conditions, records, and combat choices. The first NPC/corpse world markers reuse the existing character/remains icons as tinted sprites; dedicated actor art and richer NPC silhouettes remain future asset work. The world clock uses a separate large celestial-art atlas at `world/celestial/sun-moon.png`; the left panel contains the sun and the right panel contains the moon. The gameplay renderer crops these regions into large sky sprites, keeping them out of the 32×32 icon collection. Both icon and world-art paths use the mod-aware asset resolver, so a mod can replace either at the same relative path. Text separators and ordinary labels use simple ASCII punctuation instead of decorative glyphs that may be missing from the default font. Mod-owned assets live under the installed mod's `assets/` directory and can be referenced explicitly with `mod:<manifest-id>/<relative-path>.png`.

## Spatial HUD

The local exploration screen keeps a compact character-status panel in the header. It reads the active character's name, level, current/maximum health, and current conditions directly from authoritative game state. The shared health-gauge primitive supplies both the blood-fill indicator and numeric values. The conditions line names active conditions and explicitly says when none are present. The location title and contextual feedback remain separate from this panel, and the Leave/action controls and Android-only virtual joystick retain their existing roles.

This is the first HUD layer, not the complete reference layout. Combat telegraphs, enemy vitality, ability controls, a minimap/compass, and local ally orders are not implemented by this change.

## Camera behavior

The camera uses a viewport-scaled dead zone: ordinary small player movements do not move the camera, while crossing the dead-zone edge shifts the target by only the excess distance. A smoothed follow approaches that target, and map bounds keep the viewport inside the projected map where possible. Maps smaller than the viewport are centered.

This is not yet full room-aware camera behavior. Local maps currently have no explicit room graph, per-room bounds, or door-to-room transitions. Physical edge passages are world-exit markers, not an interior room system. Room-aware camera behavior should only be introduced alongside a defined room/door model, then tested for smooth transitions and for avoiding views outside playable spaces.

## Validation status and next gates

Automated unit tests cover deterministic map/prop/actor placement, reachable actor positions, authored chamber geometry and doorway access, room/exit reachability, map serialization and backward-compatible defaults, reachable physical exits with prop-free corridors, projection offsets, inverse input mapping, basic continuous movement, terrain/prop-footprint collision, interaction range and prop-blocked line of sight, contextual target priority, spatial-overlay state separation, dashboard action routing, dead-zone behavior, map-bound camera constraints, wall-top/prop depth ordering, selected-folder PNG dimensions, and UI icon fallbacks.

These tests are source-level coverage. They do not establish successful end-to-end play on desktop or Android; interactive testing and on-device verification are intentionally left to the repository owner.

The first representative authored room now exists as the Old Shrine chamber template. The remaining validation gate is interactive desktop and Android landscape playtesting: move around both sides of the pillar and table, cross the doorway, and inspect wall-top/face ordering and camera framing. Automated tests establish model reachability and depth calculations but do not replace visual playtesting. Full room/door topology, arbitrary authoring tools, and additional layouts remain future work.


## Contextual action discovery

Interaction discovery now produces a complete, ordered list of renderer-neutral action/target pairs for all currently eligible nearby NPCs, searchable remains, POIs, and passages. Each pair retains its target's stable identity, including individual POI and passage IDs. The Interact key/button opens the player-anchored radial wheel, which exposes the full ordered list. Directional keyboard input and touch buttons select the same stable action/target entries; confirmation revalidates current eligibility before dispatching the existing action. Reusable scene transitions, the close-up two-character dialogue presentation, and image composition for generated portrait layers remain future work under #405. Eligibility still uses the current proximity, clear-approach, actor-existence, and searchable-inventory checks.
