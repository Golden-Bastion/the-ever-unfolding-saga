# World Map

The World Map is the focused surface for understanding the current region, current location, previously visited places, and established travel connections. It is opened from the spatial pause menu. It provides fast travel between known locations without replacing ordinary physical exploration or creating a second travel ruleset.

## Interim implementation: topological route graph

The first version intentionally avoids representing geographical terrain. It visualizes the authoritative world-location graph using an automatically arranged route diagram.

- The current location is always visible and marked as the current position.
- Locations with established persistent local maps are shown as explored.
- A route and its destination appear only after the corresponding local passage tile has been physically uncovered. Merely visiting a location does not reveal every exit or the locations beyond them.
- Connections are derived from authoritative `World.routes` records and filtered through persistent route-discovery state. Each route has a stable identity, two canonical location endpoints, and explicit bidirectional or one-way direction; separate routes may connect the same pair of locations. `Location.exits` is only a transient adjacency projection and is not saved.
- The map does not reveal unrelated unvisited locations or pretend that map nodes have geographical distance, direction, terrain, or scale.
- The current region and current location are shown explicitly above the graph. Region names are also attached to their location nodes.
- Locations with established local maps are previously visited places. These are the only destinations eligible for fast travel.
- The map marks a visited destination as fast-travelable only when a shortest directed path exists from the current location using physically discovered routes that permit travel in that direction and whose intermediate locations have also been visited.
- Fast travel executes the normal travel action for every route hop. Each hop advances the world clock and turn, records history, synchronizes quests, and runs arrival events.
- A death, active threat, or consequential arrival event interrupts a multi-hop journey at that stop. The player resumes ordinary spatial play there; fast travel never skips over an encounter or event.
- Visible but unvisited frontier locations cannot be selected for fast travel. They must be reached through ordinary movement and physical passages in spatial gameplay.
- Choosing a fast-travelable map node is the fast-travel action; there is no separate destination-only Travel screen.
- Physical passage use remains the normal travel path inside spatial gameplay. It validates the passage's exact route ID and direction before entering the destination's local map; it does not invoke fast travel.

## Tracked quest destination marker

Quest tracking is explicitly selected from Character Hub → Quests. A character can track one offered, incomplete quest at a time. The quest title, description, objective list, and progress remain in the Character Hub; the spatial exploration HUD does not display a persistent quest panel.

The map resolves the first incomplete objective for the tracked quest and marks its destination with a small brass indicator on the corresponding location node:

- `VisitLocation` resolves by the objective's exact location name, with a numeric location-ID fallback for legacy objectives.
- `DiscoverEvidence` resolves the stable evidence ID through authored spatial discoveries, then finds the world location containing that discovery.
- `AcquireItem` and `DefeatEnemy` use the quest's explicitly assigned target location.

The marker advances when the current objective becomes complete. If the objective has no incomplete step or its destination cannot be resolved, no marker is shown. A known tracked destination may be added as a visible map node even beyond the ordinary exploration frontier, but tracking does not mark it explored, invent route connections, or make it eligible for fast travel. Existing route records and visited-route rules remain authoritative.

The current region can also show a broad terrain survey derived from a bounded sample of its deterministic macro-geography. It reports only broad biome/landscape categories (for example forest, desert, highlands, or mountain ridges); it does not reveal exact local terrain, landmarks, or route passages. This regional knowledge is world-persistent and survives character inheritance.

Visibility and visit eligibility primarily use `World.local_maps`: establishing a persistent local map records a durable visit. For saves created before that persistence existed, the map also recognizes retained travel-history entries written by the active character or a recorded predecessor in the same inheritance line. This compatibility fallback can only recover visits still present in the bounded history; once an old travel entry has been trimmed, it cannot reconstruct that visit. New visits continue to be recorded through persistent local maps. For new games, local map exploration reveals route knowledge only when the exit tile enters the existing revealed-tile area. The route is then retained in `World.discovered_route_ids`; fast travel uses only those known routes. A one-time migration preserves the previous frontier visibility and fast-travel behavior for existing saves.

## Presentation-only layout contract

The graph layout is computed when the map is rendered. A breadth-first traversal from the current location places nodes in columns by graph distance; nodes within a column are ordered deterministically by region name and stable location ID. The resulting pixel positions exist only in the presentation layer.

The world model must not receive temporary x/y coordinates, route bends, visual grouping offsets, map zoom, or other renderer-specific layout state. The renderer consumes stable location identities, region identities/names, exploration flags, fast-travel eligibility, and route endpoints. It must never make a displayed line authoritative for travel.

This boundary allows the interim graph layout to be discarded or replaced without migrating saves or changing the authoritative travel model.

The graph is presented inside a clipped two-dimensional viewport. Its pan and zoom are transient presentation state held by the Bevy viewport controller; they are not written to campaign data or saves. Map destination buttons are canvas-positioned controls, not members of the shared vertically stacked menu-action group; the group organizer must leave absolute-positioned controls under their owning canvas. Previously visited fast-travel destinations use the map's warm accent treatment at rest and retain hover, pressed, and selected feedback. Opening the map starts at the base zoom centered on the current location. **Fit** frames the complete currently displayed graph, and **Reset** restores the opening view. Android supports one-finger drag-to-pan and two-finger pinch-to-zoom. Desktop supports pointer drag-to-pan and mouse-wheel zoom centered on the cursor. Destination nodes use release-based manual activation: a press must begin on a destination and finish as a tap over that same node. Dragging or pinching cancels destination activation, even when the gesture ends over the original node. The toolbar and surrounding map information stay outside the transformed canvas so they remain at normal UI scale. When the viewport changes size, pan bounds and the reset position are recalculated without changing graph layout or world data.

## Final geography boundary

The current screen remains an interim topological route graph with a broad regional terrain summary. The persistent macro-geography foundation now drives terrain projection for new outdoor local maps and terrain-aware route planning, but a geography-backed world/region map is still future work: it needs an explicit map-scale projection and cartographic presentation derived from coherent geography, location placement, and route data. It is not a visual reskin of the current graph. Its requirements and boundaries are specified in [World Topology, Local Exploration, and Hybrid Travel](../design/world-topology-and-travel.md), and the root [roadmap](../../ROADMAP.md) sequences the remaining work.
## Replacement strategy

Keep the current division between:

1. **World data:** stable region/location identities, local-map persistence, and authoritative route records with stable IDs and explicit direction.
2. **World-map view:** a renderer-neutral summary of visible locations, their region context, exploration state, and route endpoints.
3. **Presentation:** temporary graph layout, line routing, visual states, and controls.
4. **Travel:** ordinary movement and physical-passage transitions remain authoritative in spatial gameplay; fast travel finds a path through previously visited locations and invokes the normal travel action for each hop.

As the macro-geography foundation matures, derive a geography-aware world/region map presentation from its terrain, location placement, and route data. The current map screen entry point, stable route/location identities, fast-travel eligibility, and travel action should remain reusable. Replace the temporary graph layout only when the geography-backed representation can supply a coherent map; do not migrate interim pixel positions into world saves.


An empty fast-travel destination list does not add a separate explanatory notice to the map. The graph and its Back action remain visible, avoiding a redundant empty-state message while preserving navigation.
