# Bundled spatial exploration art

These are first-pass original pixel-art assets packaged as protected base-game defaults. They are loaded at runtime through the same mod-aware asset resolver used for UI icons and other mod art.

- `terrain_floor.png`: worn wooden floor.
- `terrain_ash.png`: ash-covered ground.
- `terrain_rubble.png`: broken stone ground.
- `terrain_wall_top.png`: weathered masonry tops.
- `wall_face_left.png` and `wall_face_right.png`: textured isometric wall faces.
- `player_oathbound.png`: static fallback player character sprite.
- `player_oathbound_animated.png`: transparent 192×384 directional animation atlas (6 columns × 8 rows of 32×48 frames), with four idle frames and six walking frames per direction.
- `player_oathbound_animated.json`: mod-aware animation manifest defining frame/atlas dimensions and the row, frame range, duration, and looping behavior for each directional idle/walk clip.
- `passage.png`: physical edge-passage marker.
- `prop_pillar.png`: small stone pillar; 32×40 pixels, one-cell collision footprint.
- `prop_table.png`: weathered wooden table; 72×48 pixels, two-cell collision footprint.

The current art uses nearest-neighbor sampling. The static player fallback remains 16×24; the animated player contract uses 32×48 frames to provide room for a readable hooded adventurer, clothing, boots, and a sheathed sword. Terrain, wall, and prop dimensions continue to follow the existing 48×24 floor diamonds, 24×36 wall faces, and footprint/depth conventions.

A mod can replace any of these defaults by providing the same relative asset path under its own `assets/spatial/` directory. Highest mod priority wins.

Copyright (c) 2026 right-ward. Licensed under CC BY-NC-ND 4.0, as documented in `LICENSE-CONTENT`.


The animated player sheet uses the JSON manifest at `spatial/player_oathbound_animated.json`. The base-game grid is 32×48 frames in a 6-column × 8-row atlas (192×384 total). Rows are north idle/walk, east idle/walk, south idle/walk, and west idle/walk. Idle clips occupy columns 0–3 and loop at 250 ms per frame; the last two cells in each idle row remain transparent. Walking uses all six columns and loops at 120 ms per frame. Feet must remain aligned to a common ground baseline in every frame. The runtime validates the manifest and PNG dimensions and falls back to the existing static player image if either optional animation asset is missing or invalid. Mods can override the PNG and JSON independently through the same relative paths. The bundled atlas uses corrected north-facing artwork in rows 0–1 and south-facing artwork in rows 4–5; the source magenta background is transparent, and the finished sheet was downsampled with nearest-neighbor sampling.
