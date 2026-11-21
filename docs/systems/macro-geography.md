# Macro-geography

## Current scope

The macro-geography module defines renderer-independent world coordinates and deterministic terrain fields. It does not depend on Bevy or local-map tile geometry.

Implemented in v0.53.1:

- `RegionCoordinate` identifies a fixed 256 × 256 region chunk using signed `i64` chunk coordinates.
- `MacroCoordinate` identifies a sample in one shared signed global lattice and carries its vertical stratum.
- Stratum `0` is the surface; negative stratum indices identify underground layers. Surface and underground samples may share horizontal coordinates without sharing sample seeds.
- Region-local to global conversion uses checked multiplication and addition and rejects local indices outside the chunk.
- Global to region-local conversion uses Euclidean division and remainder. This is essential for negative coordinates: global sample `-1` belongs to region `-1`, local sample `255`, not region `0`.
- `coordinate_seed` derives a deterministic seed from the world seed, geography ruleset version, and global sample coordinate including stratum. It does not depend on traversal/generation order, runtime IDs, or names.
- `GEOGRAPHY_RULESET_VERSION` identifies the deterministic coordinate and terrain sampling contract as version `1`.

Implemented in v0.53.2:

- `terrain::sample_surface` produces a deterministic signed Q16 elevation sample from the world seed, ruleset version, and global surface coordinate.
- Three value-noise scales combine broad continental structure with regional and smaller-scale variation. The broad field forms connected continents; the smaller fields shape irregular coasts, peninsulas, bays, and islands.
- Positive elevation is land; zero or negative elevation is ocean. This initial land mask is a derived threshold, not yet an explicit coastline or hydrology model.
- Noise interpolation uses integer fixed-point arithmetic. The land/ocean decision therefore does not rely on platform-specific floating-point rounding.
- The sampler is stateless and coordinate-based. Sampling order, chunk visitation order, runtime IDs, and display names cannot affect results.
- The surface sampler rejects nonzero strata until underground terrain has its own generation model.

Implemented in v0.53.3:

- `climate::sample_surface` combines broad and regional deterministic temperature and moisture fields on the same global lattice.
- Temperature is reduced by elevation; moisture receives a bounded maritime influence from ocean samples at increasing distances. Precipitation potential is derived from moisture and temperature.
- The public sample exposes signed Q16 temperature, unsigned Q16 moisture and precipitation potential, plus a stable `Biome` classification. Integer arithmetic owns all classification decisions.
- The initial biome catalogue is Ocean, Coast, Desert, Grassland, Temperate Forest, Taiga, Tundra, and Alpine. Coast is assigned only to land samples directly adjacent to ocean; high elevations take the Alpine classification.
- Sampling is stateless, seed/ruleset keyed, independent of region visitation order, and rejects underground strata.

## Persistence contract

Each saved `Region` may carry an optional chunk coordinate. Newly generated regions receive deterministic, unique coordinates. An otherwise empty world seeded directly from authored content places its first region at the origin.

Existing saves deserialize without region coordinates. Their geography ruleset defaults to `0`, meaning "not embedded/migrated"; normal loading does not invent coordinates or reinterpret established topology. A dedicated migration must later place legacy regions and locations deterministically, persist migration provenance, and preserve runtime IDs, routes, history, and saved local maps.

## Boundaries and remaining work

The current elevation field is the first physical geography layer, not a complete terrain simulation. Still required:

- Explicit coast and shoreline information derived from neighboring elevation samples.
- Climate, temperature, moisture, and biome classification.
- Watersheds, drainage direction, rivers, lakes, and connected water systems.
- Mountain ranges and other major terrain barriers.
- A dedicated underground-strata generator for caves and underground networks.
- Geography-aware location footprints, route constraints, and on-demand frontier integration.

Macro-geography coordinates are not:

- Local-map tile coordinates.
- UI or World Map layout positions.
- Runtime entity IDs.
- A claim that legacy route graphs already match physical terrain.

The 256 × 256 resolution describes macro samples per region chunk. It does not prescribe local-map dimensions, real-world kilometres, or a continuous terrain renderer.
