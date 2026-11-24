# Bundled spatial exploration art

These are first-pass original pixel-art assets packaged as protected base-game defaults. They are loaded at runtime through the same mod-aware asset resolver used for UI icons and other mod art.

- `terrain_floor.png`: worn wooden floor.
- `terrain_ash.png`: ash-covered ground.
- `terrain_rubble.png`: broken stone ground; retained as the generic underground fallback when the geology atlas is unavailable.
- `terrain_wall_top.png`: weathered masonry tops used outside caves and as the fallback for missing geology art.
- `geology_tiles.png`: 144×48 six-cell atlas (48×24 per cell), with pale-granite, veil-slate, and cinder-basalt floor tiles in the first row and their raised-rock ledge tiles in the second row. The spatial renderer selects atlas cells from the sampled underground material; it retains nearest-neighbor sampling and discovered/undiscovered shading.
- `wall_face_left.png` and `wall_face_right.png`: textured isometric wall faces; remain tinted from the sampled underground geological material.
- `player_oathbound.png`: static fallback player character sprite.
- `player_oathbound_animated.png`: transparent 192×384 directional animation atlas (6 columns × 8 rows of 32×48 frames), with four idle frames and six walking frames per direction.
- `player_oathbound_animated.json`: mod-aware animation manifest defining frame/atlas dimensions and the row, frame range, duration, and looping behavior for each directional idle/walk clip.
- `passage.png`: physical edge-passage marker.
- `prop_pillar.png`: small stone pillar; 32×40 pixels, one-cell collision footprint.
- `prop_table.png`: weathered wooden table; 72×48 pixels, two-cell collision footprint.
- `actors/npc.png`: neutral hooded traveller; transparent 32×48 upright sprite, displayed at 17×22 logical pixels.
- `actors/hostile.png`: armed old-road marauder; transparent 32×48 upright sprite, displayed at 17×22 logical pixels.
- `actors/corpse.png`: prone, searchable remains; transparent 48×32 landscape sprite, displayed at 21×13 logical pixels.

Ashen Gate currently uses the terrain and masonry set to represent its broken tower bases and gate wall. Hollow Market reuses the table sprite for abandoned stall counters and the pillar sprite for unlit lantern posts. These are functional first-pass placements, not dedicated final market/gate artwork; the layout definitions remain renderer-neutral so their assets can be replaced without changing collision or room topology.

Underground cave presentation uses the dedicated geology tile atlas for walkable rock floors and raised ledge tops. Existing isometric wall-face sprites remain in use and receive material-specific tinting; sparse fracture accents remain renderer-generated. Larger geological formation silhouettes and varied cave-wall structures have not yet been added.

The current art uses nearest-neighbor sampling. The static player fallback remains 16×24; the animated player contract uses 32×48 frames to provide room for a readable hooded adventurer, clothing, boots, and a sheathed sword. Terrain, wall, and prop dimensions continue to follow the existing 48×24 floor diamonds, 24×36 wall faces, and footprint/depth conventions.

A mod can replace any of these defaults by providing the same relative asset path under its own `assets/spatial/` directory. Highest mod priority wins.

Copyright (c) 2026 right-ward. Licensed under CC BY-NC-ND 4.0, as documented in `LICENSE-CONTENT`.

## Actor artwork provenance

The three role-specific actor sprites were generated as original TEUS artwork on 20 November 2026 using OpenAI image generation from the project's written art brief. The generated transparent artwork was separated into individual files, trimmed, resampled to the dimensions above, and reduced to a restrained pixel palette. No third-party game art or external character image was supplied as a source reference. These files are part of the project's bundled content and follow the same CC BY-NC-ND 4.0 content license.


The animated player sheet uses the JSON manifest at `spatial/player_oathbound_animated.json`. The base-game grid is 32×48 frames in a 6-column × 8-row atlas (192×384 total). Rows are north idle/walk, east idle/walk, south idle/walk, and west idle/walk. Idle clips occupy columns 0–3 and loop at 250 ms per frame; the last two cells in each idle row remain transparent. Walking uses all six columns and loops at 120 ms per frame. Feet must remain aligned to a common ground baseline in every frame. The runtime validates the manifest and PNG dimensions and falls back to the existing static player image if either optional animation asset is missing or invalid. Mods can override the PNG and JSON independently through the same relative paths. The bundled atlas uses corrected north-facing artwork in rows 0–1 and south-facing artwork in rows 4–5; the source magenta background is transparent, and the finished sheet was downsampled with nearest-neighbor sampling.
