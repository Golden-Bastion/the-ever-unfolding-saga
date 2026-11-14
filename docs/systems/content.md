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

## Authored spatial discoveries

A location may define an optional `spatial_discoveries` array. Each entry has a stable `id`, a player-facing `title`, and a `description`. These definitions are converted into points of interest when that location's local map is first generated. Placement is deterministic for the established map seed and avoids blocked or unreachable cells, exits, and actor placements. An empty list keeps the older generic fallback for locations that have not received authored discoveries yet.

Mods may extend a location's discoveries or override an entry by stable ID. Entries from earlier content remain when a mod supplies additional IDs; a matching ID replaces its title and description. IDs must be unique, and the required text fields must not be blank. New discovery definitions do not rewrite an already-persisted local map: an established map remains authoritative for its generated placements and discovery state.

Points of interest are hidden until the player explores the tiles around them. Revealed discoveries become visible world markers and contextual Examine actions. The `discovered` and `interacted` flags are saved with the local map, not regenerated from campaign content after each load.

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
