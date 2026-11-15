# Pixel-Art Visual Direction and Asset Migration

> Design record migrated from [GitHub issue #229](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/229). This document defines the approved visual direction and production principles. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

Long-term art-direction foundation. This is not a single bulk art-replacement task. Product-level direction follows the canonical [The Ever-Unfolding Saga design principles](../design-principles.md). The direction below is approved visual intent, not a claim that all visuals or spatial systems are already implemented.

## Vision

Develop a coherent authored pixel-art language for a character-focused spatial RPG. Spatial gameplay should communicate movement, exploration, atmosphere, environmental interaction, and future action combat primarily through visuals and spatial presentation. Dialogue, descriptions, records, history, and contextual text remain important where they serve the experience.

The defining quality is **beauty grounded in a believable dark-fantasy world**. The visual identity is gothic and grounded, but the world must not be uniformly grim, dark, or harsh. Natural beauty, regional character, open skies, and moments of warmth are essential parts of its identity. Beauty should emerge from place, light, weather, materials, and composition rather than from ornamental excess.

Pixel art may be created collaboratively by the project owner and/or generated with assistance during development. Establish a coherent visual language before producing large quantities of final assets. Deliver it incrementally so useful improvements do not wait for a complete art set.

## Approved artistic direction

### Reference influences

The project's visual references are:

- **The Witcher 3** — environmental beauty and variety, strong regional identity, natural and atmospheric lighting, and a world that remains visually compelling beyond points of interest.
- **Kingdom Come: Deliverance** — grounded human proportions, believable equipment and materials, practical construction, historical weight, and visual clarity in interfaces and objects.
- **Dark Souls Remastered** — gothic architecture, strong silhouettes, atmospheric composition, restrained visual storytelling, and a sense of age and scale.

These are sources of principles, not templates to reproduce. TEUS must develop its own coherent identity. Do not imitate a game's exact UI, asset designs, characters, locations, or distinctive compositions.

Baldur's Gate II is not part of the approved visual reference set.

### World mood and regional beauty

- Use a grounded gothic-fantasy foundation for architecture, settlements, ruins, equipment, creatures, and threatening places.
- Do not apply the same bleakness to every region. A dangerous world can still contain peaceful, beautiful, welcoming, or awe-inspiring places.
- Let geography and local conditions determine each region's visual identity: vegetation, terrain, weather, sky, water, architecture, materials, and light should vary together.
- Natural landscapes are first-class visual subjects, not empty travel corridors between grim locations. Forests, mountains, coasts, rivers, meadows, wetlands, and other biomes should have distinct identities.
- Scandinavian nature is a welcome influence for appropriate regions: dense forests, rugged coastlines, fjords, lakes, mountains, cold clear light, and seasonal variation. It is an influence, not a requirement that the whole world resemble Scandinavia.
- Reserve visual harshness for places, conditions, and moments that justify it. Use contrast between safety and danger, shelter and exposure, abundance and desolation.

### Perspective and spatial composition

- Retain a **flexible three-quarter view** as the general spatial presentation direction.
- Do not force all environments, buildings, props, or maps into a rigid isometric grid. Perspective and composition may adapt to the scene while preserving spatial readability.
- Establish depth through coherent ground planes, overlap, scale, shadows, occlusion, and environmental layers.
- Keep movement, collision, interaction reach, and gameplay coordinates authoritative and independent of decorative perspective choices.
- Camera framing should support navigation and awareness of nearby actors, passages, threats, and points of interest. Art composition must not conceal actionable space.

### Characters and proportions

- Prefer relatively realistic human anatomy, proportions, clothing, and equipment over heavily exaggerated or chibi proportions.
- Preserve the current player sprite's general sense of scale and grounded readability. The established animated player atlas uses logical frames of 32×48 pixels; this is a current player-asset contract, not a universal size requirement for every character or object.
- Sprites should not be made arbitrarily tiny to fit more content on screen. Characters and important entities need enough pixel area for readable silhouettes, equipment, facing, and animation.
- Maintain believable relative scale between people, doors, furniture, vegetation, props, and architecture. Stylization is welcome when it improves clarity without undermining the grounded direction.
- Distinguish characters through silhouette, clothing, equipment, posture, and palette rather than relying only on small facial details.

### Palette and lighting

- Use restrained, material-led colour. Avoid both indiscriminate saturation and the assumption that dark fantasy means every surface must be nearly black.
- Dark scenes must retain enough value separation to distinguish materials, forms, depth, paths, and interactable objects.
- Prefer natural material colours and region-specific colour families: stone, soil, wood, metal, foliage, water, snow, cloth, and skin should remain distinguishable.
- Use warm light against cooler shadows where the environment supports it. Firelight, windows, sunrise, sunset, and lanterns can create warmth without turning the entire image orange.
- Let daylight, overcast skies, moonlight, fog, snow, canopy cover, and local weather produce different lighting conditions. Do not use one global darkness treatment as a substitute for atmosphere.
- Interactive elements must be readable through shape, placement, contrast, contextual animation, or restrained accents. Avoid persistent artificial outlines or bright highlights that make the world resemble a generic game UI.
- Colour must support readability but should not be the only way to communicate gameplay state.

Exact master-palette values, colour-count limits, outline policy, and per-biome palettes remain open for later asset tests. The existing interface onyx colour `#070707` remains an interface surface token, not a mandate to paint the world with near-black.

### Pixel craft, detail, and animation

- Prioritize clear silhouettes and deliberate clusters over noisy single-pixel texture.
- Use detail to describe form, material, wear, and lighting. Avoid detail that disappears at runtime scale or competes with the subject's silhouette.
- Keep pixel density and edge treatment coherent between assets that appear together. Assets from different sources must be adapted to the shared visual language rather than placed side by side unchanged.
- Animation should communicate direction, weight, intent, and state. Movement must remain legible at normal gameplay scale and should not rely on excessive frame count or visual effects.
- Keep frame layouts, clip definitions, and asset metadata explicit and data-driven. Preserve stable asset contracts where they already exist; do not silently change the player atlas contract as part of unrelated art work.

## Asset scope and integration

Pixel art should eventually cover relevant categories such as:

- Player characters, NPCs, portraits, and enemies.
- Local environments, architecture, vegetation, terrain, weather, and environmental effects.
- Items, equipment, resources, trophies, and important world objects.
- Tiles and environmental elements for future tile/grid exploration maps.
- Combat telegraphs, contextual effects, and other gameplay-significant visuals.
- UI icons and accents where they improve recognition or hierarchy.

Keep gameplay rules and world identity renderer-neutral. Asset references should be data-driven so authored content and mods can select artwork without coupling gameplay logic to a particular rendering representation. Where practical, missing or unfinished art should have a deliberate fallback rather than breaking content.

## Scale, perspective, and production constraints

The visual standard must work in desktop and Android landscape presentation. Design assets at a useful native pixel scale, then scale them consistently; do not rely on arbitrary per-asset resizing to make inconsistent artwork fit.

The following technical details are intentionally **not locked by this direction document** and should be decided through representative asset tests:

- A universal tile size or world-unit-to-pixel ratio.
- A single native render resolution or fixed camera zoom.
- One universal sprite dimension for all characters, creatures, and props.
- Exact palette indices, colour-count limits, and outline rules.
- A mandatory art toolchain or image-editor format.
- A single perspective projection for every scene.

The existing 32×48 player frame and 6×8 directional idle/walk atlas remain the current player-animation contract. New asset work should respect it unless a separately scoped change establishes and migrates a better contract.

## Migration approach

- Keep visual assets optional during the transition where practical.
- Replace ASCII pieces incrementally rather than requiring a complete art set before visual improvements can ship.
- Preserve text-only fallbacks where useful, especially for content without a finished asset.
- Keep asset references data-driven and compatible with authored content and mods.
- Review new assets in their actual gameplay context and at their intended runtime scale on both desktop and Android.
- Prefer coherent small sets of related assets over isolated high-detail assets that establish incompatible styles.

## Relationship to future world exploration

The art direction must support authored tile-based environments, moving entities, enemies, investigation targets, and future combat encounters without requiring a separate style for each system. As world topology and regional generation mature, visual variation should be driven by geography, climate, settlement identity, local resources, and authored location identity—not by random palette swaps alone.

## Non-goals

- No requirement to replace every current ASCII asset immediately.
- No implementation of the future open-world system.
- No fixed art toolchain requirement.
- No requirement that every screen become fully graphical; text-based dialogue, records, history, menus, and contextual interactions remain appropriate where they serve the experience.
- No direct reproduction of the referenced games' assets, UI, characters, or compositions.
- Intended platform scope is desktop and Android. iOS is not a project target.
