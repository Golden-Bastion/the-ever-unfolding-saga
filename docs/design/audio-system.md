# Game Audio and Music System

> Design record migrated from [GitHub issue #230](https://github.com/Golden-Bastion/the-ever-unfolding-saga/issues/230). This document preserves the settled direction, constraints, and intentionally open design choices. Implementation status and sequencing belong in the root [roadmap](../../ROADMAP.md); this document is not a progress log.

Plan and eventually implement a proper audio/music system for The Ever-Unfolding Saga.

This is a future milestone, not part of the current Android/UI work. The architecture should be considered before the eventual Bevy presentation grows too far, but the actual soundtrack can be added later when gameplay states and atmosphere are more stable.

## Scope
- Introduce an audio/music manager or equivalent abstraction rather than having individual screens manipulate audio files directly.
- Represent music through game states/events, e.g. `Exploration`, `Danger`, `Combat`, `Victory`, `Defeat`, `Title`, `Town`, `Meditation`, and major encounters.
- Support seamless looping where appropriate.
- Support fades/crossfades and clean transitions between music states.
- Add separate configurable music and sound-effects volume controls.
- Keep presentation/audio concerns separated from core game logic so the system remains usable through the eventual GUI migration.
- Start with a small soundtrack rather than trying to cover every possible situation.

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
