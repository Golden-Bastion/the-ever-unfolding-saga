# 3D Spatial World and Presentation Architecture

> Status: approved target design. This document records the intended spatial foundation for TEUS. It does not claim that the production game already implements it. The private [TEUS Lab repository](https://github.com/Golden-Bastion/teus-lab) remains the place for isolated 3D prototypes, technical experiments, and validation evidence.

## Purpose

Define the spatial foundation for a continuous, persistent, progressively generated world. The world is authoritative 3D gameplay space; presentation may remain primarily 2D pixel art. The player does not need to perceive the underlying 3D representation as a conventional 3D game.

This direction supersedes the earlier proposal in which gameplay remained two-dimensional and 3D was only a rendering technique. The earlier proposal is retained as historical design context in [the archived 2D-gameplay/3D-rendering proposal](hybrid-2d-gameplay-3d-rendering-history.md).

## 1. Authoritative spatial world

The world model exists independently of loaded geometry, spatial chunks, and rendering.

- Persistent world entities have stable identities and authoritative state whether or not their surroundings are currently loaded.
- Geography, locations, routes, ownership, resources, discoveries, and lasting changes are world data, not properties owned by a rendered scene.
- A spatial chunk is a runtime/loading representation of part of the world. Loading, unloading, or rebuilding it must not create, destroy, or reset the underlying place.
- Presentation reads authoritative spatial and world state. Camera transforms, render depth, sprite bounds, lighting, and visibility do not decide gameplay outcomes.
- Geometry generation may be deferred until it is needed, but generation must respect already-established world state and shared geographical constraints.

World state, spatial geometry, and presentation are related layers with explicit interfaces; they are not competing sources of truth.

## 2. Continuous world and topology

The target is seamless physical traversal wherever the world is physically connected. This applies to outdoor environments, settlements, buildings, interiors, caves, dungeons, and connected underground networks.

A named location is a persistent world concept and geographical description, not inherently one map, scene, or loading boundary. A large location may span many chunks; a chunk may contain part of a location or parts of more than one logical area. Region and location boundaries do not automatically interrupt movement.

Doors, gates, staircases, ladders, shafts, tunnels, cave passages, bridges, and other structures provide physical connections through the 3D environment. Crossing them should not require a destination-selection menu or a separate travel scene when the geometry is intended to be traversable. Streaming may load the destination space before the player reaches it.

The geographical adjacency graph and travel connectivity remain distinct:

- **Geographical adjacency** describes which natural regions share boundaries and how their geography relates.
- **Travel connectivity** describes established roads, paths, bridges, ferries, passes, transport services, and other route-level relationships.
- **Spatial traversability** is determined by the actual geometry, barriers, actor capabilities, and current world state.
- **Fast travel** remains an optional convenience for known, usable routes. It does not replace or define physical movement.

A travel graph may support route discovery, journey estimation, navigation guidance, faction logistics, and fast travel. It must not be treated as a substitute for the continuous spatial world or as proof that an actor can physically pass through blocked geometry.

## 3. Three-dimensional gameplay, two-dimensional visual identity

Gameplay uses actual three-dimensional positions and geometry. At minimum, authoritative spatial rules account for all three spatial axes.

This does not require realistic 3D artwork or a conventional first-person/third-person 3D presentation. TEUS may retain:

- 2D pixel-art characters, creatures, props, effects, and environmental textures.
- Sprite sheets and authored frame-based animation.
- A flexible three-quarter camera and controlled perspective.
- Orthographic or otherwise carefully constrained presentation where it best preserves readability.
- A pixel-art-led palette, silhouettes, pixel scale, and visual composition.

Sprites and textures are presentation assets placed into the 3D world. Their visible bounds and ground-contact anchors are not automatically their collision geometry or gameplay footprint. Occlusion and visual depth must not silently change physical reachability.

The camera and presentation may conceal the 3D implementation from the player. That is a valid outcome, not a requirement to advertise the game as 3D.

## 4. Movement and actor capabilities

Ordinary movement is geometry-constrained. Grounded actors follow traversable surfaces and cannot move through solid obstacles merely because a two-dimensional projection makes the path appear clear.

Movement capabilities determine which spatial connections an actor can use. Capabilities may include grounded locomotion, climbing, jumping, falling, swimming, flying, squeezing, or other abilities justified by gameplay and creature design.

The same world can therefore support different valid paths for different actors. A bird may fly over a wall; a horse may be unable to climb a narrow staircase; an injured person may be unable to traverse a difficult slope.

Navigation must consider:
- Actual 3D geometry and traversable surfaces.
- Actor size, movement capabilities, condition, and relevant equipment.
- Doors, barriers, elevation changes, slopes, ladders, and other connections.
- Hazards and movement costs where supported.
- Perception and knowledge constraints when choosing what route an actor would attempt.

A 3D navigation mesh is one possible representation, not a decision already made. Different geometry types may require different navigation representations behind common traversal queries.

## 5. Hybrid geometry representation

No single geometry representation is required for every environment. Use the representation that fits the structure and gameplay:

- Heightfields for suitable broad outdoor terrain.
- Mesh geometry for cliffs, overhangs, rock formations, complex surfaces, and structures.
- Volumetric or voxel-like representations where excavation, destruction, or enclosed terrain makes them worthwhile.
- Modular geometry for buildings, rooms, architecture, and repeatable construction.
- Specialized geometry or graph-backed structures for cave and underground networks where appropriate.

These representations must expose consistent gameplay-facing capabilities for collision, movement, navigation, spatial queries, interaction, and persistence. A geometry backend must not create a separate ruleset for movement or interaction.

The choice of representation is a technical means to support the world, not a reason to force every environment into the same shape.

## 6. Selective environmental modification

Environmental modification is systemic where the material, structure, tools, and circumstances make it plausible. Not everything is destructible or modifiable.

Keep these capabilities distinct:
- **Damage and destructibility:** whether an entity or structure can be damaged, broken, or destroyed.
- **Modification:** whether its shape or state can be altered, such as digging soil or excavating suitable rock.
- **Replacement:** whether it can be removed and something else built or installed in its place.

Examples may include digging soil, mining suitable rock, felling trees, burning flammable structures, demolishing a vulnerable wall, damaging a bridge, or collapsing a supported cave section. Solid geological formations and appropriately durable structures need not be arbitrarily removable.

Modification rules should follow material and structural properties, tools, force, and gameplay abstractions. They should not be a collection of unrelated one-off scripts where a reusable rule is justified.

Persistent changes must be stored as authoritative world changes or stable modifications to generated geometry. Regenerating or streaming a chunk must never restore a destroyed bridge, erase an excavation, or undo another established change.

## 7. Gameplay-driven physical simulation

TEUS is a game, not a scientific simulator. Simulate physical processes when they produce meaningful gameplay, believable consequences, or useful systemic interactions. Choose an abstraction appropriate to the experience rather than pursuing physical accuracy for its own sake.

Examples include fire spreading among suitable materials, smoke affecting visibility, water following terrain and interacting with passages, falling objects, structural collapse, and damage propagation. These are examples of possible capabilities, not a commitment to implement all of them at once.

Keep physical resolution distinct from world-level consequences:
- Physics and geometry resolve physical contact, movement, forces, and supported physical events.
- World simulation owns authoritative consequences such as injuries, deaths, fires as ongoing world conditions, damaged infrastructure, NPC evacuation decisions, faction responses, and history.
- NPC and faction systems decide how actors respond using their own knowledge, goals, capabilities, and resources.

A process may use detailed simulation while relevant and aggregate or scheduled resolution when inactive, provided its promised consequences and causal history are preserved. Do not run continuous high-cost simulation merely because an engine can do so.

## 8. World units and coordinate spaces

The canonical scale is **1 world unit (WU) = 1 metre**.

Distinguish three coordinate responsibilities:

1. **Global coordinates** identify the authoritative position in the world.
2. **Regional coordinates** express that same position relative to a stable geographical region identity.
3. **Local coordinates** express it relative to a nearby simulation, physics, or rendering origin.

These are transformations of one position, not separate authoritative positions. Re-origining a local simulation or render space must not change an entity's global position.

The global representation must remain reliable at very large distances. A high-precision or integer/fixed-point global representation with nearby floating-point coordinates is a possible implementation, but the exact numeric types, precision, and conversion contract remain open for technical design and validation.

The world has no predetermined maximum extent. New regions are generated progressively as the player approaches the established frontier. A finite coordinate representation must still have explicit overflow handling; “unbounded” is a world-design goal, not permission to ignore numeric limits.

## 9. Spatial chunks and streaming

Use uniform base spatial chunk dimensions as the common technical loading unit. Large locations and environments are composed of more chunks, not special map types with unrelated streaming rules. The exact chunk dimensions and vertical partitioning remain open.

A chunk is not a region, location, biome, or simulation domain. It is a technical unit for spatial generation, loading, memory management, and possibly persistence partitioning. Different environments may contain very different geometry and entity counts within the same base chunk dimensions.

Keep three independent decisions:
- **Spatial loading:** whether geometry and runtime spatial structures are resident.
- **Simulation activity:** what entities and processes are currently updated in detail, aggregate form, or through deferred work.
- **Rendering:** what is visible or needed by the camera and presentation.

A loaded chunk need not be fully simulated or rendered. A relevant entity or process may need simulation while outside the rendered area. A visible region may use simplified presentation while retaining authoritative geometry and state.

Streaming is relevance-based, not distance-only. Consider proximity, expected movement, visibility, likely paths, connected doors/passages, active encounters, NPCs, scheduled activity, simulation dependencies, and memory/performance budgets. Predictive loading should prepare connected spaces before the player reaches them and avoid visible loading boundaries during ordinary movement.

The player should move naturally between chunks and logical locations. Chunk boundaries must not require an interaction, travel selection, scene transition, or artificial pause.

## 10. Unloading, persistence, and reconstruction

Unloading releases temporary runtime representations and resources; it does not erase the world.

Persist meaningful state, including as applicable:
- Modified or destroyed geometry and structures.
- Excavations and construction.
- Items and entities that remain in the world.
- Stable entity identities, ownership, relationships, and important memories.
- Discoveries and spatial changes.
- Ongoing processes, deadlines, and state needed for deterministic catch-up.

Do not automatically persist every transient physics-engine detail. Persist the authoritative gameplay state needed to reconstruct a valid result. When a chunk reloads, reconstruct geometry and runtime entities from deterministic base generation plus persistent modifications and current world state.

Ongoing processes must either continue through an appropriate active/aggregate simulation or be resolved through scheduled/deferred updates. Unloading cannot freeze a fire, erase an obligation, or make an NPC forget a consequential commitment unless the relevant gameplay rule explicitly says so.

## 11. Platform and validation constraints

Desktop and Android remain first-class targets. The architecture must support bounded active geometry, relevance-based streaming, and different rendering detail without changing authoritative gameplay outcomes.

Validation should eventually cover:
- Continuous traversal across chunk, region, location, building, cave, and underground boundaries.
- No visible or gameplay discontinuity when adjacent chunks load in different orders.
- Correct collision and navigation across supported geometry representations.
- Different actor capabilities producing valid, distinct traversal results.
- Stable global positions during local-origin changes.
- Persistent destruction, excavation, construction, item placement, and entity state after unload/reload and save/load.
- Correct separation of simulation, loading, and rendering relevance.
- Deterministic generation and idempotent reconstruction.
- Bounded CPU, memory, GPU, and loading costs on representative desktop and Android hardware.

## Details intentionally left open

The following are not settled by this specification:

- Exact base chunk dimensions and vertical chunk layout.
- Exact global coordinate numeric representation and precision strategy.
- Whether individual geometry types use heightfields, meshes, voxels, or another specific backend in each case.
- Physics engine configuration and which processes warrant physical simulation.
- Navigation representation and algorithms for each geometry type.
- Exact streaming radii, relevance weights, prefetch rules, and memory budgets.
- Exact world dimensions for regions, settlements, interiors, and continents.
- Rendering projection, camera controls, sprite-plane conventions, lighting, and shadow implementation details.
- The production integration sequence from TEUS Lab experiments.

These should be resolved through focused design and representative prototypes, not guessed in this document.

## Related specifications

- [World topology and seamless traversal](world-topology-and-travel.md).
- [Procedural content generation](procedural-content-generation.md).
- [World scale and travel](world-scale-and-travel.md).
- [World simulation](world-simulation.md).
- [NPC behaviour](npc-behavior.md).
- [Spatial gameplay](spatial-gameplay.md).
- [Pixel-art visual direction](pixel-art-direction.md).
- [Architecture](../architecture.md).
