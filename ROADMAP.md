# The Ever-Unfolding Saga

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md). The intended product direction is recorded in [`docs/design-principles.md`](docs/design-principles.md); this roadmap distinguishes those long-term goals from what is currently implemented and validated.

## Current state

The v0.52.0–v0.52.6 storage, mod-management, and character-focused interface work is complete and recorded in the roadmap history. Spatial exploration and interaction are now the active development line. PR #404 implements persistent spatial actors, contextual action/target resolution, and the modular portrait catalogue. Its implementation commit passed automated checks, but desktop and Android landscape playtesting remains outstanding. The player-anchored radial interaction wheel, shared animated scene transitions, and dedicated close-up dialogue presentation are implemented under #405. Dialogue composes the existing deterministic player/NPC portrait layers through mod-aware asset loading; manual desktop and Android landscape playtesting remains outstanding.

## Active milestone

**v0.52.x — Spatial exploration and interaction** (prototype #353; actor and interaction foundation #404; presentation follow-up #405; world-topology design #365)

The renderer-neutral local-map model, deterministic generation, Bevy exploration view, continuous movement, blocked-terrain collision, camera tracking, depth ordering, and world-persistent map state are implemented. The prototype includes desktop keyboard movement, an Android virtual joystick, first-pass pixel-art terrain and character assets, basic prop collision and occlusion, and the authored Old Shrine chamber layout (#381). The exact active exploration position and session remain character-specific, while established local maps and their lasting changes persist with the world.

PR #404 extends Explore to safe and dangerous locations and removes detached Talk and Search Remains dashboard actions. NPCs and searchable remains receive deterministic placements on reachable map cells and persist with their local maps; older saves receive safe defaults until a map is first entered. Nearby NPCs, remains, points of interest, and passages resolve into stable action/target pairs. The player-anchored radial action wheel presents every eligible pair, with directional keyboard selection and Android touch activation, and revalidates the selected identity before dispatching the existing Talk, Search Remains, Examine, or Enter behavior. NPC dialogue now transitions into a dedicated close-up scene showing the player and NPC together; both portraits are composited from the deterministic, weighted, mod-extensible catalogue of transparent body, face, hair, headgear, and accessory layers. The shared transition primitive animates scene entry and exit while the spatial session remains mounted, and the mod-aware loader supplies the selected portrait PNGs with a text fallback when no NPC layer can be loaded.

Manual desktop and Android landscape playtesting remains required before #404 is complete. The #405 interaction presentation now includes the animated radial wheel, reusable scene-root transitions, and the close-up two-character dialogue scene with composited portrait layers. Manual desktop and Android landscape playtesting remains required before the interaction milestone is considered fully validated. Physical passage-to-passage travel, expanded room topology, additional authored layouts, and room-aware camera transitions remain future work under #365 and the spatial design follow-ups.

Keep the existing world model, quests, time, narrative, and location-level actions authoritative. Spatial exploration supplements rather than replaces them. Real-time spatial combat and audio remain outside this milestone.

Related design issues: #228, #229, and #252.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

The product direction is a character-focused, graphics-first spatial RPG backed by a persistent simulation. Spatial exploration is being delivered incrementally; real-time spatial action combat and audio integration remain future work, not current milestone claims. Keep world systems character-centred rather than turning them into remote grand-strategy management. Desktop and mobile support, hybrid input, scalable visual/simulation quality, offline single-player, player-controlled updates, and robust save/mod compatibility remain guiding constraints. See the [design principles](docs/design-principles.md).

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.

## Release distribution

GitHub Release publishing can optionally mirror all five desktop builds and three Android APKs to itch.io through Butler. The Android release workflow waits for the complete set of release assets before uploading them to their corresponding platform/architecture channels. Enable the mirror by configuring the repository Actions variable `ITCHIO_TARGET` and secret `BUTLER_API_KEY`; without both values, GitHub Release publishing continues without the itch.io upload.
