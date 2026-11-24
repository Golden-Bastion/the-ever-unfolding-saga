# Game Audio and Music System

> Design record migrated from [GitHub issue #230](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/230). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

Define and implement a proper audio/music system for The Ever-Unfolding Saga.

The initial audio-system implementation is part of the current v0.53.x development line; see the root roadmap for its remaining scope and sequencing. This document defines the intended boundaries and acceptance criteria. Keep the first implementation small and state/event-driven; soundtrack breadth can grow as gameplay states and atmosphere become more stable.

## Scope
- Introduce an audio/music manager or equivalent abstraction rather than having individual screens manipulate audio files directly.
- Represent music through game states/events, e.g. `Exploration`, `Danger`, `Combat`, `Victory`, `Defeat`, `Title`, `Town`, `Meditation`, and major encounters.
- Support seamless looping where appropriate.
- Support fades/crossfades and clean transitions between music states.
- Add separate configurable music and sound-effects volume controls.
- Keep presentation/audio concerns separated from core game logic so the system remains usable through the eventual GUI migration.
- Start with a small soundtrack rather than trying to cover every possible situation.

## Implementation boundary

The initial implementation uses [Verdandi](https://github.com/Golden-Bastion/verdandi) as the procedural music engine. Verdandi owns seeded composition, musical presets, intensity/layer control, looping playback, preset crossfades, and built-in musical stingers. TEUS owns state resolution and maps gameplay contexts to Verdandi controls.

General one-shot sound effects and environmental ambience remain in Bevy's native audio layer. Verdandi is not treated as a general-purpose game audio manager, and it does not own TEUS asset paths, cue IDs, or licensing records. The host-side audio manager routes those cues and applies the independent SFX gain.

Current integration and known gaps are documented in [the audio system reference](../systems/audio.md). Verdandi follow-ups are tracked in [issue #19](https://github.com/Golden-Bastion/verdandi/issues/19) (transport/gain/fades), [issue #20](https://github.com/Golden-Bastion/verdandi/issues/20) (non-musical cue scope), and [issue #21](https://github.com/Golden-Bastion/verdandi/issues/21) (defeat and extensible stingers).

## Initial soundtrack candidates
- Main/title theme
- Exploration/world theme
- Town/settlement theme
- Combat theme
- Danger/tense exploration theme
- Major encounter/boss theme
- Meditation/rest theme
- Victory and defeat/event stingers
- Ambient effects as appropriate (rain, wind, fire, caves, etc.)

## Music sourcing constraint
Potential sources to evaluate when this milestone begins:
- Arcade Composer — currently the most promising dedicated game-music option found during planning.
- OpenGameArt and other genuinely free/openly licensed game-audio sources — useful, but will require searching for tracks that fit in.
- Other free tools/sources discovered later.

## Licensing/documentation
For every third-party or generated track used in the game, record its source and applicable license/usage terms in project documentation so the soundtrack has a traceable paper trail.

## Acceptance criteria
- The game can change music based on gameplay state/event without presentation-specific code handling audio files directly.
- Music can loop and transition cleanly.
- Music and SFX volumes are independently configurable.
- At least the initial core soundtrack states are wired.
- All shipped audio has documented provenance and a license/usage basis compatible with the game's distribution.
