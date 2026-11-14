# Pixel-Art Visual Direction and Asset Migration

> Design record migrated from [GitHub issue #229](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/229). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

Long-term art-direction foundation. This should not become a single bulk art-replacement task. Product-level direction follows the canonical [The Ever-Unfolding Saga design principles](../design-principles.md), which supersede conflicting older text-first wording in this issue. This is design intent, not a claim that all visuals or spatial systems are already implemented.

## Vision

Develop a coherent authored pixel-art language for a character-focused, graphics-first spatial RPG. Current ASCII artwork and text presentation may remain in existing screens and as fallbacks, but they do not define the product's overall identity. Spatial gameplay should communicate movement, exploration, atmosphere, environmental interaction, and future action combat primarily through visuals and spatial presentation.

Pixel art may be created collaboratively by the project owner and/or generated with assistance during development. Establish a coherent visual language before producing large quantities of final assets. Keep delivery incremental so useful improvements do not wait for a complete art set.

## Direction

Pixel art should eventually cover relevant categories such as:

- Characters and NPC portraits/sprites.
- Location and environmental scenes.
- Items, trophies, and other important objects.
- Tiles/environmental elements for the future tile/grid exploration maps.
- Enemies and combat entities.
- UI accents or other contextual visual pieces where they improve readability.

Dialogue, descriptions, records, history, and contextual text remain important. Pixel art and spatial presentation should be primary during spatial gameplay, communicating atmosphere, exploration, combat, and context rather than serving as optional decoration around an otherwise text-first experience.

## Migration approach

- Keep visual assets optional during the transition where practical.
- Replace ASCII pieces incrementally rather than requiring a complete art set before visual improvements can ship.
- Preserve text-only fallbacks where useful, especially for content without a finished asset.
- Keep asset references data-driven so content can select appropriate artwork without coupling gameplay rules to a specific renderer representation.

## Art-direction work required

Before a large production pass, define the target pixel-art style, palette approach, native pixel resolution, tile dimensions, character proportions, perspective/camera assumptions, sprite readability, animation expectations, environment layering, UI integration, asset naming/organization, and a practical workflow for creating/iterating assets.

The visual direction should account for desktop and Android landscape presentation and remain feasible for a small project with incremental asset creation.

## Relationship to future world exploration

The art direction should be designed with the future spatial exploration system in mind. The same visual language should be capable of representing authored tile-based environments, moving entities, enemies, investigation targets, and combat encounters without forcing a separate art style later.

## Non-goals for this issue

- No requirement to replace every current ASCII asset immediately.
- No implementation of the future open-world system.
- No fixed art toolchain requirement yet.
- No requirement that every screen become fully graphical; text-based dialogue, records, history, menus, and contextual interactions remain appropriate where they serve the experience.
- Intended platform scope is desktop and Android. iOS is not a project target.
