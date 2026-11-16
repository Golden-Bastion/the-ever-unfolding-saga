# Content System

The content system moves campaign definitions out of gameplay code and loads them as validated runtime data.

## Base content

The base campaign is stored in `data/base_content.json`. Stable content IDs identify locations, factions, NPCs, quests, encounters, atmosphere, and events. World creation seeds game state from the loaded definitions instead of relying on large hardcoded bootstrap tables.

## Modules

The content implementation was split into three responsibilities:

- `content/definitions.rs` contains data structures and validation rules.
- `content/loader.rs` handles base/mod discovery, loading, merging, event filtering, path discovery, and loader tests.
- `content/seeding.rs` translates validated content into the initial world state.

`content.rs` remains a compatibility-facing facade for the existing loading entry point.

## Mods

Mods are discovered under `data/mods` and loaded after the base content. Content is merged using stable IDs and location keys, allowing mods to replace or extend compatible definitions without duplicating the whole campaign.

Broken mod files produce load warnings instead of crashing the game. Mod validation is performed alongside the base validation rules and can reference compatible base content where the schema allows it.

## Validation

Content loading validates duplicate IDs, broken references, event definitions, faction references, reputation ranges, and other cross-content dependencies. Invalid definitions are rejected while valid content continues loading when possible.

## Authored local-map layouts

A location may optionally declare a `spatial_layout` identifier. The current built-in identifiers are `shrine_chamber` (Old Shrine), `ashen_gate_causeway` (Ashen Gate), and `hollow_market_square` (Hollow Market). These select deterministic renderer-neutral terrain, props, room membership, and door topology for newly generated local maps. The gate and market templates require at least a 24×16 map. Unknown identifiers and layouts that do not fit the configured dimensions fall back to procedural generation. Authored layout changes do not rewrite established maps saved by a player.

## Authored spatial discoveries

A location may define an optional `spatial_discoveries` array. Each entry has a stable `id`, a player-facing `title`, a `description`, and an optional `requires_evidence` array containing stable IDs of other spatial discoveries. These definitions become points of interest when that location's local map is first generated. Placement is deterministic for the established map seed and avoids blocked or unreachable cells, exits, and actor placements. All listed prerequisites must be known by the current character before the Examine action becomes available. An empty list keeps the older generic fallback for locations that have not received authored discoveries yet.

Evidence IDs are global across campaign locations. Content validation rejects empty, repeated, self-referential, unknown, or cyclic prerequisites. Mods may extend a location's discoveries or override an entry by stable ID; a matching ID replaces its title, description, and prerequisite list. New discovery definitions do not rewrite an already-persisted local map: an established map remains authoritative for its generated placements and evidence requirements.

The local map stores the physical `discovered` and `interacted` flags. Character understanding is separate: each character stores the evidence IDs they personally learned in `known_evidence_ids`. Examining a clue records its ID for that character. A later character may examine a site that was already interacted with in a previous life, but does not inherit the previous character's deductions. Older saves default missing prerequisite and character-knowledge fields to empty collections.

## Evidence-linked quest objectives

A quest template may define an optional `required_evidence_ids` array. Each ID must reference an authored spatial discovery and must be unique within that quest. Campaign seeding adds a `DiscoverEvidence` objective for each required ID, using the discovery title as its player-facing label. These objectives are appended to the quest's existing visit, item, and encounter objectives; they do not replace them. Progress follows the current character's known evidence, not the shared map's physical interaction flags.

## Optional assets

Content may reference optional ASCII portraits, location scene art, and item illustrations. Missing optional visual assets must not crash the game or make content inaccessible. Use appropriate fallbacks while preserving the intended graphics-first presentation for spatial gameplay; text-only fallback is a resilience measure, not the product’s overall identity.

## Compatibility and runtime use

Campaign content is runtime data rather than save data. Saves keep persistent world state and rehydrate the current campaign definitions when loaded. The runtime caches loaded campaign content so repeated gameplay actions do not reload files unnecessarily.

## Design direction

Keep campaign data declarative and stable-ID based. Put reusable content in data files and keep game code responsible for generic behavior rather than embedding campaign-specific definitions in control flow.


## Modular character illustrations

Campaign content may define reusable `character_parts`. The bundled campaign includes a 15-part starter catalogue spanning body, face, hair, headgear, and accessory slots. Every layer is a transparent 64×96 PNG aligned to a shared portrait canvas. Each part has a stable ID, slot, mod-aware PNG asset reference, selection weight, draw layer, and optional required/excluded appearance tags. NPC definitions may specify optional `appearance_tags`; the player and NPC use the same renderer-neutral weighted generator.

The generator selects zero or one compatible part per slot from a deterministic seed derived from the world generation seed and the actor's persistent entity ID. Headgear and accessory slots are optional and receive a weighted "none" outcome, so NPCs may naturally appear without either layer. The same actor therefore retains the same composition across redraws and save/load without storing Bevy handles or selected asset paths in saves. Player creation can explicitly choose none for either optional slot; that choice is stored as a stable slot-specific marker, while older saves without the marker continue to use the first available part. Mods may add parts or replace a definition by stable ID; assets are resolved through the normal mod-aware loader. The dialogue scene composites available transparent PNG layers in ascending draw order. Missing assets or empty slots are omitted; the legacy ASCII portrait is retained as a fallback when no usable NPC layer can be loaded.

The content and generator layers do not load images or depend on Bevy. A frontend composes the returned assets in their declared draw order.
