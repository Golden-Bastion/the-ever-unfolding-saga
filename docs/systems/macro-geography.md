# Macro-geography

## Current scope

The macro-geography module defines the shared coordinate and deterministic sampling contract for the persistent world. It is renderer-independent and does not depend on Bevy or local-map tile geometry.

Implemented in v0.53.1:

- `RegionCoordinate` identifies a fixed 256 × 256 region chunk using signed `i64` chunk coordinates.
- `MacroCoordinate` identifies a sample in one shared signed global lattice and carries its vertical stratum.
- Stratum `0` is the surface; negative stratum indices identify underground layers. Surface and underground samples may share horizontal coordinates without sharing sample seeds.
- Region-local to global conversion uses checked multiplication and addition and rejects local indices outside the chunk.
- Global to region-local conversion uses Euclidean division and remainder. This is essential for negative coordinates: global sample `-1` belongs to region `-1`, local sample `255`, not region `0`.
- `coordinate_seed` derives a deterministic seed from the world seed, geography ruleset version, and global sample coordinate including stratum. It does not depend on traversal/generation order, runtime IDs, or names.
- `GEOGRAPHY_RULESET_VERSION` currently identifies the initial coordinate/sampling contract as version `1`.

## Persistence contract

Each saved `Region` may carry an optional chunk coordinate. Newly generated regions receive deterministic, unique coordinates. An otherwise empty world seeded directly from authored content places its first region at the origin.

Existing saves deserialize without region coordinates. Their geography ruleset defaults to `0`, meaning "not embedded/migrated"; normal loading does not invent coordinates or reinterpret established topology. A dedicated migration must later place legacy regions and locations deterministically, persist migration provenance, and preserve runtime IDs, routes, history, and saved local maps.

## Boundaries

This module does not currently generate elevation, landmasses, oceans, coastlines, climate, biomes, watersheds, rivers, mountain ranges, lakes, underground strata, location footprints, or physical routes. Those systems must derive from the same global coordinate lattice and versioned world seed.

Macro-geography coordinates are not:

- Local-map tile coordinates.
- UI or World Map layout positions.
- Runtime entity IDs.
- A claim that legacy route graphs already match physical terrain.

The 256 × 256 resolution describes macro samples per region chunk. It does not prescribe local-map dimensions, real-world kilometres, or a continuous terrain renderer.
