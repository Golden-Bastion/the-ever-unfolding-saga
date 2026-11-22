# World Topology Compatibility Audit

> **Baseline note:** implementation observations below describe the pre-route v0.52.70 state. The v0.53.0 route-record implementation and its remaining boundaries are recorded in [Implementation follow-up](#implementation-follow-up--v0530) at the end of this document.


> Audit snapshot: `main` at `ea200cda3d8dfe3fe44d4c1ea52b3c47af722caa` (v0.52.70), 20 November 2026. This document records the current implementation before v0.53.x world-topology changes. It supplements, and does not replace, the settled [World Topology, Local Exploration, and Hybrid Travel](world-topology-and-travel.md) design.

## Purpose and limits

The v0.53.x topology work must extend the existing world/location graph and local spatial maps without invalidating established campaigns. This audit identifies the persistent identities and behaviors that a coordinate-backed world must preserve.

This change is documentation-only. It adds no coordinates, route entities, generator changes, save migrations, or gameplay behavior. Requirements already settled in the topology design remain authoritative; unresolved implementation choices are identified rather than silently decided here.

## Current implementation

### World and region identity

The authoritative world model is in `src/model.rs`:

- `World` stores a monotonic `next_id`, optional `WorldGenerationMetadata`, `regions`, `locations`, `history`, and persistent `local_maps`.
- `World::allocate_id()` assigns sequential numeric `EntityId` values. Region, location, character, history, NPC, faction, item, and other runtime entities share this allocator.
- `Region` currently contains a numeric runtime ID, name, description, and a list of numeric location IDs. It has no coordinate, footprint, terrain, or persistent generation key.
- `Location` contains a numeric runtime ID, name, description, numeric region ID, danger state, corpse IDs, a list of numeric exit IDs, and optional local-map layout metadata. It has no macro-geographic position, footprint, terrain constraints, or explicit route records.
- `WorldGenerationMetadata` is optional and currently stores only the seed and the original region/location/extra-edge counts. It has no world-generation ruleset version or stable generation-key information.

Runtime numeric IDs are persistent once written to a save, but their initial assignment is tied to the order in which the world and its entities are created. They are not the coordinate-derived identity described by the future topology design.

### New-world generation and authored content

Normal new-world creation is in `model.rs::create_new_state`. It generates a finite world using the default `WorldGenerationConfig`, stores the seed and configuration counts in `WorldGenerationMetadata`, then applies authored campaign content.

`procedural.rs::generate_world` currently:

1. Creates regions and locations in index order, assigning runtime IDs through `World::allocate_id()`.
2. Assigns locations to regions using their position in the generated location list.
3. Uses one sequential `DeterministicRng` stream for generated danger values and extra graph-edge selection.
4. Connects locations as a chain, then adds a configured number of random reciprocal edges.

The generator has tests proving that the same seed and configuration reproduce the same finite graph. It does not yet support independently generating a region from a coordinate, or prove that region generation is independent of frontier visit order. Changing the generator algorithm or the amount/order of random draws can change newly generated results; the stored seed and counts do not identify the algorithm that produced them.

Authored content is defined in `content/definitions.rs`:

- A campaign currently has one `RegionContent` plus a list of `LocationContent` entries.
- Region and location definitions have stable string IDs in content, and location exits refer to other authored location IDs.
- The authored schema has no regional coordinate, per-location region assignment, physical placement constraints, footprint, or route connection point.

Those content IDs do not currently survive as runtime identity fields. `Region` has no authored content ID, and `Location` has no authored content ID or procedural generation key. `content/seeding.rs::seed_world` creates/updates the runtime region and locates existing locations by display name. `procedural.rs::place_authored_content` sorts authored locations by stable content ID and maps them onto the first generated location slots; it anchors those locations to the first generated region and retains the generator's existing edges while adding authored links.

Thus authored IDs help select deterministic initial slots, but saved runtime worlds are subsequently addressed by numeric IDs and names, not by those authored IDs. Name changes and content-pack evolution need an explicit identity policy before authored geography is expanded.

### Graph routes and travel

The current `Location.exits` lists form the authoritative world-location graph.

- `game/actions.rs::has_reciprocal_location_connection` requires both endpoints to list one another. Normal `travel_to` rejects missing or non-reciprocal connections.
- A successful travel hop advances world time and the character turn, updates the character's location, records history, synchronizes quests, and runs arrival-event/threat handling.
- Fast travel finds a shortest path through reciprocal connections whose destinations are already considered visited. It invokes normal travel for every hop and stops when death, a threat, or a consequential arrival event interrupts the journey.
- Persistent local maps are the primary visited-location record. The fast-travel logic also has a bounded-history fallback for saves created before local maps were persisted.

The graph stores adjacency only. It has no first-class route ID, route type, terrain requirements, travel cost, direction policy, or physical connection-point metadata. Intentional one-way connections are not represented by the current normal travel contract.

### Local-map identity, passages, and persistence

Local maps are generated lazily when the player enters Explore. Existing maps are reused on re-entry and saved in `World.local_maps`; saved terrain, room geometry, props, discoveries, and other meaningful state are authoritative rather than regenerated when generator code changes.

There are three related identity contracts to preserve:

- `bevy_spatial.rs::enter` currently constructs the map key as `location.<runtime numeric location ID>`.
- The local-map seed is derived by `spatial.rs::local_map_seed` from the world seed and that map key. The world seed comes from `WorldGenerationMetadata` when present; otherwise the fallback is `World.id`.
- `MapExit` stores a local string ID, grid position, and numeric `target_location_id`. Its current ID is composed from the local-map key and target runtime location ID.

Comments around `LocalMap.location_id` and `local_map_seed` describe a stable location identity, but the supplied value is currently derived from the runtime numeric ID rather than an authored or coordinate-derived key. Changing that key affects map lookups and seeding, as well as encounter/discovery IDs derived from it and save compatibility.

For new maps, `spatial.rs::configure_map_exits` converts current graph neighbors into physical edge passages. It deduplicates destinations while preserving list order, then assigns exits around east, west, north, and south edges in that order (repeating sides for additional routes). Passage positions and IDs are a local-map representation of the graph, not an independent travel graph.

There is an important migration boundary in `bevy_spatial.rs::enter`: if a saved map has no exits, the current world graph is used to configure them. If the map already has one or more exits, that list is retained as-is. Consequently, adding a graph connection to a location whose saved map already has passages will not currently append a new physical passage to that map. Future topology changes need an explicit, tested reconciliation policy that preserves existing local geometry and meaningful passage state.

The local-map generator has its own `LOCAL_MAP_GENERATOR_VERSION` and stores the generator version and seed on each map. This version is separate from world/region generation; existing map geometry remains authoritative after creation.

### Save compatibility

The current outer save format is version 4 (`persistence.rs::SAVE_FILE_VERSION`). The loader deserializes the stored `GameState`, then calls `normalize_loaded_game`; that explicit normalization currently adjusts a small set of legacy character/time defaults for save versions below 2. It does not migrate world topology or infer geographic metadata.

Several topology-related fields are already optional/defaulted for older saves, including optional world-generation metadata, the world's local-map collection, optional location spatial-layout metadata, and local-map exits. Those defaults allow old worlds/maps to load without inventing new geography. They are compatibility contracts, not evidence that geography migration has already been implemented.

A topology migration must preserve at least:

- Existing world, region, and location runtime IDs, and the `next_id` allocator boundary.
- Existing graph adjacency and whether a connection is reciprocal.
- Existing world history, character location, inherited-world state, and visited-location semantics.
- Existing `World.local_maps`, map keys, terrain, rooms, doors, props, discoveries, encounter records, and passage state.
- Existing local-map seeds and generator versions. New world/region generation must not silently regenerate a saved local map.

## Gaps against the approved topology design

| Area | Current state | Required extension |
|---|---|---|
| Regional geography | Regions have no coordinates or shared landscape data. | Stable signed world coordinates and boundary-compatible macro-geography. |
| Location placement | Locations belong to a region but have no physical anchor or footprint. | Stable location placement/footprints constrained by regional geography. |
| Generation identity | One finite sequential generator; no world ruleset version or coordinate generation key. | Versioned, order-independent generation from world seed and stable coordinate/key inputs. |
| Authored identity | String content IDs are not retained in runtime region/location records. | Preserve authored IDs and distinguish them from runtime numeric entity IDs. |
| Routes | Numeric adjacency lists; reciprocal graph links are required. | Stable route identity and explicit physical route/connection metadata, including intentional one-way policy if supported. |
| Local passages | IDs and targets derive from map/runtime location IDs; passage positions derive from neighbor list order. | Stable route-to-local-exit mapping with a non-destructive policy for established maps. |
| Frontier expansion | No coordinate-addressable on-demand region creation. | Idempotent frontier materialization that does not reroll established regions, locations, routes, or maps. |
| Legacy saves | Existing graph/local maps load; no geography migration exists. | An explicit deterministic compatibility strategy that preserves old topology and local-map state. |
| Map presentation | Interim graph layout is presentation-only. | Continue deriving future geographic presentation from authoritative world geography and routes; do not persist graph-layout pixels. |

## Compatibility invariants for implementation

1. **Never regenerate an established world to migrate it.** New metadata must attach to the existing saved world without replacing runtime IDs, graph edges, or established authored/procedural content.
2. **Keep runtime entity IDs and generation keys separate.** Runtime IDs remain useful for current gameplay references. New region/location/route generation must use stable authored IDs or coordinate-derived keys, not the order in which `next_id` is consumed.
3. **Make generation rules explicit and versioned.** New deterministic region/terrain sampling must include a ruleset version. Independent region requests must produce identical shared-boundary values regardless of request order.
4. **Keep local maps authoritative.** New topology may add a migration for passages, but must not regenerate saved terrain, move established POIs/actors/props, or discard discoveries.
5. **Keep the graph authoritative for travel.** A displayed route, map line, or local passage must never create a connection by itself. Physical exits and fast travel must validate against the same topology source.
6. **Make expansion idempotent.** Reaching the same frontier more than once must resolve to the same stable region/location/route identities and must not duplicate content or consume unrelated random streams.
7. **Preserve character/world ownership.** Regional geography, established routes, and physical discoveries remain world-owned; exact live character position/session state remains character-owned.

## Recommended implementation order

The sequence in the approved topology design remains appropriate. These substeps make its compatibility prerequisites explicit:

1. **Finalize geographic identity and legacy embedding.** Define stable coordinate-backed region/location keys and how worlds without generation metadata/geography receive their persisted compatibility embedding. Existing runtime IDs and route identities remain stable; do not assign geographic identity from display names or rerun the old generator.
2. **Version the generation contract.** Add a versioned ruleset/key contract before using coordinates to derive generated terrain or locations. Keep the current finite-generator behavior for existing saved worlds.
3. **Introduce regional coordinate/geography primitives.** Establish signed coordinate conventions, region extent/resolution, shared-boundary sampling, and deterministic test oracles before implementing a large terrain field.
4. **Anchor locations and connect routes.** Add authored/generated location placement constraints and define how existing stable route endpoints map to geographic connection points and local `MapExit` records.
5. **Implement frontier materialization.** Ensure a frontier region and its connecting route are created once, before presenting the route as usable. Generate only what is required to resolve that frontier.
6. **Extend geographic passage reconciliation.** Define terrain-aware connection-point placement for the existing stable routes and local passages while preserving established passage identities, positions, and local modifications.
7. **Integrate known-route fast travel and persistence.** Reuse the existing travel action and interruption semantics. Add conservative migrations and inheritance coverage.
8. **Replace interim map geography only after authoritative geography exists.** Current graph layout remains presentation-only until it can be derived from actual world data.

## Owner-selected legacy-world policy

On 20 November 2026, the project owner selected deterministic embedding of existing worlds into the coordinate-backed topology. Existing worlds are to be migrated into the new system rather than retained as a separate topology-only mode.

The migration contract is:

- Assign coordinates to existing regions and macro-geographic anchors to existing locations through a deterministic embedding based on saved region/location identities and graph structure.
- Use stable runtime IDs and deterministic tie-breaking; do not derive placements from mutable display names, current content ordering, or fresh random draws.
- Persist assigned coordinates, compatibility-seed/provenance metadata, and the embedding version so loading or repeating migration cannot move established places.
- Preserve runtime IDs, `next_id`, existing graph adjacency (including whether edges are reciprocal), history, character location, and every saved local map and its contents.
- Treat embedded coordinates and any synthesized surrounding terrain as an explicitly marked compatibility layer, not recovered historical geography. New geography must accommodate established route endpoints and saved local maps rather than relocating them.

This resolves whether legacy worlds should be embedded or left behind. The exact embedding algorithm, coordinate schema, and deterministic compatibility-seed derivation still need to be designed and tested before the save migration is implemented. In particular, geography scale and route geometry must be chosen so the embedding can satisfy the existing graph without pretending that old route layouts were authored against terrain that did not yet exist.

## Design decisions still open for coordinate-backed topology

- **Authored placement contract (partially resolved in v0.53.14):** `LocationContent` may provide an exact surface anchor and footprint radius, and persisted location-geography records retain authored content IDs as stable keys. Environmental archetypes now provide settlement, wilderness, and landmark placement preferences; region-level placement constraints and local-map connection-point requirements remain open.
- **Geography scale (resolved 21 November 2026):** each region is a signed chunk of 256 × 256 macro samples. `RegionCoordinate` identifies chunks and `MacroCoordinate` identifies positions in the shared global sample lattice; both include a vertical stratum index, with zero for the surface and negative values for underground layers. This is separate from local-map tile dimensions; continuous terrain sampling/interpolation remains future implementation work.
- **Geographic route-to-passage contract (updated in v0.53.16):** World routes persist bounded terrain-aware macro paths, broad profiles, and explicit crossing requirements. Established local-map passages inherit route profiles and prefer the compass heading indicated by the macro path. Pending cardinal frontier passages persist a stable target marker but no graph endpoint; using one creates or reuses one adjacent region and destination location after a valid land path is found, then converts the exact passage in place to its authoritative route. Existing passage identities, positions, and map geometry remain unchanged. Richer local connection-point placement, infrastructure construction/enforcement, and vertical cave entrances remain open.

## Focused validation required by later implementation stages

- Same seed, ruleset, and coordinate produce identical region data regardless of the order in which neighboring regions are requested.
- Adjacent regions produce compatible shared-boundary terrain/features when generated independently and in either order.
- Repeated frontier materialization resolves to the same stable IDs and does not duplicate regions, locations, routes, authored content, or encounters.
- Changing generator ruleset versions affects only new/unestablished geography; persisted region/location/route data remains authoritative.
- Existing save fixtures with `generation = None`, generated-world metadata, empty local-map lists, and populated local maps preserve their world graph and saved local geometry through migration and round-trip.
- Every physical map exit resolves to an authoritative route and valid destination; required routes expose compatible physical passages at both ends when route rules require reciprocity.
- Adding a route to an explored location preserves prior passage IDs, positions, terrain edits, POIs, discoveries, and actor/encounter state.
- Fast travel continues to use known routes and the normal per-hop travel action, including time, event, threat, quest, death, and encounter interruption behavior.

## Implementation references

- `src/model.rs` — `World`, `Region`, `Location`, `WorldGenerationMetadata`, numeric ID allocation, new-world and inherited-world creation.
- `src/procedural.rs` — finite world generation and authored-location slot mapping.
- `src/content/definitions.rs` and `src/content/seeding.rs` — authored region/location IDs and content-to-runtime seeding.
- `src/game/actions.rs` — route-direction validation, visited-location rules, fast-travel pathfinding, and per-hop travel behavior.
- `src/spatial.rs` — local-map generation, generator version, map exits, and stable map-local structures.
- `src/bevy_spatial.rs` — lazy map creation, current runtime-derived map keys, graph-to-passage synchronization, and map persistence.
- `src/persistence.rs` — save version, serde deserialization, and current legacy normalization.
- `docs/systems/world-map.md` — interim topological map and presentation-only layout contract.

## Implementation follow-up — v0.53.0

The first route-identity layer was implemented after this audit was merged.

- `World.routes` now stores persistent `WorldRoute` records with stable IDs, canonical endpoint ordering, and explicit bidirectional/A-to-B/B-to-A direction. Multiple route IDs may join the same endpoint pair.
- `Location.exits` is a transient, non-persisted adjacency projection rebuilt from the route records. New save data stores routes rather than a second copy of the graph.
- Save format v5 migrates version-4 and earlier adjacency data deterministically to `route.compat.legacy.{lower_id}.{higher_id}` IDs. Direction is inferred from the original outgoing adjacency, and the migration does not allocate entity IDs or change `next_id`.
- Existing local-map passages receive a matching route ID while preserving their existing passage ID and position. New and additional routes are appended through idempotent reconciliation; saved terrain and established passage geometry are not regenerated or moved.
- Travel, known-route fast-travel traversal, World Map route views, procedural generation, authored seeding, and World Lab route editing now use persistent route records. Physical passage interactions validate the passage's specific route ID.

This layer intentionally does not yet define route type/profile, terrain constraints, macro-geographic route geometry, distance, travel cost, or route discovery state. Those remain dependent on the coordinate-backed world topology design and are not implied by this implementation.


## Implementation follow-up — macro-geography coordinate foundation

The first coordinate-contract stage is implemented on the v0.53.x development line.

- Added signed `RegionCoordinate` and `MacroCoordinate` value types with checked region-to-global conversion and Euclidean global-to-region partitioning, including negative coordinates.
- Fixed the initial region chunk resolution at 256 × 256 macro samples.
- Added stable coordinate-derived sample seeds keyed by world seed, geography ruleset version, and global sample coordinate. Sampling order and runtime entity allocation do not affect the result.
- New generated worlds persist distinct coordinates for their initial regions. Direct authored seeding into an otherwise empty world anchors its first region at the origin.
- Added optional region coordinates and a geography ruleset marker to persistent world data. Pre-geography saves deserialize with unassigned coordinates and ruleset version zero; the deterministic legacy-world embedding remains a separate migration stage.
- Subsequent v0.53.x work added the continuous surface/underground geography fields, deterministic legacy embedding, persistent location anchors/footprints, bounded terrain-aware route paths, crossing profiles, route-aligned local-map passages, and idempotent on-demand frontier materialization. Vertical cave entrances/inter-stratum routes and richer local connection-point placement remain outstanding.


## Implementation follow-up — location geography and corridor assessment (v0.53.14)

- Added world-persisted geography seed, ruleset/embedding versions, and location-geography records. These are defaulted for old saves; the outer save format does not change.
- Legacy worlds without a geography seed receive a deterministic compatibility seed derived from saved world, region, location, and route identities. Names and mutable content are excluded. Existing generation seeds and assigned region coordinates are preserved.
- Missing region coordinates are assigned in stable route-graph breadth-first order, with stable numeric-ID tie-breaking and collision avoidance. Existing coordinates are not moved.
- Every location without a saved geography record receives a deterministic macro anchor within its owning region chunk and a footprint radius validated against surrounding land samples. Authored content may provide an exact surface anchor and footprint radius. Existing anchors always take precedence.
- Added a bounded straight-line corridor assessment for route candidates. It exposes separate ocean, lake, river, unresolved-water, and mountain-ridge signals. This is not yet a route pathfinder and does not change authoritative route connectivity or travel permission.
- The migration is idempotent and does not allocate entity IDs or mutate route records, history, or saved local maps.

## Implementation follow-up — terrain-aware route planning and passage profiles (v0.53.15)

- Added deterministic bounded A* planning between persisted location anchors on the shared macro lattice. Ocean terrain is impassable; elevation and mountain-ridge strength contribute soft costs. Search, endpoint distance, grid spacing, and persisted path length are explicitly bounded.
- Persisted each route's canonical endpoint-A-to-endpoint-B path, planning status, total cost, broad route profile, and detected crossing requirements. Confirmed rivers require a ford-or-bridge solution, lakes require a ferry, unresolved river estimates require survey, and mountain ridges are marked as pass candidates.
- New local-map passages inherit the route profile and prefer the compass side indicated by the macro path. Existing passage IDs, positions, terrain, discoveries, and other saved map state are preserved. Missing passage profiles are backfilled during reconciliation.
- Older saves default the optional route-geography and map-passage profile fields. Existing route graphs remain authoritative; this change does not remove graph connectivity or claim that required crossing infrastructure has already been constructed.
- Ocean ferry routes and cave routes are not generated by this overland planner. Cave connections require explicit vertical entrances; frontier materialization and richer local-map connection-point constraints remain future work.


## Implementation follow-up — on-demand frontier materialization (v0.53.16)

- Added read-only cardinal frontier discovery for geography-backed surface locations. Target IDs derive from the persisted source stable key and exact source/target region coordinates.
- Generated local maps can retain pending frontier passages with no target location or route ID. They are offered as contextual spatial interactions, not as already-established graph connections.
- Using a pending passage validates its identity against the source location/region, deterministically searches edge-biased land anchors in the adjacent region, and requires the bounded route planner to find a valid overland path before mutating world state.
- Materialization creates a missing adjacent region and one destination location, or reuses the already-existing adjacent region. It persists the destination's geography record and a bidirectional route with canonical macro path/profile. It does not generate any other region or synthesize full settlement content.
- The source passage is upgraded to the route in place, preserving its local ID and position. The destination map receives the reciprocal passage through normal route reconciliation when entered. Repeated requests return the existing destination/route without consuming new IDs or rerolling geography.
- The optional pending-frontier field defaults absent in older saves. The outer save format and macro-geography ruleset do not change.


## Follow-up: cave map transition migration (v0.53.19)

The cave-topology integration extends the local `MapExit` record with an optional serialized `CaveConnectionTarget`. The field defaults to absent, so older save data remains readable without a save-format bump. Ordinary overland and frontier exits remain edge passages; cave entrances, horizontal chamber links, and vertical shafts use interior positions.

On entry, local-map generator version 6 reconciliation adds missing cave transitions to the existing saved map. Placement is deterministic and restricted to walkable interior cells reachable from the map entry; it avoids existing exits, points of interest, actors, encounters, and prop footprints. Existing terrain, discoveries, props, encounters, actor placements, and established passage IDs/positions are retained. A cave destination and authoritative route are materialized only when the player uses the passage; the pending passage is upgraded in place. This is an additive map migration, not a full layout regeneration.


## Follow-up: route connection placement and presentation (v0.53.20)

The local-map reconciliation work now appends newly discovered graph routes to established maps without regenerating their terrain or relocating existing passages. New ordinary route and frontier passages prefer the center of their chosen map edge, then distribute additional connections outward. The placement order still follows the persisted macro route heading when available and respects authored gate thresholds. The local-map generator version is now 7; this records the changed placement policy but does not trigger full map regeneration.

`MapExit::visual_kind` derives semantic presentation from the route profile, pending frontier target, or cave connection kind. The Bevy renderer currently applies restrained category tints using the existing generic passage image. These are provisional visual distinctions only. Dedicated artwork for trails/roads, mountain passes, river and water crossings, gates, bridges, docks, cave entrances, shafts, and chamber passages is tracked in [#491](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/491). Route identity, reachability, and interaction remain owned by the world graph and renderer-neutral spatial model.
