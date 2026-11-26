# World Scale, Coordinates, and Seamless Travel

## Design requirement

The world must feel large enough to support meaningful journeys. It is a continuous, progressively generated 3D spatial world rather than a collection of local maps connected by mandatory transitions. Outdoor environments, settlements, buildings, interiors, caves, dungeons, and connected underground spaces are physically traversable wherever geography and structures permit.

**Canonical scale: 1 world unit (WU) = 1 metre.** Use this unit for geometry, character dimensions, movement, physics, collision, navigation, and physical distance. Rendering scale and pixel density are independent of world scale.

The world has no predetermined maximum geographical extent. It expands by generating additional distinct regions at the frontier. Regions are variable-sized natural geographical areas with stable boundaries; their dimensions are not determined by spatial chunk size. Exact dimensions for regions, continents, settlements, and locations remain open and should be established through representative gameplay needs rather than arbitrary universal numbers.

An ordinary outdoor journey should feel substantial without making movement artificially slow. Interiors can remain compact when their fiction calls for it. A large settlement, wilderness, ruin complex, or underground network can span many spatial chunks.

## Movement and time

- Preserve responsive character movement. Do not make the player artificially slow just to make a small map take longer to cross.
- Let travel duration emerge from actual distance, terrain, route quality, weather, encumbrance, character capability, and meaningful interruptions where those systems exist.
- Keep physical travel and known-route fast travel complementary. Fast travel should represent a journey the character could actually undertake, not erase the geography.
- Stream spatial chunks transparently as the player moves. Chunk boundaries must not require a travel choice, scene transition, or interruption.
- Use uniform base spatial chunk dimensions as a technical loading unit; large environments are composed of more chunks. Chunk dimensions do not define natural region size.
- Maintain geographical and physical continuity across chunk, location, and region boundaries. Roads, rivers, mountain ranges, coastlines, and cave passages must not reset or terminate merely because a technical partition ends.
- Keep spatial loading, active simulation, and rendering relevance independent. Streaming is relevance-based rather than distance-only.

## Current implementation baseline

The current v0.53.x implementation still uses bounded local maps: new outdoor maps default to 64×48 tiles, compact interiors may be smaller, the authored shrine chamber uses 32×24, and World Lab exposes 64×48, 80×56, and 96×64 presets. These are transitional implementation dimensions, not the target world scale or spatial chunk dimensions. Existing saved maps and their established state remain authoritative during the current implementation.

The current route-travel implementation derives a relative journey cost from persisted route geometry, terrain, biome modifiers, and explicit crossing delays. Those legacy route-cost units are not metres and must eventually be reconciled with 1 WU = 1 metre and actual 3D traversable distance. Known-route fast travel remains an optional convenience and should estimate journeys using authoritative route and world-time rules. Weather, route condition, encumbrance, and character capability may contribute when their authoritative systems exist. New generic outdoor maps now derive broad terrain from the location's persisted macro anchor and footprint. A bounded macro sample grid is resampled across local tile dimensions with aspect-ratio-aware sampling; macro samples are not treated as individual tiles or physical distance units. The projection currently distinguishes open land, ocean/lake water, sandy coast/desert, and rocky highlands. Newly generated generic outdoor maps also receive tile-resolution river bands derived from the shared watershed network. Their curved centre-lines share exact endpoints between adjacent drainage cells, so a river does not restart at a local-map or region boundary; local tiles inside a channel are rendered as blocked water with provisional material tints. Planned route segments store Q16 bank-entry/exit positions when they cross both sides of a channel. Those exact bank pairs are projected into a location's local tile grid when they fall inside its footprint, allowing the local generator to prioritize a feasible ford/bridge span for that route. Local route exits likewise use the persisted path's exact footprint-boundary intersection rather than only its broad heading. Physical arrivals use the matching reciprocal exit or a safe projected inbound edge on newly generated maps. Water-safe local corridors do not erase river/lake tiles or bypass closed doors. These are derived geometric details, not a fluid simulation or a claim that bridge/ford structures already exist. The current production renderer still creates complete local-map scenes and does not yet implement the target continuous-world chunk streaming. Performance, memory use, and camera behavior on representative Android hardware must be measured as the streaming architecture is prototyped.

## Scale hierarchy

1. **Spatial chunk:** a uniform base technical unit for generation, loading, memory management, and runtime spatial representation. It is not a player-facing map or a natural geographical unit.
2. **Location:** a persistent named place or area with its own identity, history, and relevant entities. It may span many chunks and may include outdoor, indoor, and underground spaces.
3. **Region:** a variable-sized natural geographical area with stable boundaries, containing or referencing natural geography, locations, and travel infrastructure.
4. **World:** the progressively generated collection of connected regions, with coherent continental geography, watersheds, coastlines, settlements, underground networks, and travel infrastructure.

The physical unit is settled at 1 WU per metre. Exact chunk dimensions, regional extents, and representative location sizes remain open. Calibrate them together using outdoor, settlement, interior, and underground gameplay examples.

## Implementation constraints

- Preserve renderer-neutral world and geography data. Rendering chunks must not become the authority for terrain, route connectivity, or persistence.
- Keep stable world, region, location, route, entity, and discovery identities across streaming and generation changes.
- Reconstruct spatial chunks from deterministic base geometry plus persistent world modifications. Unloading a chunk must not erase state; loading it must not reroll established geography or entities.
- Existing local maps remain authoritative under the current implementation until a deliberate migration replaces them; the target architecture does not require preserving the one-map-per-location model.
- Design for desktop and Android together. Large worlds should use bounded active rendering, deterministic generation, and sensible memory budgets rather than spawning every tile in a region at once.
- Use shared global geography coordinates for continuity, but do not equate macro samples, technical coordinate partitions, spatial chunks, natural regions, or rendered geometry.
