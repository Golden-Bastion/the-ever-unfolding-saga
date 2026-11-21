# Changelog

This file records completed work in the current unreleased development line. Older releases are preserved in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md), while unfinished work remains in [ROADMAP.md](ROADMAP.md). The concise, manually authored summary for the release being prepared or most recently published belongs in [RELEASE-NOTES.md](RELEASE-NOTES.md).

## Unreleased — v0.53.x

### v0.53.7: Bounded depression and spill analysis

- Added deterministic closed-basin resolution by tracing the existing coarse watershed to an inland sink, then finding its lowest local spill level with a minimax flood over a fixed 17 × 17 coarse-cell window.
- Exposed basin-floor elevation, local spill elevation, depression depth, and the connected below-spill footprint; disconnected low terrain is excluded.
- Kept the result renderer-independent, bounded, and derived on demand. Ocean outlets, unresolved traces, and underground strata are excluded. This does not yet place persistent lakes or simulate precipitation, discharge, or water volume.
- Added automated coverage for closed-sink resolution, deterministic sampling, lowest-saddle selection, connected basin footprints, negative coordinates, region ownership, and flat-window behavior.
- Bumped Rust and Android version metadata to 0.53.7.

### v0.53.6: Bounded river-channel extraction

- Added deterministic reverse traversal of the coarse watershed graph to estimate contributing area for each queried drainage cell.
- Capped each query at 128 contributing coarse cells and exposed truncation explicitly; truncated results are lower bounds rather than fabricated complete basin totals.
- Classified confirmed river channels at eight contributing coarse cells, while keeping below-threshold truncated results inconclusive.
- Kept hydrology renderer-independent, on-demand, and free of persisted state or save-format changes. Added automated coverage for determinism, bounds, classification thresholds, truncation, negative coordinates, region ownership, and underground rejection.
- Bumped Rust and Android version metadata to 0.53.6.

### v0.53.5: Hierarchical watershed foundation

- Added deterministic coarse drainage cells on a 4,096-sample global lattice, with downstream links to the lowest strictly lower neighboring cell and stable signed-coordinate tie-breaking.
- Added local D8 drainage direction within each coarse cell, keeping local minima distinct from the larger drainage network.
- Added bounded coarse-network outlet tracing that distinguishes ocean outlets, closed inland basins, and explicit search-limit continuations.
- Kept watershed sampling renderer-independent, on-demand, fixed-point-terrain-backed, and independent of region visitation order. No save-format or persistence changes.
- Added automated coverage for deterministic drainage, downhill coarse links, local direction constraints, outlet classification, negative coordinates, region boundaries, and underground rejection.
- Bumped Rust and Android version metadata to 0.53.5.

### v0.53.4: Explicit shoreline data

- Added a deterministic shoreline sampler exposing opposite-terrain adjacency for both land and water samples.
- Added an eight-direction mask for shoreline orientation, derived from neighboring global elevation samples and consistent across region chunk boundaries.
- Reused the shared shoreline sampler for coastal biome classification instead of duplicating neighbor checks.
- Added automated coverage for land/water shoreline classification, direction masks, deterministic sampling, negative coordinates, region-boundary continuity, and underground rejection.
- Bumped Rust and Android version metadata to 0.53.4.


### v0.53.3: Surface climate and biome generation

- Added deterministic regional temperature and moisture fields on the shared global macro-coordinate lattice. Elevation cools high ground, and nearby ocean samples contribute bounded maritime moisture influence.
- Derived precipitation potential from moisture and temperature, and exposed signed/unsigned Q16 climate values with renderer-independent biome classification.
- Added the initial surface biome catalogue: ocean, coast, desert, grassland, temperate forest, taiga, tundra, and alpine. Coast classification uses adjacent ocean elevation samples; underground strata remain excluded.
- Added regression coverage for deterministic sampling, environmental variation, elevation cooling, ocean moisture influence, biome thresholds, region-edge coast continuity, and underground rejection.
- Bumped Rust and Android version metadata to 0.53.3.


### v0.53.2: Continuous surface elevation foundation

- Added a renderer-independent surface elevation sampler on the shared global macro-coordinate lattice.
- Generated continent-scale landmasses with regional and fine-scale variation for bays, peninsulas, and smaller islands. Elevation uses fixed-point interpolation so land/ocean classification does not depend on platform floating-point behavior.
- Sampling is deterministic from world seed, geography ruleset version, global coordinates, and independent noise channels. Negative coordinates and region boundaries use the same global field.
- Kept underground strata explicitly outside the surface sampler; explicit coast/watershed data, rivers, mountains, lakes, terrain-backed locations, and legacy-world embedding remain future work.

### v0.53.1: Macro-geography coordinate foundation

- Added renderer-independent signed region-chunk and global macro-sample coordinate types. Regions occupy 256 × 256 macro samples, with checked coordinate conversion and correct Euclidean partitioning for negative coordinates.
- Added deterministic coordinate-derived sample seeds keyed by world seed, geography ruleset version, and global coordinate; the result is independent of generation order and runtime entity IDs.
- Persisted optional region coordinates and geography ruleset metadata. New generated worlds receive stable region coordinates; legacy saves default to unembedded geography (`coordinate = None`, ruleset version `0`) for the planned explicit compatibility migration.
- Bumped the save format to v6 and added v5 backward-compatibility coverage. Terrain fields, continental features, legacy embedding, and physical location/route placement remain future work.


### v0.53.0: Persistent world route identities

- Added persistent `WorldRoute` records with stable string IDs, canonical endpoint ordering, and explicit bidirectional, A-to-B, or B-to-A travel direction. `World.routes` is the sole persisted authority for connectivity; location exit lists are rebuilt as a transient compatibility view.
- Added deterministic migration of legacy location adjacency into route records. IDs derive from saved endpoint IDs, one-way edges remain one-way, and the migration does not allocate runtime entity IDs or change `World.next_id`.
- Bumped the save format to v5. Legacy local-map passages retain their original IDs and positions while receiving matching route IDs; new route connections are reconciled into existing maps without regenerating terrain or relocating established passages.
- Linked physical passage interactions to their exact route identities, allowed distinct routes to share the same destination, and updated normal travel, fast-travel pathfinding, the interim World Map, campaign seeding, procedural generation, and World Lab route authoring to use route records.
- Added regression coverage for legacy save migration, one-way travel, route-linked passages, multiple routes to one destination, numeric ordering in the destination-only compatibility helper, compass-side passage allocation, and idempotent map reconciliation. New passages cycle east, west, north, and south before reusing a side; authored gate layouts keep north/south passages beyond the threshold. World Lab coverage compares stable route identities and destinations rather than the lossy legacy adjacency cache.
- Bumped Rust and Android version metadata to 0.53.0.

### Release and workflow maintenance

- Added a manually maintained `RELEASE-NOTES.md` for concise, version-specific GitHub Release summaries. CI/CD validates its version heading and uses its title and body instead of extracting the large development changelog.
- Removed automatic workflow-run cleanup from GitHub Actions. The standalone cleanup script now targets a specified release, previews deletions by default, requires `--apply` to delete, retries transient API errors, and preserves the selected release runs and active runs.
