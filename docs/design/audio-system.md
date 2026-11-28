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

Verdandi is the single audio engine for TEUS. It owns seeded procedural composition, musical layers and stingers, procedural sound generation, PCM playback, mixing, looping, fades, and independent music/effects/ambience buses. Its Bevy 0.20 plugin supplies the application's audio output and exposes one unified controller.

TEUS owns authoritative state resolution and maps gameplay contexts and semantic gameplay events to Verdandi controls and generators. It does not maintain a parallel Bevy audio player, cue queue, or mixer. Host-owned recordings, if introduced later, must be decoded to PCM off the audio callback and supplied as Verdandi clips.

Current integration and remaining event wiring are documented in [the audio system reference](../systems/audio.md). Verdandi issues [#19](https://github.com/Golden-Bastion/verdandi/issues/19) (transport and gain controls) and [#20](https://github.com/Golden-Bastion/verdandi/issues/20) (non-musical playback) are complete. [Issue #21](https://github.com/Golden-Bastion/verdandi/issues/21) remains open for defeat/retreat and extensible event stingers. TEUS still needs to connect available generators to concrete gameplay events and review their output in-game.

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
