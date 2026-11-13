# World Map

The World Map is the focused surface for understanding the current region, current location, previously visited places, and established travel connections. It is opened from the spatial pause menu. It provides fast travel between known locations without replacing ordinary physical exploration or creating a second travel ruleset.

## Interim implementation: topological route graph

The first version intentionally avoids representing geographical terrain. It visualizes the authoritative world-location graph using an automatically arranged route diagram.

- The current location is always visible and marked as the current position.
- Locations with established persistent local maps are shown as explored.
- Direct exits from the current or an explored location are shown as frontier locations; those locations are not marked explored until their own local map has been established.
- Connections are drawn only when the corresponding source location's authoritative `exits` list contains the destination. Reciprocal connections are identified separately from one-way connections.
- The map does not reveal unrelated unvisited locations or pretend that map nodes have geographical distance, direction, terrain, or scale.
- The current region and current location are shown explicitly above the graph. Region names are also attached to their location nodes.
- Locations with established local maps are previously visited places. These are the only destinations eligible for fast travel.
- The map marks a visited destination as fast-travelable only when a shortest path exists from the current location using reciprocal world-graph connections whose intermediate locations have also been visited.
- Fast travel executes the normal travel action for every route hop. Each hop advances the world clock and turn, records history, synchronizes quests, and runs arrival events.
- A death, active threat, or consequential arrival event interrupts a multi-hop journey at that stop. The player resumes ordinary spatial play there; fast travel never skips over an encounter or event.
- Visible but unvisited frontier locations cannot be selected for fast travel. They must be reached through ordinary movement and physical passages in spatial gameplay.
- Choosing a fast-travelable map node is the fast-travel action; there is no separate destination-only Travel screen.
- Physical passage use remains the normal travel path inside spatial gameplay. It continues to validate the immediate reciprocal connection and enters the destination's local map without invoking fast travel.

Visibility and visit eligibility primarily use `World.local_maps`: establishing a persistent local map records a durable visit. For saves created before that persistence existed, the map also recognizes retained travel-history entries written by the active character or a recorded predecessor in the same inheritance line. This compatibility fallback can only recover visits still present in the bounded history; once an old travel entry has been trimmed, it cannot reconstruct that visit. New visits continue to be recorded through persistent local maps. This is an interim discovery rule, not the final definition of cartography or fog of war.

## Presentation-only layout contract

The graph layout is computed when the map is rendered. A breadth-first traversal from the current location places nodes in columns by graph distance; nodes within a column are ordered deterministically by region name and stable location ID. The resulting pixel positions exist only in the presentation layer.

The world model must not receive temporary x/y coordinates, route bends, visual grouping offsets, map zoom, or other renderer-specific layout state. The renderer consumes stable location identities, region identities/names, exploration flags, fast-travel eligibility, and route endpoints. It must never make a displayed line authoritative for travel.

This boundary allows the interim graph layout to be discarded or replaced without migrating saves or changing the authoritative travel model.

The graph is presented inside a clipped two-dimensional viewport. Its pan and zoom are transient presentation state held by the Bevy viewport controller; they are not written to campaign data or saves. Map destination buttons are canvas-positioned controls, not members of the shared vertically stacked menu-action group; the group organizer must leave absolute-positioned controls under their owning canvas. Previously visited fast-travel destinations use the map's warm accent treatment at rest and retain hover, pressed, and selected feedback. Opening the map starts at the base zoom centered on the current location. **Fit** frames the complete currently displayed graph, and **Reset** restores the opening view. Android supports one-finger drag-to-pan and two-finger pinch-to-zoom. Desktop supports pointer drag-to-pan and mouse-wheel zoom centered on the cursor. Destination nodes use release-based manual activation: a press must begin on a destination and finish as a tap over that same node. Dragging or pinching cancels destination activation, even when the gesture ends over the original node. The toolbar and surrounding map information stay outside the transformed canvas so they remain at normal UI scale. When the viewport changes size, pan bounds and the reset position are recalculated without changing graph layout or world data.

## Intended final direction

The final map should be a continuous illustrated landscape derived from coherent world geography, with the actual location network and routes represented on top of it. Geography must be generated and persisted as world data rather than invented by the map renderer.

The future system must support:

- One shared macro-geography across region boundaries, with coherent terrain and large-scale features rather than separately generated region backdrops.
- Stable geography from world seed, global coordinates, and a versioned generation ruleset, independent of exploration order.
- Location placement and footprints that respect terrain, climate/biome, elevation, and authored environmental constraints.
- Routes that correspond to physically plausible connections through the landscape and agree with local-map exits.
- Authored and procedurally generated locations participating in the same geography.
- Deterministic frontier expansion, persistent established geography/routes, and conservative migration for older worlds that have no geographical metadata.
- A map renderer that derives its illustration from actual geography, locations, and authoritative connections.

These requirements are recorded as future work in [issue #365](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/365). The interim map is not a substitute for that work.

## Replacement strategy

Keep the current division between:

1. **World data:** stable region/location identities, local-map persistence, and authoritative route edges.
2. **World-map view:** a renderer-neutral summary of visible locations, their region context, exploration state, and route endpoints.
3. **Presentation:** temporary graph layout, line routing, visual states, and controls.
4. **Travel:** ordinary movement and physical-passage transitions remain authoritative in spatial gameplay; fast travel finds a path through previously visited locations and invokes the normal travel action for each hop.

When macro-geography arrives, add geography to the world-generation/topology layer and derive a geography-aware map presentation from it. The current map screen entry point, stable route/location identities, fast-travel eligibility, and travel action should remain reusable. Remove temporary graph layout only when the geography-backed representation can supply a coherent map; do not migrate interim pixel positions into world saves.
