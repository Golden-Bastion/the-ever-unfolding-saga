# Bundled spatial exploration art

These are first-pass original pixel-art assets packaged as protected base-game defaults. They are loaded at runtime through the same mod-aware asset resolver used for UI icons and other mod art.

- `terrain_floor.png`: worn wooden floor.
- `terrain_ash.png`: ash-covered ground.
- `terrain_rubble.png`: broken stone ground.
- `terrain_wall_top.png`: weathered masonry tops.
- `wall_face_left.png` and `wall_face_right.png`: textured isometric wall faces.
- `player_oathbound.png`: static fallback player character sprite.
- `player_oathbound_animated.png`: intentionally empty placeholder for the future 80×96 directional animation sheet. Keep this path stable when adding the final PNG.
- `passage.png`: physical edge-passage marker.
- `prop_pillar.png`: small stone pillar; 32×40 pixels, one-cell collision footprint.
- `prop_table.png`: weathered wooden table; 72×48 pixels, two-cell collision footprint.

The current art uses nearest-neighbor sampling. Its dimensions are matched to the prototype's 48×24 floor diamonds, 16×24 character sprite, 24×36 wall faces, and the footprint/depth conventions for the pillar and table. These are a starting point, not final art direction.

A mod can replace any of these defaults by providing the same relative asset path under its own `assets/spatial/` directory. Highest mod priority wins.

Copyright (c) 2026 right-ward. Licensed under CC BY-NC-ND 4.0, as documented in `LICENSE-CONTENT`.


The animated player sheet uses 16×24 frames in a row-major 5-column × 4-row grid. Rows are north, east, south, and west; each row starts with one idle frame followed by four walk frames. The empty PNG placeholder is not valid artwork: the runtime rejects it and keeps using the static player sprite until a valid sheet replaces it. Mods may override the animated sheet at the same relative path.
