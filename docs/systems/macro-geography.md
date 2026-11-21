# Macro-geography

## Current scope

The macro-geography module defines renderer-independent world coordinates and deterministic terrain, shoreline, climate, hierarchical watershed, precipitation-weighted river-channel, and closed-basin depression fields. It does not depend on Bevy or local-map tile geometry.

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

Implemented in v0.53.4:

- `coast::sample_surface` exposes shoreline adjacency for both land and ocean samples using the eight immediate compass neighbors.
- Each sample includes an eight-bit direction mask identifying neighboring samples of the opposite terrain type, allowing callers to determine shoreline orientation without reconstructing neighbor queries.
- Land-side and water-side shoreline samples share the same deterministic elevation boundary. The result is derived on demand from global coordinates and remains consistent across region chunks.
- Climate biome classification now consumes the shared shoreline sampler instead of maintaining a duplicate adjacency check.
- Shoreline sampling rejects underground strata and introduces no persisted state or save migration.

Implemented in v0.53.5:

- `watershed::sample_surface` exposes a coarse drainage cell on a 4,096-sample lattice, its strictly lower downstream coarse cell, and local D8 drainage direction within the same coarse cell.
- Coarse downstream links select the lowest lower neighbor, with signed-coordinate tie-breaking for stable results. Strictly decreasing representative elevation makes the coarse drainage graph acyclic.
- `watershed::trace_to_outlet` follows the coarse network to ocean or a closed inland basin. Traces have a fixed 128-link bound and report `SearchLimit` explicitly rather than inventing an outlet.
- Local depressions do not erase the larger-scale downstream link. The coarse network and local D8 sample are exposed separately so later depression handling and river accumulation can build on them.
- All samples are derived on demand from the existing fixed-point terrain field. Sampling rejects underground strata, handles negative coordinates through Euclidean cell partitioning, and introduces no persisted state or save migration.

Implemented in v0.53.6:

- `rivers::sample_surface` estimates contributing area by traversing the reverse of the deterministic coarse watershed graph. It counts upstream coarse cells, including the sampled cell, without scanning the unbounded world.
- Each query accepts at most 128 contributing cells and examines no more than 1,024 neighboring candidates. The result explicitly reports when additional upstream contributors were found but excluded by the work bound; a truncated count is a lower bound, not a complete basin total.
- A channel is confirmed when at least eight coarse cells contribute. A truncated result below that threshold remains inconclusive rather than being incorrectly classified as a non-river.
- River sampling is renderer-independent, deterministic, on-demand, and rejects underground strata. It adds no persisted hydrology state or save migration.

Implemented in v0.53.7:

- `depressions::sample_surface` follows the coarse watershed to a closed inland outlet and resolves its local spill elevation using a deterministic minimax flood over a fixed 17 × 17 window of coarse terrain cells.
- The result exposes the sink coordinate, floor elevation, local spill elevation, depression depth, and the connected footprint below the spill level. The footprint excludes disconnected low terrain elsewhere in the search window.
- The query has a fixed spatial/work bound and remains stateless, renderer-independent, and consistent across region ownership and negative coordinates. Ocean outlets, unresolved outlet traces, and underground strata do not produce depression samples.
- This is a terrain-only depression foundation, not a claim that every depression is a filled lake. It does not yet account for precipitation when estimating drainage contribution, place persistent waterbodies, or generate a connected water system; no save migration or persisted hydrology state is introduced.

Implemented in v0.53.8:

- `rivers::sample_surface` now sums the climate precipitation-potential value for each accepted upstream coarse cell, alongside its contributing-area count.
- The unsigned Q16 × coarse-cell sum is a relative wetness/contributing-flow index, not a physical discharge or water-volume measurement. The existing river-channel threshold remains area-based until flow thresholds can be calibrated against a more complete water model.
- Each query retains the 128-contributor and 1,024-neighbor-candidate bounds. A truncated result explicitly makes both the contributing-cell count and weighted sum lower bounds.
- The result is deterministic, fixed-point, renderer-independent, and derived on demand. It introduces no persisted hydrology state or save migration.

Implemented in v0.53.9:

- Extended the bounded depression result with a compact connected-footprint bitset and coarse-cell membership checks. Disconnected low terrain remains excluded.
- Added `waterbodies::sample_surface` for deterministic inland-lake coverage. It combines closed-basin identity, connected footprint membership, basin-floor precipitation potential, and coarse-cell elevation to derive a water surface between the basin floor and local spill saddle.
- Lakes use stable IDs derived from world seed, geography ruleset, and basin outlet. Every covered coarse cell in one basin resolves to the same identity; ocean, coastal water, unresolved traces, and underground strata are not inland lakes.
- The first river relationship is an outlet inflow indicator based on the existing river-channel accumulation at the basin sink. Precipitation is only a relative fill proxy; this is not calibrated discharge, volume, evaporation, seasonal hydrology, or a connected water simulation.
- Added optional world-level waterbody modification records keyed by stable generated IDs. Only explicit property changes are saved; generated lake geometry remains deterministic. The new field defaults empty when loading older saves, so the outer save format version does not change.
- Automated tests cover deterministic lake identity and coverage, connected footprint membership, underground rejection, region ownership, modification persistence, and old-save defaults.

Implemented in v0.53.10:

- `hydrology::sample_surface` composes the existing terrain, lake, river, and watershed samplers into one surface-water view. It distinguishes ocean, inland lakes, and classified river channels while leaving ordinary non-channel land unclassified as water.
- River samples expose their immediate downstream coarse cell and classify that destination as ocean, a specific lake outlet, another river channel, lower-threshold land, an inconclusive truncated estimate, or a closed basin. Truncated below-threshold accumulation is explicitly unresolved, not classified as dry land. Ocean and lake cells are terminal nodes; waterbody coverage takes precedence over a coincident river-channel estimate.
- Connectivity reuses the existing strictly downhill coarse watershed graph and bounded river accumulation. It is a local topology contract, not a second routing simulation, physically calibrated discharge, seasonal water balance, or water transport simulation. Generated water geometry remains derived on demand and no save migration is introduced.
- Automated tests cover deterministic river links, downhill destination invariants, terminal ocean/lake nodes, and underground rejection.

## Persistence contract

Each saved `Region` may carry an optional chunk coordinate. Newly generated regions receive deterministic, unique coordinates. An otherwise empty world seeded directly from authored content places its first region at the origin.

Existing saves deserialize without region coordinates. Their geography ruleset defaults to `0`, meaning "not embedded/migrated"; normal loading does not invent coordinates or reinterpret established topology. A dedicated migration must later place legacy regions and locations deterministically, persist migration provenance, and preserve runtime IDs, routes, history, and saved local maps.

## Boundaries and remaining work

The current elevation field is the first physical geography layer, not a complete terrain simulation. Still required:

- Physically calibrated discharge, seasonal water balance, and simulated river/lake water transport; current connectivity exposes deterministic graph links rather than moving or balancing water.
- Mountain ranges and other major terrain barriers.
- A dedicated underground-strata generator for caves and underground networks.
- Geography-aware location footprints, route constraints, and on-demand frontier integration.

Macro-geography coordinates are not:

- Local-map tile coordinates.
- UI or World Map layout positions.
- Runtime entity IDs.
- A claim that legacy route graphs already match physical terrain.

The 256 × 256 resolution describes macro samples per region chunk. It does not prescribe local-map dimensions, real-world kilometres, or a continuous terrain renderer.