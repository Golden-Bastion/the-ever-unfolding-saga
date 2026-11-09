# Celestial world art

`sun-moon.png` is a transparent landscape artwork atlas shared by the compatibility Gameplay sky and the compact spatial time-of-day indicator. The sun occupies the left panel and the moon occupies the right panel. The renderers crop each panel into a sky sprite; neither body is a UI icon. Spatial exploration uses a smaller presentation of the same art rather than placing an open sky above the angled local map.

The bundled image is loaded through the mod-aware asset resolver. A mod can replace it by providing `assets/world/celestial/sun-moon.png` in its package. Keep both subjects in their existing left/right regions and preserve transparent space around their artwork so the sky background remains visible.
