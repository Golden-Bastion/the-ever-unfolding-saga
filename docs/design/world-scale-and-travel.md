# World Scale and Travel

## Design requirement

The world must feel large enough to support meaningful journeys. Local maps, locations, regions, and the world map are different spatial scales; they must not collapse into small interchangeable screens.

An ordinary outdoor area should take several minutes to cross on foot, not a few seconds. A region must represent a substantially larger area than one local map and contain room for multiple distinct places, routes, and environmental transitions. Journeys between regions should communicate real distance and terrain rather than behaving like adjacent menu selections.

This is a scale requirement, not a request to make every interior enormous. A shrine chamber, house, cave room, or small shop can remain compact when its fiction calls for it. Outdoor wilderness, settlement districts, ruins, and other substantial locations need enough space to explore and contain multiple points of interest.

## Movement and time

- Preserve responsive character movement. Do not make the player artificially slow just to make a small map take longer to cross.
- Let travel duration emerge from actual distance, terrain, route quality, weather, encumbrance, character capability, and meaningful interruptions where those systems exist.
- Keep physical travel and known-route fast travel complementary. Fast travel should represent a journey the character could actually undertake, not erase the geography.
- Use local-map transitions, connected subareas, or streamed chunks when a location is too large to represent efficiently as one fully loaded tile grid.
- Maintain continuity across chunk and region boundaries. Roads, rivers, mountain ranges, coastlines, and cave passages must not reset or terminate merely because a rendering or generation chunk ends.

## Current implementation baseline

New outdoor local maps now default to 64×48 tiles, up from the prototype's 24×16. Compact interiors remain smaller where appropriate; the authored shrine chamber uses 32×24. World Lab exposes 64×48, 80×56, and 96×64 presets. The local-map generator version advances for the new dimensions and density, while saved maps retain their established dimensions, terrain, passages, actors, and discoveries.

This is an initial increase, not the final scale target. The current renderer still creates a complete local-map scene rather than streaming a large world in chunks, and the current 64×48 dimensions do not by themselves guarantee the intended multi-minute outdoor traversal. Performance and camera behavior on Android must be measured before increasing the fully rendered tile budget further.

## Scale hierarchy

1. **Local map:** a traversable piece of a place, such as a courtyard, street district, wilderness tract, ruin complex, or cave network. It may contain several rooms and points of interest.
2. **Location:** a named destination that may eventually comprise multiple connected local areas, entrances, districts, floors, or subregions. A location is not synonymous with one small room-sized map.
3. **Region:** a broad geographic area containing multiple locations and long-distance routes. Its macro-geography should influence the character of all its constituent places.
4. **World:** a continuous-feeling network of regions with coherent large-scale landforms, watersheds, coastlines, settlements, and travel corridors.

The exact physical units, tile-to-distance conversion, and final target dimensions remain implementation decisions. They should be calibrated together using representative outdoor, settlement, interior, and underground examples rather than selected independently.

## Implementation constraints

- Preserve renderer-neutral world and geography data. Rendering chunks must not become the authority for terrain, route connectivity, or persistence.
- Keep stable location, route, passage, and discovery identities across resizing or streaming changes.
- Do not regenerate or resize a player's established local map merely because defaults change. Any future migration must preserve player-authored terrain changes, discoveries, actors, props, and passage positions.
- Design for desktop and Android together. Large worlds should use bounded active rendering, deterministic generation, and sensible memory budgets rather than spawning every tile in a region at once.
- Use the macro-geography lattice for regional continuity, but do not assume that one macro sample maps directly to one local-map tile.
