# World Topology Compatibility Audit

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

1. **Settle identity and legacy-world policy.** Define stable region, location, and route keys; define how worlds without generation metadata/geography will be represented. Do not assign identity from display names or rerun the old generator.
2. **Version the generation contract.** Add a versioned ruleset/key contract before using coordinates to derive generated terrain or locations. Keep the current finite-generator behavior for existing saved worlds.
3. **Introduce regional coordinate/geography primitives.** Establish signed coordinate conventions, region extent/resolution, shared-boundary sampling, and deterministic test oracles before implementing a large terrain field.
4. **Anchor locations and routes.** Add authored/generated location placement constraints and stable route endpoints; define how route edges map to local `MapExit` records.
5. **Implement frontier materialization.** Ensure a frontier region and its connecting route are created once, before presenting the route as usable. Generate only what is required to resolve that frontier.
6. **Reconcile saved local passages safely.** Specify how new/changed route metadata updates existing maps without shifting old passage identity or discarding local modifications.
7. **Integrate known-route fast travel and persistence.** Reuse the existing travel action and interruption semantics. Add conservative migrations and inheritance coverage.
8. **Replace interim map geography only after authoritative geography exists.** Current graph layout remains presentation-only until it can be derived from actual world data.

## Decisions still required before persistent schema work

These choices are not settled by the current code or by the high-level topology direction:

- **Legacy-world representation:** Should old finite worlds receive a deterministic compatibility embedding into region coordinates, or remain explicitly marked as legacy/topology-only while new coordinate-backed regions are introduced through controlled anchors? Either path must preserve the existing graph and local maps; assigning physical terrain to unknown old places must not be presented as recovered historical geography.
- **Stable route model:** Should a route be a separate persistent record with its own ID/endpoints, or a stable-ID-bearing edge owned by the world graph? The model must support reciprocal routes now and make intentional one-way routes explicit rather than accidental.
- **Authored placement contract:** Which placement/footprint/connection-point constraints belong on `RegionContent` and `LocationContent`, and how are authored IDs retained in saved runtime entities when display names change?
- **Geography scale:** What are region dimensions and macro-geography sampling resolution? The design deliberately leaves universal local tile sizes and continuous global terrain implementation details open.
- **Passage migration policy:** When a route is added/changed after a map has been explored, should migration add a passage to unused space, use a stable authored connection point, or require an explicit map revision? Existing passage positions/state must not silently shift.

The recommended default is to keep legacy worlds playable as legacy topology until the owner approves an explicit, non-destructive mapping policy. Do not fabricate coordinates or backfill physical terrain solely to make the eventual map look geographic.

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
- `src/game/actions.rs` — reciprocal graph validation, visited-location rules, fast-travel pathfinding, and per-hop travel behavior.
- `src/spatial.rs` — local-map generation, generator version, map exits, and stable map-local structures.
- `src/bevy_spatial.rs` — lazy map creation, current runtime-derived map keys, graph-to-passage synchronization, and map persistence.
- `src/persistence.rs` — save version, serde deserialization, and current legacy normalization.
- `docs/systems/world-map.md` — interim topological map and presentation-only layout contract.
