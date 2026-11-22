# Changelog

This file records completed work in the current unreleased development line. Older releases are preserved in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md), while unfinished work remains in [ROADMAP.md](ROADMAP.md). The concise, manually authored summary for the release being prepared or most recently published belongs in [RELEASE-NOTES.md](RELEASE-NOTES.md).

## Unreleased — v0.53.x

### v0.53.22: Environmental location placement archetypes

- Added optional authored `settlement`, `wilderness`, and `landmark` archetypes to location content. Older content defaults to wilderness.
- Settlement placement favors flatter land and nearby classified rivers or inland lakes; landmark placement favors elevation and mountain-ridge character; wilderness placement retains broad environmental flexibility.
- Land and footprint validity remain primary, archetypes are preferences rather than hard biome constraints, and explicit authored anchors remain authoritative.
- Advanced the location-geography embedding version. Existing persisted anchors are not moved; older worlds without anchors use the new policy when embedded.
- Added automated coverage for archetype scoring and legacy content defaults.
- Bumped Rust and Android metadata to 0.53.22.

### v0.53.21: Provisional spatial material palette

- Added distinct, restrained material tints for existing floor, ash, rubble, and wall tiles, plus stone pillars and wooden tables.
- Preserved discovery shading while allowing discovered surfaces and props to retain their material identity.
- Kept route-connection and actor tints unchanged. The palette is renderer-only and does not add biome or simulation state.
- Bumped Rust and Android version metadata to 0.53.21.

### v0.53.20: Route connection placement and semantic presentation

- New ordinary route and frontier passages now prefer the middle of their selected map edge, expanding outward for additional connections instead of starting at a corner. Existing saved passage positions and IDs are never moved.
- Route allocation remains deterministic and continues to honor macro-geography heading preferences, authored gate thresholds, and fallback compass sides.
- Added renderer-neutral semantic categories for frontiers, trails, mountain passes, river/water crossings, mixed or uncertain routes, unclassified legacy routes, cave entrances, vertical shafts, and horizontal cave passages.
- The spatial renderer now applies restrained provisional tints to those categories. These are interim distinctions, not substitutes for dedicated assets; the remaining art requirements are recorded in [#491](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/491).
- Advanced the local-map generator version to 7. This does not regenerate established maps; newly introduced passages use the improved placement while old passage identities and positions remain intact.
- Added automated coverage for centered route/frontier placement and semantic presentation classification.
- Bumped Rust and Android metadata to 0.53.20.

### v0.53.19: Physical cave transitions on local maps

- Added persistent cave-transition metadata to local-map passages, with backward-compatible defaults for maps saved before cave topology integration.
- Surface entrances, horizontal chamber passages, and vertical shafts now appear as reachable interior transitions. They are not misrepresented as overland map-edge exits.
- Cave transitions are discovered through the existing contextual interaction system. The selected connection is revalidated and materialized on use, then the same local passage is upgraded to the authoritative bidirectional world route.
- Existing maps are migrated in place on entry: deterministic cave transitions are added and the local-map generator version advances without discarding established terrain, discoveries, props, encounters, or route passages.
- Surface locations expose at most three nearby cave entrances to keep local maps readable. Cave chamber maps expose their immediate reciprocal horizontal and adjacent-stratum links.
- Added automated coverage for reachable interior placement, idempotent map reconciliation, legacy map-exit defaults, and pending cave interaction discovery.
- Bumped Rust and Android version metadata to 0.53.19.

### v0.53.18: Persistent individual cave chambers

- Added `geography::caves` to expose immediate cave connections from a persistent location: reciprocal horizontal chamber passages, surface entrances, and shafts between adjacent underground strata.
- Materializing a connection creates or reuses one cave chamber location and its owning region, using a stable key and macro anchor derived from the generated cave network rather than runtime entity IDs.
- Persisted each materialized connection as one bidirectional authoritative `WorldRoute`. Repeated materialization reuses the same destination and route; reverse traversal from a cave chamber resolves the existing surface endpoint instead of creating a duplicate.
- Kept cave routes separate from overland path geometry. The existing surface A* planner is not misapplied to underground coordinates; local-map physical transitions remain the next integration step.
- Added automated coverage for shaft materialization, stable chamber identity, horizontal connection discovery, and reciprocal route reuse.
- Bumped Rust and Android version metadata to 0.53.18.

### v0.53.17: Deterministic vertical cave connections

- Added surface-to-first-stratum entrance candidates only where the surface sample is land and the corresponding underground sample is open cave space.
- Added inter-stratum shaft candidates only where both immediately adjacent underground strata are open at the same global horizontal coordinate. Connections never skip layers.
- Each cave network cell exposes at most one stable connection per adjacent layer pair. Candidate selection is coordinate-seeded and independent of region visitation order or runtime entity IDs.
- Added bounded nearby queries, clamped to 25 cave cells, with no world scan, generated-state cache, or persisted cave geometry.
- Added automated coverage for surface/underground validity, adjacent-stratum invariants, deterministic identity, negative coordinates, and bounded queries.
- These are geography connection candidates, not yet authoritative world routes, generated cave locations, or local-map entrances. Those integrations remain follow-up work.

### v0.53.16: Deterministic on-demand frontier materialization

- Added stable cardinal frontier target identities derived from a source location's persisted geography key and adjacent region coordinates. Listing or displaying a frontier does not generate a region or location.
- Added deterministic, edge-biased land-anchor candidates for adjacent regions. Frontier materialization requires the bounded terrain-aware route planner to find a valid land path before changing world state.
- Materializing a frontier creates or reuses only its adjacent region and one destination location, then persists the canonical bidirectional world route and planned macro path. No neighboring regions are generated recursively.
- Added persisted pending frontier passage markers to local maps. The player can use a pending passage to materialize its destination; the same passage ID and position are then upgraded in place to the new authoritative route. Existing passages and local-map geometry remain unchanged.
- Repeated materialization reuses stable region/location/route records and does not depend on runtime allocator position. Older saves default the optional frontier passage field; no save-format or geography-ruleset bump is required.
- Added automated coverage for read-only frontier listing, deterministic identity, allocator-order independence, bounded materialization/idempotence, invalid frontier rejection, pending passage interactions, in-place passage upgrade, and legacy field defaults.

### v0.53.15: Terrain-aware route planning and physical passage profiles

- Added bounded deterministic A* route planning over the shared macro-geography lattice. Ocean terrain is impassable; elevation and mountain ridges contribute soft route costs.
- Persisted canonical route paths, planning status, total cost, broad route profiles, and explicit crossing records. River crossings require a ford or bridge, lake crossings require a ferry, unresolved river estimates require survey, and mountain ridges are marked as pass candidates.
- Generate route geography during location embedding and when adding routes to geography-backed worlds. Existing route plans are preserved; older saves default the optional route field and receive deterministic plans through the existing geography migration.
- Connected route geography to local-map passages. New passages inherit route profiles and prefer the compass side indicated by the macro path. Existing passage IDs, positions, and saved map geometry remain unchanged; missing profiles are backfilled.
- Kept cave-route generation deferred until explicit vertical entrances connect surface locations to underground strata. Frontier materialization also remains future work.
- Added automated coverage for deterministic bounded paths, land/end-point invariants, route-heading reversal, persistence defaults, route-plan idempotence, passage direction, and compatibility-preserving profile backfill.

### v0.53.14: Geography-backed locations and route corridor assessment

- Added persisted geography seeds, ruleset/embedding versions, and stable macro-geographic anchors/footprints for world locations.
- Added deterministic compatibility embedding for legacy regions without coordinates. Placement order follows saved route-graph structure with stable ID tie-breaking; existing coordinates and all established world identities remain unchanged.
- Derived a compatibility geography seed from saved immutable identities when a legacy world has no generation seed. The migration preserves IDs, next_id, routes, history, and saved local maps and is idempotent.
- Added optional authored exact surface anchors and footprint radii. Generated anchors are deterministic within their owning region and authored placement can preserve deliberate geography.
- Added bounded candidate-route corridor assessment with distinct ocean, lake, river, unresolved-water, and mountain-ridge signals. This is a terrain diagnostic, not a pathfinder or travel authorization.
- Added automated tests for embedding, deterministic placement, authored anchors, bounded footprints, and corridor assessment. No save-format or geography-ruleset bump was required.

### v0.53.13: Deterministic underground cave networks

- Added renderer-independent underground sampling for negative geography strata, with independent deterministic layers and modestly increasing chamber frequency at greater stratum indices.
- Generated deterministically jittered chamber nodes on a shared 64-sample global lattice. Neighboring nodes connect through reciprocal cardinal passages only when both chambers exist and their deterministic shared edge is open.
- Exposed solid rock, chamber, and passage classifications, stable chamber-center coordinates, and connected-neighbor masks for future route and local-map integration.
- Kept each query bounded to one node and at most four neighbors, with checked coordinate arithmetic and correct negative-coordinate behavior. No world-sized searches, mutable generation caches, persisted generated geometry, or save migration.
- Added automated coverage for deterministic sampling, independent strata, cave/solid distribution, reciprocal passages, region-boundary continuity, negative coordinates, and extreme-coordinate safety.
- Bumped Rust and Android version metadata to 0.53.13.

### v0.53.12: Route-facing mountain barriers

- Exposed mountain-ridge strength and a consistent mountain-barrier classification through the shared surface-terrain sample, avoiding duplicate mountain-field sampling by future route and location-placement systems.
- Defined ridge strength as a soft crossing-resistance signal: lower-strength corridors can guide candidate mountain-pass placement, while actual crossing requirements remain the responsibility of route type and local-map connections.
- Kept the change derived, deterministic, fixed-point, and backward-compatible with geography ruleset 1. No save-format or geography-ruleset change.
- Clarified that physical discharge calibration and seasonal water balance are not required for the current geography milestone; seasonal rainfall/drought consequences are deferred to time-driven world simulation.
- Added automated coverage for barrier-signal consistency, land-only mountain classification, underground rejection, and legacy-ruleset behavior.
- Bumped Rust and Android version metadata to 0.53.12.

### v0.53.11: Deterministic mountain ranges

- Added a deterministic, warped ridged-noise mountain field with broad connected belts and regional variation.
- Integrated bounded mountain uplift into the shared surface elevation sampler, so climate, watersheds, rivers, and inland depressions consume the same terrain.
- Kept generation fixed-point, stateless, renderer-independent, and region-order independent. Ocean and underground samples receive no uplift.
- Advanced geography ruleset to version 2 while preserving exact ruleset 1 elevation behavior for existing worlds. No save migration or persisted mountain state was introduced.
- Added automated coverage for deterministic mountain sampling, uplift integration, region-edge continuity, underground rejection, and legacy-ruleset compatibility.
- Bumped Rust and Android version metadata to 0.53.11.


### v0.53.10: Surface-water connectivity

- Added a renderer-independent hydrology sampler that identifies ocean, inland-lake, and river-channel cells using the existing geography layers.
- Exposed each river channel's immediate downstream coarse cell and classified the destination as ocean, lake, another river channel, lower-threshold land, an inconclusive truncated estimate, or a closed basin. Truncated below-threshold accumulation remains unresolved rather than being treated as dry land. Lake and ocean coverage take precedence at terminal water cells.
- Reused the existing strictly downhill watershed graph; no second routing simulation, persistent generated-water state, or physical discharge claim was introduced.
- Added automated coverage for terminal ocean/lake nodes, deterministic river links, downhill destination invariants, and underground rejection.
- Bumped Rust and Android version metadata to 0.53.10.

### v0.53.9: Deterministic inland lakes and waterbody state

- Extended depression samples with a compact connected-basin footprint and deterministic coarse-cell membership checks.
- Added inland-lake coverage derived from closed-basin geometry, basin-floor precipitation potential, and coarse-cell elevation. Water surfaces are bounded between basin floor and local spill saddle; this is a relative first-pass fill model, not calibrated hydrology.
- Added stable lake IDs derived from world seed, geography ruleset, and basin outlet, plus an initial river-inflow indicator using the existing outlet river-channel estimate.
- Added optional generic world-persisted waterbody modification records keyed by stable IDs. Generated geography remains derived on demand; old saves default to no modifications without a save-format version bump.
- Added automated coverage for connected basin membership, deterministic lake sampling and identity, underground rejection, region ownership, modification round-trips, and backward-compatible defaults.
- Kept the basin-count convenience helper test-only so strict Clippy checks do not flag production dead code.
- Bumped Rust and Android version metadata to 0.53.9.

### v0.53.8: Precipitation-weighted river accumulation

- Extended bounded reverse-watershed accumulation to sum each contributing coarse cell's deterministic climate precipitation-potential value alongside contributing area.
- Exposed the sum as an unsigned Q16 × coarse-cell relative index. It is not a physical discharge or water-volume estimate; existing river-channel classification remains area-based until flow thresholds can be calibrated against a fuller water model.
- Preserved the 128-contributor and 1,024-neighbor-candidate bounds. Truncated results expose lower-bound contributing counts and precipitation sums.
- Added automated coverage for deterministic weighted accumulation, upstream/downstream contribution inclusion, fixed-point bounds, ocean exclusion, negative coordinates, region ownership, and underground rejection.
- Kept hydrology renderer-independent, on-demand, and free of persisted state or save-format changes.
- Bumped Rust and Android version metadata to 0.53.8.

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
