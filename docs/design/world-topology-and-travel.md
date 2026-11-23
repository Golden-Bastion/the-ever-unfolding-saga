# World Topology, Local Exploration, and Hybrid Travel

> Design record migrated from [GitHub issue #365](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/365). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

The design direction is settled and its core topology/geography foundations are being implemented incrementally. This document records the world-topology and travel decisions, constraints, and intentionally open choices.

The design extends the existing graph-based world and local spatial-exploration prototype. It does not require a seamless global terrain map, and it does not replace existing location-level gameplay actions.

## Goal

Create a persistent, extensible world in which the player can optionally explore every location spatially, travel between locations by physically using map exits where appropriate, and use fast travel on established routes when the intervening journey does not need to be played.

The world should be able to expand beyond its initial frontier without generating or simulating every possible region up front.

The spatial scale must remain meaningful: outdoor areas should take several minutes to cross, locations may contain multiple connected local areas, and regions must represent far more distance than a single local map. See [World Scale and Travel](world-scale-and-travel.md) for the scale hierarchy and implementation constraints. The current 64×48 local-map default is only an initial increase, not the final target.

## Settled design decisions

### 1. Two levels of geography

The world uses two complementary structures:

- **Regional layer:** a potentially very large set of regions identified by stable world coordinates. Regions establish broad geography and contain world-level locations and routes.
- **Location layer:** a graph of settlements, wilderness areas, ruins, landmarks, and other travel destinations. Explicit graph connections determine which locations can be reached from which others.

A region is not a continuous tile map. Regions and locations are connected through the world graph. Intermediate locations may connect major destinations, as in area-based exploration games where travel occurs through linked maps.

The global world does not need one seamless terrain surface. Each location has its own local map, which is loaded as a separate spatial area when explored.

### 2. Explore is an optional action at every location

Every accessible world location should offer an **Explore** action and a corresponding local spatial area.

- Exploration is optional. Existing textual/location-level actions such as talking, using inventory, meditating, managing quests, and interacting with known services remain directly available without walking around a map.
- Safe locations are not excluded from spatial exploration. Settlements and other safe sites may have useful local geography, exits, landmarks, and points of interest, while still retaining their direct textual actions.
- Local maps may be authored or procedurally generated, and their scale/detail can vary by location. A small or simple location does not need the same complexity as a wilderness region or major settlement.
- A location's map should be generated lazily when needed, not by rendering or fully generating every location at world creation.
- Once established, a local map and its meaningful state must persist rather than silently rerolling on re-entry.

### 3. Spatial travel between locations

Local maps can provide physical route exits such as roads, gates, mountain passes, bridges, docks, cave passages, and trailheads.

- A physical exit represents a connection in the authoritative world-location graph; the graph and map must not disagree about which destination is reachable.
- Reaching and using an exit transitions to the connected destination's separate local map. This is an area transition, not a seamless traversal of all terrain between the two locations.
- Some routes may pass through intermediate world locations. Those locations are real places that can be discovered and explored, not merely decorative sections of a loading screen.
- Routes to ungenerated frontier regions can be represented as stable frontier exits. Using one creates the destination region and connects it to the already established world.
- A route must not become a one-way dead end by accident. Intentional one-way transitions must be explicit; otherwise the graph connection and its physical exits should be reciprocal.
- Map exits should use stable route/destination identities rather than depending on arbitrary tile coordinates or display names alone.

Walking to a physical exit should be the natural way to discover new routes, approach dangerous or authored travel situations, and explore places along a journey.

#### Lake ferry transitions

A route that crosses an inland lake should expose a physical ferry landing at the route passage in the current location. The player boards there, enters a distinct crossing scene, and arrives at the opposite shore before the connected destination location loads. Treat this as a short authored-feeling cutscene rather than a second explorable map or a seamless global-water simulation. When a route crosses multiple lakes, present the crossings in travel order.

The crossing scene consumes the exact persisted lake identity and shore pair. It must not invent a dock in an endpoint local map when the macro crossing lies elsewhere, and it must not fabricate shore positions when geography cannot resolve both banks. Provisional colored shapes are acceptable until dedicated landing, boat, and water artwork exists. The sequence is transient presentation; world-route identity and travel remain authoritative. Known-route fast travel can continue to skip the local cutscene.

### 4. Keep fast travel alongside spatial travel

The design retains fast travel as a convenience, rather than forcing players to physically repeat every journey.

- Fast travel uses known, established routes. It cannot reveal or bypass an unknown frontier connection.
- By default, fast travel should be available for routes the character has discovered, provided the character is not in active combat, a blocking encounter, or another state that requires direct play.
- Fast travel consumes appropriate world time and invokes the route-level travel/event rules. It must not silently pause the world's consequences just because the player skipped the local map.
- Fast travel may skip intermediate locations only when those locations and the route are already known. Unknown or designated significant route segments can require spatial travel.
- Specific routes may disable fast travel when a quest, encounter, authored sequence, or gameplay rule depends on travelling through them directly.
- The normal travel interface should communicate when direct spatial travel is required and why fast travel is unavailable.

The exact travel-time values and the conditions for travel events belong to the travel-system design and balancing work; they are not fixed here.

### 5. On-demand regional expansion

The regional coordinate model enables broad expansion without generating the whole world in advance.

- Use stable signed coordinates for regions relative to the world's origin.
- Derive untouched region characteristics from the world seed, region coordinates, and an explicit generation-ruleset version. Region generation should not depend on the order in which other regions were visited.
- Generate a frontier destination only when it is actually reached or otherwise deliberately created by authored content.
- Do not recursively generate every neighboring region when one region is created.
- Each new region should have enough coherent structure to connect into the world graph and offer valid locations/routes before it is presented as an explorable destination.
- Existing region coordinates, generated locations, graph edges, authored anchors, and established local maps are persistent world state; they must not be overwritten by regeneration.
- Separate generation keys from runtime numeric entity IDs. Random choices should derive from stable keys, not from assumptions about the order in which global IDs were allocated.

The initial frontier materialization implementation follows this contract: opening a local map may present pending cardinal frontier passages, but the invitation has no world destination or route yet. Only using it materializes one adjacent region (when absent), one destination location, and the connecting route. The destination anchor is selected deterministically near the shared edge and accepted only when the existing bounded overland planner finds a valid land path. The pending map passage is converted in place to the authoritative route, retaining its local ID and edge position. Reaching the frontier a second time resolves to the same persisted records, and no neighboring regions are recursively generated. Existing local maps remain authoritative. This stage does not yet generate richer frontier settlement content or detailed local connection-point geometry.

“Effectively unbounded” means the design supports continued expansion within practical coordinate, storage, and performance limits. It is not a requirement to simulate an infinite world or generate the entire surrounding world.

### 6. Persistence and inheritance

The world topology and world-persistent discoveries survive character death.

World-persistent state includes generated region/location identities, established travel connections, discovered routes, important local-map changes, and meaningful discoveries. Character-specific exploration session state—such as the dead character's exact position or temporary activity—does not automatically transfer to the next character.

A successor should be able to revisit locations and routes and encounter the persistent consequences of prior lives. Existing save files and established worlds should retain their current locations, exits, history, and local-map data through conservative migration.

**Legacy-world embedding decision (20 November 2026):** existing worlds must be brought into the coordinate-backed topology through a deterministic, one-time compatibility embedding. They will not remain on a separate topology-only path. The embedding must derive placements from the saved region/location identities and graph structure, use stable ID-based tie-breaking, and persist the result with migration provenance. It must preserve all existing runtime IDs, `next_id`, graph edges, world history, and saved local maps. This is a compatibility placement, not a claim that the generated coordinates recover the world's original physical geography. New terrain/geography synthesized around those placements must be explicitly understood as a compatibility layer and must fit around established routes and local maps rather than relocating them. Worlds without generation metadata must also receive a deterministic persisted compatibility seed if the geography layer requires one; that seed must be derived from stable saved data, not display names or mutable content.

## Persistent route records

Persistent world connectivity is represented by first-class route records. `World.routes` is the sole saved authority for travel topology; each route has a stable string ID, two location endpoints stored in canonical numeric-ID order, and an explicit direction (`Bidirectional`, `AtoB`, or `BtoA`). Distinct route IDs may connect the same pair of locations. Route identity is independent of display names, local-map positions, and presentation order.

`Location.exits` is now a derived in-memory compatibility view only. It is rebuilt from route records after load and is omitted from new saves. Gameplay travel and known-route pathfinding consult route records directly. A physical `MapExit` carries the identity of the route it represents; entering a passage validates that exact route rather than merely checking that some connection exists between the same endpoints.

### Legacy save migration

Save format v5 migrates pre-route location adjacency deterministically. The migration normalizes each endpoint pair, derives a route ID as `route.compat.legacy.{lower_id}.{higher_id}`, and preserves whether the old adjacency was bidirectional or one-way. It does not allocate entity IDs or change `World.next_id`.

Existing local-map passages are assigned the first matching route ID in stable lexical order without changing their passage ID or grid position. When additional routes exist between the same locations, reconciliation appends distinct passages. Reconciliation never clears established exits or regenerates saved map geometry; repeated reconciliation is idempotent.

Route records establish identity and graph connectivity and now persist a deterministic terrain-aware path between their endpoint anchors. The bounded A* planner samples the shared macro lattice at 32-sample intervals, avoids ocean terrain, and treats elevation and mountain-ridge strength as soft costs. Each route stores its planning status, canonical endpoint-A-to-endpoint-B path, total cost, broad profile, and detected crossings. A confirmed river crossing is recorded only when a route segment intersects both banks of the deterministic local channel geometry; the record retains the river identity, relative width/flow, and Q16 bank-entry/exit positions. Inland-lake crossings retain the stable generated lake ID and, when the path enters and exits over dry land, exact Q16 shore-entry/shore-exit positions; the crossing coordinate is the rounded shore-pair midpoint. Narrower rivers require a ford and wider rivers require a bridge; lakes require a ferry; unresolved river estimates require survey; mountain-ridge crossings are identified as pass candidates. These remain route-level infrastructure requirements, not proof that the corresponding planned journey has a constructed crossing.

New geography-backed outdoor maps project river channels into local tiles and place at most one deterministic, walkable ford or bridge across each connected channel component when both banks are present. Narrow/medium channels use fords; wide/very-wide channels use bridges. This makes local river geography physically traversable without making ocean or lake water walkable, but these generic crossings are not yet tied to each route's exact persisted bank pair. Resolved inland-lake crossings with a stable lake ID and two verified shores expose a ferry landing at the route passage and a transient boarding/crossing/landing sequence. The current ferry scene uses colored placeholders rather than authored docks or full boat animation. Route-specific river crossing presentation, explicit infrastructure ownership/construction, and consistent enforcement of unsupported crossings remain follow-up work.

New local-map passages inherit the route profile and prefer the compass side indicated by the planned macro path. Existing passage positions, IDs, and saved map geometry are never relocated during reconciliation. The direct-corridor assessment remains available as a lightweight diagnostic, but route creation uses the overland A* planner, which blocks ocean terrain and treats elevation and mountain ridges as soft costs. Underground cave routes are handled separately through the generated cave network: deterministic surface entrances, reciprocal chamber passages, and adjacent-stratum shafts materialize or reuse persistent cave-chamber locations and bidirectional routes. Using a cave connection upgrades the same reachable interior map transition in place; older maps are augmented on entry without discarding established map state. Route discovery state, infrastructure ownership/construction, travel costs, and richer authored local-map connection-point placement remain future topology work. Frontier materialization is implemented as a deterministic, on-demand route-graph expansion and does not generate a terrain world-map chunk or populate a full settlement simulation.

## System boundaries

The responsibilities are deliberately separated:

- **World topology** owns region identity, location identity, and authoritative travel connections.
- **Local-map generation** owns map geometry and local points of interest for one location.
- **Travel** owns route discovery, transition rules, fast-travel eligibility, elapsed time, and travel-level events.
- **Content generation** populates regions and locations with inhabitants, resources, discoveries, and opportunities. It is designed separately.
- **World simulation** determines which supported entities and processes change over time; a large world does not imply that every entity runs a continuous simulation.
- **Persistence/inheritance** keeps established world changes and separates them from character-specific state.
- **Presentation/input** exposes Explore, physical exits, and fast travel without making Bevy presentation code authoritative for gameplay rules.

## Architectural constraints

- Extend the existing world/location graph and renderer-neutral spatial model rather than creating an independent parallel travel system.
- Preserve direct access to existing textual interactions at safe locations.
- Use explicit route/destination identities and validate consistency between world-graph connections and local-map exits.
- Keep local-map generation deterministic where practical and preserve maps after creation.
- Do not require detailed simulation of distant regions merely because they exist.
- Introduce the capability incrementally with focused tests and safe save migrations.
- The initial spatial-exploration milestone remains limited to its current acceptance criteria; this issue describes the direction of later development.

## Implementation sequence

1. **Topology specification and compatibility audit:** define region coordinates, stable generation keys, graph connections, and how existing worlds/saves are represented.
2. **Explore availability across location types:** make the optional action consistently available while preserving direct textual actions; define default local-map profiles for settlements, wilderness, ruins, and other locations.
3. **Map exits and graph-linked transitions:** let the player reach and use physical exits that transition to the correct connected location, with reciprocal connections and persistence.
4. **Frontier region expansion:** create a region on demand through a frontier route, connect it into the world, and verify repeat visits do not reroll it.
5. **Known-route fast travel:** retain convenient travel while enforcing time, discovery, encounter, and authored-route rules.
6. **Persistence/inheritance integration:** preserve geography, routes, and local discoveries across saves and subsequent characters.
7. **Incremental content and simulation integration:** add meaningful generated opportunities and route-level consequences through their own designs.

Each stage should be separately testable. The sequence does not require every stage to ship at once.

## Acceptance criteria for the eventual system

- Explore is available at every accessible world location, but existing textual/location-level actions do not require spatial movement.
- Entering Explore opens the correct local map; re-entry restores the established map and its persistent changes.
- Physical map exits correspond to valid graph connections and transition to the correct destination.
- Connected locations and intermediate route locations can be explored independently without a seamless global terrain map.
- A frontier route can create a new region on demand; revisiting that route leads to the same established destination.
- Generation of a region is repeatable from stable world/region inputs regardless of exploration order, while already-persisted changes take precedence over the generator.
- Fast travel uses known routes, advances world time appropriately, and respects active encounter and authored-route restrictions.
- Existing worlds, authored starting locations, routes, local maps, and save files remain valid after migration.
- The system works through the shared gameplay model and can be used by supported frontends without duplicating authoritative world/travel logic.

## Additional geography requirements recorded during issue review

The intended regional/world map is a geography-informed route graph, not a generic illustrated background. The world data must describe a coherent landscape first; map presentation is then derived from that authoritative data.

### World-generation requirements

- **One continuous macro-geography:** regions are named/managed areas within a shared world space, not independently generated islands or unrelated maps. Terrain and large-scale features must remain coherent across region boundaries.
- **Stable global geography:** use the world seed, stable global coordinates, and an explicit generation-ruleset version to derive terrain fields. Adjacent regions must sample compatible values and features at their shared boundaries, regardless of generation order.
- **Meaningful physical features:** represent enough macro-scale data—such as elevation, biome/climate, watersheds, rivers, coastlines, and major terrain barriers—to make placement and travel routes plausible. This does not require simulating every distant tile continuously.
- **Locations anchored to the landscape:** generated and authored locations have stable geographical placement/footprints and environmental requirements. Their local spatial maps must fit their surrounding macro-geography rather than being unrelated themed scenes.
- **Physically plausible connections:** the authoritative location graph remains the source of travel connectivity, but its edges must correspond to physically possible routes through the shared landscape (roads, trails, passes, bridges, fords, docks, cave passages, and so on). Local-map exits must agree with those connections.
- **Authored/procedural compatibility:** authored locations can specify placement constraints, terrain/biome requirements, footprints, and connection points. Procedural placement must respect them and join them into the same continuous landscape.
- **Deterministic expansion and persistence:** region generation is independent of exploration order. Existing coordinates, generated geography, authored anchors, local maps, and established routes remain stable when frontier regions are created or rules evolve.
- **Conservative migration:** old worlds without geography metadata retain their existing region/location identities and graph. Migration must assign a stable compatible representation without arbitrarily rearranging or invalidating established content.
- **Derived map presentation:** the eventual illustrated world/region map is generated from the actual terrain, location visuals, and route data. It must not invent a separate generic landscape or maintain its own competing geography.

Mountain ridge strength is exposed as a soft route-corridor resistance signal, not an absolute impassability flag. Route placement should prefer weaker corridors through a ridge belt where practical; these low-resistance crossings are candidates for mountain passes. The actual route type (trail, road, engineered pass, or another authored connection) determines what infrastructure or local-map passage is required. Water remains a separate crossing constraint: oceans, lakes, confirmed rivers, and unresolved river estimates must retain their distinct classifications. A route must not silently treat a water feature or an inconclusive river estimate as ordinary dry land. Underground cave geography is sampled independently on negative strata. Chamber nodes expose stable global identities and reciprocal passage links to neighboring chambers; a physical cave route may use only a passage represented by that connected network. Separate strata are not implicitly connected vertically: only explicit deterministic entrance/shaft links between adjacent layers are traversable; matching horizontal coordinates alone never imply a connection.

### Selected macro-geography coordinate contract

The initial macro-geography implementation uses a hierarchical signed coordinate model:

- Each region occupies one fixed **256 × 256 macro-sample chunk**.
- `RegionCoordinate` identifies a chunk in signed region coordinates. The origin is `(0, 0)`; negative coordinates support expansion in every direction. Its `stratum` is zero at the surface and negative for underground layers.
- `MacroCoordinate` identifies a sample in one shared global lattice and carries the same stratum index. Conversion uses Euclidean division/remainder so negative coordinates partition consistently; separate underground networks can occupy independently addressable strata while sharing horizontal coordinates.
- A region-local sample `(x, y)` maps to global sample `(region.x * 256 + x, region.y * 256 + y)`, with checked arithmetic and local indices in `0..256`.
- Coordinate-derived sample seeds depend only on world seed, geography ruleset version, and signed global coordinate including stratum. Generation order, runtime entity IDs, and display names are excluded.
- Geography ruleset version `1` identifies the original coordinate and elevation sampling contract; ruleset `2` adds deterministic mountain uplift while preserving the ruleset 1 elevation field for existing worlds pinned to it. Legacy saves default to ruleset `0` and unassigned region coordinates until the explicit compatibility embedding migration.

The coordinate foundation has since been extended with deterministic elevation, climate and biomes, shorelines, hierarchical watersheds, river-channel estimates, closed-basin lakes, surface-water connectivity, mountain ranges, underground cave generation, and geography-backed location anchors/footprints. Persistent cave-chamber routes and reachable local-map transitions are implemented separately from the overland planner. More detailed underground geology, route-specific physical infrastructure, and a geography-backed World Map presentation remain future work. Region coordinates are gameplay data, not World Map presentation positions. Macro samples are also distinct from local-map tiles; no fixed conversion between those resolutions is implied.

### Architectural boundary

World topology owns region/location identity and route edges. Macro-geography owns the continuous terrain model; geography-aware location placement and footprints build on that foundation. Local-map generation derives playable local environments from those anchors. Travel validates and uses the corresponding physical connections. Map rendering only visualizes these systems; visual layout coordinates are not authoritative world data.

The simple graph-style World Map being implemented separately as an interim feature must not introduce geographical coordinates or persist its temporary graph-layout positions. It should consume region/location identities and route edges so it can later be replaced by a geography-backed presentation without changing gameplay travel rules.
