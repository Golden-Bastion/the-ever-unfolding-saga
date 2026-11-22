# Macro-geography

## Current scope

The macro-geography module defines renderer-independent world coordinates and deterministic surface terrain, mountain ranges and route-facing barrier signals, shoreline, climate, hierarchical watershed, precipitation-weighted river-channel, closed-basin depression, underground cave-network fields, persistent location anchors/footprints, legacy embedding, bounded terrain-aware route planning, on-demand frontier materialization, and persistent individual cave-chamber topology. It does not depend on Bevy or local-map tile geometry.

Implemented in v0.53.1:

- `RegionCoordinate` identifies a fixed 256 × 256 region chunk using signed `i64` chunk coordinates.
- `MacroCoordinate` identifies a sample in one shared signed global lattice and carries its vertical stratum.
- Stratum `0` is the surface; negative stratum indices identify underground layers. Surface and underground samples may share horizontal coordinates without sharing sample seeds.
- `underground::sample_underground` samples negative strata independently. It generates jittered chamber nodes on a 64-sample global lattice and reciprocal cardinal links where both neighboring chambers exist and the shared deterministic edge is open. Chamber-to-chamber passages follow the segment between stable node centers, so networks remain coherent across region boundaries.
- Underground samples distinguish solid rock, chambers, and passages. Open samples expose their stable chamber-center coordinate and connected-neighbor mask. A query inspects one node and at most four neighbors; it does not scan the world or persist generated cave geometry.
- Stratum numbers are layer identifiers, not calibrated physical depths. The initial model modestly increases chamber frequency on deeper layers but does not model rock composition, geological formation, or underground water. Deterministic vertical-connection candidates are described below.
- Region-local to global conversion uses checked multiplication and addition and rejects local indices outside the chunk.
- Global to region-local conversion uses Euclidean division and remainder. This is essential for negative coordinates: global sample `-1` belongs to region `-1`, local sample `255`, not region `0`.
- `coordinate_seed` derives a deterministic seed from the world seed, geography ruleset version, and global sample coordinate including stratum. It does not depend on traversal/generation order, runtime IDs, or names.
- `GEOGRAPHY_RULESET_VERSION` identifies the deterministic coordinate and terrain sampling contract. Ruleset 1 retains the original elevation field; ruleset 2 adds mountain uplift.

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

Implemented in v0.53.11:

- Added a deterministic, warped ridged-noise mountain field on the shared global macro lattice. Broad ridge belts receive regional variation and bounded positive uplift on land.
- Integrated mountain uplift into the authoritative surface elevation sampler, so climate cooling, watershed routing, river accumulation, and depression analysis consume the same mountain-shaped terrain.
- Kept all calculations fixed-point, stateless, renderer-independent, and independent of region generation order. Ocean and underground samples receive no mountain uplift.
- Advanced the geography ruleset to version 2. Existing worlds pinned to ruleset 1 retain their original elevation field; no save migration or persisted mountain state is introduced.

Implemented in v0.53.14:

- Added persisted location-geography records with stable authored/generated/compatibility keys, global macro anchors, and validated footprint radii. New locations receive deterministic anchors within their owning region chunk; authored content may optionally specify an exact surface anchor and footprint radius.
- Added deterministic compatibility embedding for legacy regions without coordinates. Region ordering follows a stable breadth-first traversal of the saved route graph with stable ID tie-breaking. Existing coordinates are never moved.
- Legacy worlds receive a persisted geography seed derived from saved world/region/location/route identities when no generation seed exists. Display names and content ordering are excluded. Geography ruleset and embedding versions are persisted separately from the save-format version.
- Added a bounded candidate-corridor assessment that reports ocean, lake, river, unresolved-water, and mountain-ridge samples independently. It is retained as a lightweight diagnostic; route creation now uses the terrain-aware planner described below.
- Automated tests cover stable compatibility embedding, identity/allocator preservation, deterministic location placement, authored anchors, bounded footprints, and route-corridor assessment.

Implemented in v0.53.13:

- Added `underground::sample_underground` for deterministic cave geography on negative strata. Surface and positive strata are rejected; each negative stratum has its own coordinate-derived network.
- Generated sparse chamber nodes on a shared 64-sample global lattice. Adjacent chambers connect through deterministic, reciprocal cardinal passages only when both chamber nodes exist and their shared edge is open.
- Samples distinguish solid rock, chambers, and passages. Open samples expose the stable global center coordinate of their chamber node and its connected-neighbor mask, giving later route and local-map systems explicit topology instead of treating caves as unrelated empty patches.
- Queries inspect one chamber and at most four neighboring nodes. They use checked coordinate arithmetic, Euclidean partitioning for negative coordinates, and no world-sized search, mutable generation cache, or persisted generated geometry.
- Stratum indices are independent layer identifiers, not physical depth measurements. Deeper layers modestly increase chamber frequency; geological materials, vertical shafts between layers, underground water, and cave gameplay remain future work.
- Added automated coverage for deterministic seed/layer variation, solid-rock distribution, reciprocal connected passages, region-boundary continuity, negative coordinates, and extreme-coordinate safety.

Implemented in v0.53.12:

- Extended the shared surface-terrain sample with the mountain-ridge strength already used to generate elevation. Route and location-placement systems can now inspect the barrier signal without separately resampling the mountain field.
- Exposed a consistent mountain-barrier classification while retaining the underlying Q16 strength. The signal is a soft crossing-resistance hint, not a rule that makes every mountain cell impassable; route selection can seek weaker corridors through a ridge belt as candidate passes.
- Kept the field deterministic, fixed-point, renderer-independent, and unchanged for ruleset 1. This is derived data only and introduces no save migration or geography-ruleset increment.
- Water barriers remain represented by the existing surface-water sampler: oceans, lakes, rivers, and unresolved river estimates are distinct and must not be collapsed into ordinary land during route planning.

Implemented in v0.53.15:

- Replaced direct-line route creation with deterministic bounded A* over the shared macro lattice. The planner uses a 32-sample search grid with midpoint validation at 16-sample intervals, an eight-direction grid, a 4,096-node expansion cap, a 96-cell maximum endpoint distance, and a 256-point persisted path cap.
- Ocean terrain is impassable for this overland planner. Elevation changes and mountain-ridge strength contribute soft movement costs, allowing routes to prefer lower, gentler corridors without treating every mountain as an absolute barrier.
- Persisted route geography records the canonical endpoint-A-to-endpoint-B path, planning status, total cost, broad route profile, and detected physical crossings. River crossings identify a ford-or-bridge requirement, lake crossings identify a ferry requirement, and inconclusive river estimates require survey; mountain barriers are marked as pass candidates.
- Route geography is generated when location geography is embedded and when new routes are added to an already-geography-backed world. Existing saved route plans are never silently regenerated. Older saves default the new route field to absent and receive plans through the existing geography migration.
- New local-map passages inherit their route profile and prefer the compass side indicated by the macro path. Existing passages retain their IDs, positions, and map geometry; reconciliation only backfills missing route profiles.
- Cave routes remain unsupported until explicit vertical entrances connect surface locations to underground strata. This planner does not invent shafts or infer physical depth from stratum numbers.
- Automated tests cover deterministic land paths, endpoint and land invariants, route-heading reversal, old-save defaults, route-plan persistence, passage-side preference, profile backfilling, and preservation of established passage positions.

Implemented in v0.53.16:

- `frontier_targets_for_location` exposes up to four cardinal frontier invitations from a surface location without creating world entities. Stable target keys derive from the source location's persisted geography key and exact source/target region coordinates; generation order, display names, and runtime ID allocation do not affect geography identity.
- `materialize_frontier` validates the persisted frontier target, selects deterministic candidate anchors near the destination region edge, and requires the existing bounded A* planner to produce a valid land path before allocating world IDs.
- A successful request creates or reuses only the adjacent region and one destination location, then persists a bidirectional `WorldRoute` with its canonical macro path and crossing profile. It does not recursively generate neighboring regions. Repeated requests return the same destination and route.
- Local maps persist pending frontier passages as `MapExit` records with an explicit frontier marker and no destination/route yet. Interacting with one materializes the destination and upgrades that exact passage in place, preserving its map-local ID, position, and geometry. The destination's reciprocal passage is reconciled when its map is entered.
- The optional pending-frontier marker defaults absent when reading older saves. Existing region coordinates, location anchors, routes, and established local maps are not regenerated or relocated. No save-format or geography-ruleset change was required.
- Automated tests cover read-only candidate listing, stable identities, allocator-order-independent coordinates/paths, single-region expansion, repeat-use idempotence, invalid target rejection, pending interaction discovery, and passage upgrade.

Implemented in v0.53.17:

- `vertical::connection_for_cell` derives at most one vertical opening for an underground network cell and the layer immediately above it. For upper stratum zero, the upper endpoint must be exposed land and the lower endpoint must be open cave space on stratum -1. For negative upper strata, both endpoints must be open cave samples and the lower endpoint is exactly one stratum deeper.
- Candidate coordinates are selected from a fixed 9 × 9 grid around the cave cell center. A stable coordinate-derived score selects the same candidate regardless of query order, region ownership, or runtime IDs.
- `vertical::connections_near` inspects at most 25 network cells and returns stable, ordered candidates. No world-sized scan, mutable cache, or persisted generated cave geometry is introduced.
- These candidates expose valid vertical geography but do not yet allocate cave locations, add authoritative `WorldRoute` records, or create local-map entrances. The next integration stage must connect them to persistent world topology without changing established locations or maps.

Implemented in v0.53.18:

- `caves::connection_targets_for_location` exposes immediate cave connections from a persisted location. Surface locations receive bounded nearby shaft candidates; cave locations receive only their actual reciprocal horizontal neighbors and vertical connections attached to their own chamber.
- Each generated chamber has its own persistent `Location`, anchored at the underground sampler's stable chamber center and keyed by stratum plus signed global coordinates. Its region is created or reused at the matching region/stratum coordinate.
- `caves::materialize_connection` validates the selected target against the current deterministic topology, creates or reuses exactly one destination chamber/region, and adds a bidirectional authoritative `WorldRoute`. Stable route IDs are based on the physical connection, not allocated entity IDs.
- A cave chamber exposes the reverse side of a previously materialized surface shaft and reuses the original surface location. Deeper shafts and horizontal passages likewise resolve to one route identity from either endpoint.
- Cave routes deliberately do not receive surface A* paths. The existing overland planner reports unsupported strata rather than inventing a surface corridor. Local-map shaft/entrance placement and interaction remain the next integration step; this stage changes persistent world topology only.
- Generated cave geometry remains derived on demand. Only materialized locations, regions, and routes are persisted; repeated requests are idempotent and no world-sized scan or recursive network generation is performed.

Implemented in v0.53.20:

- New surface-route and frontier passages use center-out edge candidates. The first connection approaches near the middle of its selected side; additional connections spread symmetrically toward the corners. Existing saved passage IDs and positions are not relocated.
- The placement order still honors the persisted macro route heading, deterministic compass-side fallback, and authored gate threshold waypoints. This improves the local connection point without pretending the coarse macro route path is detailed local-map road geometry.
- `MapExit::visual_kind` derives a renderer-neutral category from the stored route profile, pending-frontier state, or cave target: trail, mountain pass, river/water crossing, mixed/uncertain/unclassified route, frontier, surface cave entrance, vertical shaft, or horizontal cave passage.
- The Bevy spatial renderer maps those categories to restrained provisional tints while retaining one generic passage image. Dedicated physical route and cave assets are tracked in [#491](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/491); gameplay route identity and reachability do not depend on the art.
- Local-map generator version 7 records the placement behavior change. Established maps are not regenerated and their existing passages remain untouched; only newly added passages use center-out placement.

Implemented in v0.53.19:

- `spatial::configure_map_cave_connections` converts the immediate cave targets for a location into physical interior map passages. Surface locations expose up to three nearest nearby entrances; cave chambers expose their immediate horizontal neighbors and adjacent-stratum shafts.
- Interior passage positions are selected deterministically from walkable cells reachable from the map entry. They avoid existing exits, discoveries, actors, encounters, and prop footprints; they do not require a fake edge opening or a separate overland path.
- A pending cave passage stores its `CaveConnectionTarget` and has no destination location or route yet. Interaction revalidates and materializes exactly that target, then upgrades the same passage with the resulting authoritative route and location IDs.
- Cave routes are excluded from ordinary edge-route reconciliation. When a chamber map is entered, its deterministic cave targets recreate the physical reverse passage and reuse an already materialized route when present.
- Local-map generator version 6 marks the migration. Older maps are upgraded on entry by adding transitions in place; existing terrain, discovered tiles, points of interest, props, encounters, actor placements, and established route passages are retained. The new optional passage field defaults absent in older saves.

## Persistence contract

Each saved `Region` may carry an optional chunk coordinate. Newly generated regions receive deterministic, unique coordinates. An otherwise empty world seeded directly from authored content places its first region at the origin.

Existing saves deserialize without region coordinates. Their geography ruleset defaults to `0`, meaning "not embedded/migrated"; normal loading does not invent coordinates or reinterpret established topology. A dedicated migration must later place legacy regions and locations deterministically, persist migration provenance, and preserve runtime IDs, routes, history, and saved local maps.

## Boundaries and remaining work

The macro-geography fields, underground cave networks, deterministic legacy embedding, persistent location anchors/footprints, bounded terrain-aware route paths, crossing profiles, route-aligned local-map passages, idempotent on-demand frontier materialization, deterministic vertical candidates, and persistent individual cave-chamber locations/routes are implemented. Remaining geography/topology work is richer authored-location connection placement. Environmental placement archetypes are implemented: settlements prefer flatter land near classified freshwater, wilderness locations retain broad land preferences, and landmarks favor elevated, geologically distinctive terrain. Dedicated route artwork is tracked under #491.

The geography milestone does not require physically calibrated discharge, seasonal water balance, or simulated river/lake transport. The relative precipitation-weighted river index and deterministic connectivity are sufficient for terrain classification and route foundations. Seasonal rainfall, drought, and their consequences for settlements, crops, and travel belong to the time-driven world-simulation work rather than this geography layer.

Macro-geography coordinates are not:

- Local-map tile coordinates.
- UI or World Map layout positions.
- Runtime entity IDs.
- A claim that legacy route graphs already match physical terrain.

The 256 × 256 resolution describes macro samples per region chunk. It does not prescribe local-map dimensions, real-world kilometres, or a continuous terrain renderer.

## Location placement archetypes

New location anchors use an authored `archetype` preference, defaulting to `wilderness` for older campaign content and generated locations without an explicit type.

- `settlement` ranks 128 deterministic land candidates toward lower-relief terrain, then checks at most the 24 strongest candidates for a classified river or inland lake within 128 macro samples in a cardinal direction. This bounds the more expensive hydrology probes.
- `wilderness` keeps broad environmental preferences and does not require proximity to water or low relief.
- `landmark` favors higher elevation and stronger mountain-ridge character, allowing distinctive elevated sites.

Land is required for generated candidates, and a full requested footprint is strongly preferred before archetype preferences are applied. These are deterministic ranking preferences, not hard biome restrictions or guarantees that a region contains every archetype's ideal terrain. Freshwater proximity uses the existing bounded hydrology sampler; no water state is added or persisted. Explicit authored anchors remain authoritative. Existing saved location anchors are never moved; the location-geography embedding version advances so older worlds that still lack anchors use the new placement policy when migrated.
