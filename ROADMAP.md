# The Ever-Unfolding Saga

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md). The intended product direction is recorded in [`docs/design-principles.md`](docs/design-principles.md); this roadmap distinguishes those long-term goals from what is currently implemented and validated.

## Current state

The v0.52.0–v0.52.6 storage, mod-management, and character-focused interface work is complete and recorded in the roadmap history. Spatial exploration and interaction are now the active development line. PR #404 implements persistent spatial actors, contextual action/target resolution, and the modular portrait catalogue. Its implementation commit passed automated checks, but desktop and Android landscape playtesting remains outstanding. The player-anchored radial interaction wheel now adapts to landscape viewport dimensions, adds a second ring for crowded target sets, and animates selected-entry feedback. Shared scene transitions and the dedicated close-up dialogue presentation are also implemented under #405. Dialogue composes the existing deterministic player/NPC portrait layers through mod-aware asset loading; manual desktop and Android landscape playtesting remains outstanding.

## Active milestone

**v0.52.x — Spatial exploration and interaction** (prototype #353; actor and interaction foundation #404; presentation follow-up #405; world-topology design #365)

The renderer-neutral local-map model, deterministic generation, Bevy exploration view, continuous movement, blocked-terrain collision, camera tracking, depth ordering, and world-persistent map state are implemented. The prototype includes desktop keyboard movement, an Android virtual joystick, first-pass pixel-art terrain and character assets, basic prop collision and occlusion, and the authored Old Shrine chamber layout (#381). The exact active exploration position and session remain character-specific, while established local maps and their lasting changes persist with the world.

PR #404 extends Explore to safe and dangerous locations and removes detached Talk and Search Remains dashboard actions. NPCs and searchable remains receive deterministic placements on reachable map cells and persist with their local maps; older saves receive safe defaults until a map is first entered. Nearby NPCs, remains, points of interest, and passages resolve into stable action/target pairs through renderer-neutral discovery in `interaction_actions.rs`; the Bevy adapter consumes those results rather than owning proximity, line-of-sight, actor-state, and inventory eligibility rules. The player-anchored radial action wheel presents every eligible pair, adapts to available landscape space, uses a second ring when nearby targets crowd the first, and gives selection changes a brief eased pulse. Directional keyboard selection and Android touch activation use the visible entry positions, and selected identities are revalidated before dispatching the existing Talk, Search Remains, Examine, or Enter behavior. New local maps can also receive multiple deterministic, campaign-authored points of interest; they remain hidden until their map tiles are explored, then appear as world markers and eligible Examine actions. Their reveal and examination state persists with the local map, and mods can extend or override definitions by stable ID. NPC dialogue now transitions into a dedicated close-up scene showing the player and NPC together; both portraits are composited from the deterministic, weighted, mod-extensible catalogue of transparent body, face, hair, headgear, and accessory layers. The shared transition primitive animates scene entry and exit while the spatial session remains mounted, and the mod-aware loader supplies the selected portrait PNGs with a text fallback when no NPC layer can be loaded.

Manual desktop and Android landscape playtesting remains required before #404 is complete. The #405 interaction presentation now includes the animated radial wheel with responsive crowded-target layout and eased selection feedback, reusable scene-root transitions, and the close-up two-character dialogue scene with composited portrait layers. Manual desktop and Android landscape playtesting remains required before the interaction milestone is considered fully validated. The first authored spatial-discovery slice now covers Old Shrine with the Black Altar and Fresh Soot points of interest. Physical passages now invoke the existing graph-travel rules and transition directly into connected destination maps without leaving spatial mode. The current map and character session are persisted before travel; destination maps are restored from world state or generated lazily on first arrival. Frontier-region creation, known-route fast travel, expanded room topology, additional authored layouts, and room-aware camera transitions remain future work under #365. Richer multi-step clue chains, evidence collection, investigation outcomes, and dynamic encounter integration also remain future work.

Keep the existing world model, quests, time, narrative, and location-level actions authoritative. Spatial exploration supplements rather than replaces them. Real-time spatial combat and audio remain outside this milestone.

Related design issues: #228, #229, and #252.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

The product direction is a character-focused, graphics-first spatial RPG backed by a persistent simulation. Spatial exploration is being delivered incrementally; real-time spatial action combat and audio integration remain future work, not current milestone claims. Keep world systems character-centred rather than turning them into remote grand-strategy management. Desktop and mobile support, hybrid input, scalable visual/simulation quality, offline single-player, player-controlled updates, and robust save/mod compatibility remain guiding constraints. See the [design principles](docs/design-principles.md).

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.

## Release distribution

GitHub Release publishing can optionally mirror all five desktop builds and three Android APKs to itch.io through Butler. The Android release workflow waits for the complete set of release assets before uploading them to their corresponding platform/architecture channels. Enable the mirror by configuring the repository Actions variable `ITCHIO_TARGET` and secret `BUTLER_API_KEY`; without both values, GitHub Release publishing continues without the itch.io upload.
