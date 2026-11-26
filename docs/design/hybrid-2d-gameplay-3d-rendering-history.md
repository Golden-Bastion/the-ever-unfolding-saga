# Historical Proposal: 2D Gameplay and 3D Rendering

> Status: superseded historical proposal. This document records an earlier direction in which gameplay remained 2D and 3D was used only for presentation. It is retained to explain the evolution of the design, not as an active specification. The approved target is now the authoritative 3D spatial world described in [3D Spatial World and Presentation Architecture](3d-spatial-world-and-presentation.md). Prototype implementation and validation remain in the private [TEUS Lab repository](https://github.com/Golden-Bastion/teus-lab).

## Separate implementation and tracking

The standalone experiment and all active work around it—including source code, build configuration, assets, platform workflows, test cases, validation results, issues, and any recommendation about production integration—are maintained exclusively in the private [TEUS Lab repository](https://github.com/Golden-Bastion/teus-lab).

TEUS Lab is the source of truth for the experiment's current state and next steps. This document intentionally does not duplicate that task tracking or attempt to keep experimental implementation details synchronized. The production game remains on its existing 2D renderer; no production renderer migration has been approved.

## Purpose

Evolve spatial presentation away from increasingly bespoke 2D draw-order rules without changing the game's underlying 2D gameplay model or its pixel-art identity.

The target is a **2D gameplay world presented through a 3D rendering scene**. The game should continue to reason about movement, collision, navigation, map generation, interactions, and persistence in the existing two-dimensional grid/world coordinate system. A 3D scene should provide a more general rendering space for positioning sprite-based objects, resolving visibility, and eventually supporting coherent lighting and shadows.

This is not a proposal to turn TEUS into a conventional 3D game. It does not require 3D character or environment models, freely variable gameplay elevation, or a third gameplay movement axis.

## Goals

- Preserve the flexible three-quarter presentation and the established pixel-art visual direction.
- Keep gameplay and persistent world data renderer-neutral and two-dimensional.
- Replace fragile, object-specific draw-order workarounds with a consistent spatial rendering model where practical.
- Give sprites and environmental elements stable ground-contact anchors and explicit visual bounds.
- Make it possible to add coherent world lighting and shadows without requiring every asset to become a 3D model.
- Keep desktop and Android presentation behavior and gameplay capability equivalent.
- Introduce the architecture incrementally, with a working comparison against the current renderer before committing to broad migration.

## Non-goals

- A fully 3D simulation or third axis of player movement.
- 3D character rigs, skeletal animation, or mandatory 3D environment assets.
- Replacing tile-based generation, grid navigation, existing collision rules, or stable interaction/target identities.
- Replacing the current art direction with perspective-heavy or realistic rendering.
- Adding lighting effects merely for visual novelty when they reduce pixel-art readability or mobile performance.
- Rewriting unrelated UI, gameplay, save, or content systems as part of the renderer change.

## Current foundation and motivation

The current spatial model already has a useful separation:

- `spatial.rs` owns renderer-neutral local-map geometry, terrain, prop footprints, actor placements, discovery state, and deterministic generation.
- The character's continuous exploration position is maintained as gameplay/session state.
- `interaction_actions.rs` owns eligible contextual action/target pairs and authoritative range/approach validation.
- `bevy_spatial.rs` adapts that model to Bevy and currently owns projection, sprite transforms, camera follow, rendering, and depth ordering.

The current renderer uses a fixed three-quarter projection and explicit depth constants/biases for terrain, wall tops, wall faces, and actors. It sorts or separates some geometry according to ground-contact positions and uses tie-breaking rules to handle edge cases. Those rules are understandable for the current prototype, but each new prop shape, overhang, tall sprite, or environmental feature can introduce another special case.

The objective is not to label the existing implementation as inherently wrong. A 2D renderer can support sophisticated sorting. The question is whether a 3D scene and depth buffer provide a simpler, more extensible foundation for the visual relationships TEUS intends to support.

## Core architectural decision

Keep two distinct coordinate responsibilities.

### Gameplay space: authoritative 2D

The world model remains two-dimensional:

- Local-map grid coordinates and continuous character positions remain authoritative for movement.
- Terrain walkability, prop footprints, collision, navigation, discovery, interaction range, line of sight, and map exits remain gameplay rules.
- Persistent entity and prop placements remain expressed in renderer-neutral world/grid coordinates.
- Gameplay code must not inspect camera transforms, render depth, sprite bounds, lighting state, or Bevy entities to decide outcomes.

A renderer may derive a 3D transform from a gameplay position, but it must never write a visual depth or elevation adjustment back into authoritative gameplay state.

### Presentation space: 3D scene

The Bevy presentation adapter maps gameplay positions into a 3D scene:

- The two gameplay axes locate an object's ground position on the scene's ground plane.
- The camera uses an orthographic projection and a controlled three-quarter orientation to preserve the established composition.
- Sprite assets remain 2D textures on appropriately oriented planes or equivalent sprite-rendering primitives.
- A sprite's ground-contact anchor determines where it meets the gameplay ground plane. The visible image may extend above that point without changing its collision footprint.
- Camera-space depth and the depth buffer establish visibility between world elements where their geometry and render materials support it.
- Optional visual elevation is presentation metadata only. It may be used for an asset's ground offset, shadow casting, or a deliberately elevated visual, but it does not create a gameplay movement axis.

The exact mapping of the two gameplay axes onto the renderer's ground plane, sprite-plane orientation, and camera transform must be proven in a technical prototype. The implementation should choose one convention and document it rather than scattering coordinate conversions through rendering systems.

## Object presentation contract

World objects should have explicit presentation metadata instead of relying on their image dimensions to imply gameplay or depth behavior.

At minimum, the rendering boundary should be able to obtain:

- **Ground anchor:** the point on the sprite that corresponds to its placement on the ground (for example, a character's feet or a tree's trunk base).
- **Visual bounds:** the sprite's rendered extent, independent of its gameplay footprint.
- **Gameplay footprint reference:** the existing 2D occupied cells or collision shape, owned by the world model.
- **Visual depth/elevation offset:** an optional presentation-only adjustment for composition, never an independent gameplay coordinate.
- **Material/render mode:** the information needed to select opaque, cutout, or blended rendering behavior.
- **Shadow participation:** whether and how the visual contributes to a shadow representation.

The same conceptual contract should work for characters, trees, walls, pillars, tables, doors, furniture, pickups, and other props. It must not require each new object kind to invent its own global sorting formula.

For objects whose visible sprite overhangs their footprint, visibility should be based on their spatial placement and render geometry, not on treating every opaque pixel as a separate gameplay obstacle. Occlusion and collision are separate concerns.

## Depth, occlusion, and sprite composition

The target is to let scene depth resolve ordinary front/behind relationships instead of assigning hand-maintained depth constants to each object category.

Important constraints:

- Depth must correspond consistently to the object's ground placement and the chosen camera orientation.
- Ground surfaces must not accidentally cover actors or props because of arbitrary sprite-plane placement.
- Large sprites and overhanging canopies must retain a useful ground anchor; their image center is not their logical position.
- Walls and other partially occluding objects may need separate visual pieces (for example, top and face) or carefully designed geometry. A single flat sprite cannot always express the correct visibility relationship for every possible camera position.
- Alpha-cutout pixel art should be evaluated separately from blended transparency. Blended sprites can introduce depth-sorting limitations even in a 3D renderer.
- Stable tie-breaking may still be needed for objects sharing the same ground depth. The goal is to reduce bespoke rules, not promise that all ordering problems disappear automatically.

The prototype must include at least one actor moving behind and in front of a tree-like overhanging sprite, a wall with a distinct top/face relationship, and two actors crossing paths. These cases should be compared against the current renderer.

## Lighting and shadows

A 3D scene creates opportunities for coherent lighting, but flat pixel-art sprites do not automatically behave like lit 3D surfaces. Lighting must be an explicit asset/material pipeline decision.

The architecture should allow these techniques to be evaluated independently:

1. **Ground lighting:** use 3D lights and/or a ground shader for light pools, directional illumination, and environmental variation.
2. **Sprite lighting:** use a compatible material/shader, authored normal information, or another controlled technique when a sprite should respond to light. Do not assume ordinary unlit sprite materials will react correctly to scene lights.
3. **Shadows:** use alpha-aware shadow casting or deliberate shadow sprites/decals. Shadow shape, softness, and pixel scale must remain consistent with the art direction.
4. **Ambient treatment:** use restrained ambient/environmental lighting to preserve readability in dark interiors and at night.
5. **Pixel stability:** lighting and shadow sampling must not introduce shimmering, blurry edges, inconsistent pixel sizes, or uncontrolled colour gradients.

Normal maps are an optional enhancement, not a prerequisite for adopting a 3D scene. They should be introduced only if tests show that they improve lighting without undermining the authored pixel-art look.

Dynamic shadows should be selective. Static environmental objects may use baked/authored shadows or inexpensive shadow representations where that produces a better result. Mobile performance must be measured before enabling expensive real-time shadowing broadly.

## Pixel-art rendering constraints

The renderer must preserve the art direction in [Pixel-art visual direction](pixel-art-direction.md):

- Keep the flexible three-quarter view; do not force a rigid isometric grid.
- Preserve intentional pixel clusters, silhouettes, outlines, and authored palette relationships.
- Use nearest-neighbour texture sampling where appropriate.
- Avoid unintended filtering, subpixel jitter, texture bleeding, and inconsistent sprite scale.
- Evaluate camera and transform snapping carefully. Snapping must not make movement or animation visibly stutter.
- Lighting should support form, atmosphere, time of day, and readability rather than recolour every sprite indiscriminately.
- Any post-processing or pixelation pass must be evaluated on actual game assets and both target platform classes, not only on a test scene.

The existing 2D asset pipeline should remain usable. A 3D renderer must not force artists or content authors to convert ordinary sprite assets into 3D models.

## Bevy integration direction

The current Bevy dependency enables the 2D rendering feature set. A 3D-backed prototype will need to evaluate the corresponding Bevy 0.20 camera, mesh/plane, material, light, shadow, and render features, including the dependency and platform implications of enabling them.

The technical spike should answer:

- Can the chosen Bevy 0.20 render path display existing sprite-sheet animations on scene-oriented planes while retaining atlas animation and mod-aware asset loading?
- Can the world camera use the intended orthographic three-quarter view without breaking the separate full-window UI camera, viewport, HUD, overlays, or screen transitions?
- Can the renderer use a depth buffer for ground-anchored sprite visibility with stable results?
- Which render materials support alpha cutouts, lighting, and shadows, and what custom material work is required?
- Can the existing asset references and loading flow remain intact?
- What is the smallest Bevy feature set that supports the chosen path on desktop and Android?
- Does enabling the 3D render path introduce unacceptable binary size, startup, memory, GPU, or device-compatibility costs?

Keep the 3D implementation behind the Bevy presentation boundary. Renderer-neutral map and interaction modules must not gain Bevy dependencies.

## Camera and movement

The gameplay coordinate system and movement rules do not change as a consequence of the camera implementation.

The camera remains orthographic, with a flexible three-quarter composition. It should continue to follow the player smoothly and respect the existing world viewport beneath the spatial HUD. Camera orientation and projection must not alter movement speed, collision distance, interaction range, or navigation direction.

The prototype must verify that:

- Keyboard and Android joystick input still map to the same two gameplay axes.
- Character movement remains continuous and collision-checked in gameplay space.
- Camera follow and viewport resizing behave consistently across desktop and Android.
- UI remains in screen space and is not affected by world lighting, depth, or camera movement.
- Existing contextual prompts, interaction wheel, dialogue overlays, and pause/navigation flows remain usable.

## Production integration boundary

The hybrid approach is a possible future presentation direction, not a selected production architecture. Prototype work and feasibility investigation belong in TEUS Lab, not in the TEUS application repository.

A recommendation to change the production renderer must return to TEUS as a separate, owner-approved task. That proposal should include the lab's relevant evidence, a clear proceed/defer/reject recommendation, the expected benefits and costs, platform implications, and a bounded integration plan. The TEUS roadmap should only gain integration work after that proposal is accepted.

If integration is approved, replace the current spatial renderer behind its existing gameplay/presentation boundary. Do not duplicate world simulation or create a second map model. Keep the existing renderer available during the transition until feature parity and validation are established.

## Performance and platform requirements

Desktop and Android are first-class targets. The renderer choice must be based on measured representative scenes, not assumptions that a 3D pipeline is automatically faster or slower.

Measure at minimum:

- Frame time and frame-time variance.
- GPU and CPU workload where available.
- Draw calls and visible entity/material counts.
- Texture and render-target memory.
- Startup and asset-loading cost.
- Binary/package size impact from Bevy feature changes.
- Thermal behavior and sustained performance on representative Android hardware.

Prefer simple geometry, shared materials, texture atlases, and selective lighting. Avoid per-object lights, unnecessary 3D meshes, and expensive transparent overdraw. Establish budgets from actual device measurements rather than arbitrary ceilings.

A 2D fallback renderer is not a permanent requirement, but the current renderer should remain available during prototyping and migration so comparisons and rollback are practical.

## Requirements for a future production integration proposal

TEUS Lab defines and tracks acceptance for its experiments. The following are not TEUS roadmap tasks; they describe the evidence a future production-integration proposal should bring back:

1. The relevant renderer limitations and representative visual cases are documented.
2. The proposed approach demonstrates its handling of ground anchors, depth/occlusion, sprite animation, and difficult object relationships.
3. Any selected lighting/shadow approach is evaluated against the approved pixel-art direction.
4. Desktop and Android feasibility, including Bevy feature requirements, device compatibility, and performance risks, is supported by evidence.
5. The proposal states whether production integration is recommended, deferred, or rejected, and explains why.

Completing the lab investigation would not itself migrate the production spatial renderer. Production integration remains a separate decision and must be represented as such in the TEUS roadmap only after approval.

## Related specifications

- [Spatial gameplay](spatial-gameplay.md) — gameplay rules and spatial interaction direction.
- [Spatial exploration system](../systems/spatial-exploration.md) — current renderer-neutral map model and Bevy presentation behavior.
- [Pixel-art visual direction](pixel-art-direction.md) — approved art and rendering constraints.
- [Presentation direction](presentation-direction.md) — camera-facing gameplay presentation and HUD responsibilities.
- [Architecture](../architecture.md) — state ownership and renderer boundaries.
