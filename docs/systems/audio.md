# Audio System

## Responsibility

The audio manager is a Bevy presentation adapter. It consumes authoritative lifecycle, combat, threat, and location state, then selects audio presentation. It does not own gameplay rules, alter simulation state, or require gameplay systems to load or manipulate audio files.

The boundary is split deliberately:

- **Verdandi** owns procedural background music: seeded composition, musical presets, arrangement intensity, looping playback, and musical event stingers.
- **Bevy audio** owns host-supplied one-shot effects and looping environmental ambience. TEUS retains ownership of cue IDs, asset paths, and when cues are requested.
- **AudioSettings** owns the player's persistent music and sound-effects volume preferences.

The integrated Verdandi dependency is version 0.6.0, pinned to commit 182bca1, and licensed under Apache-2.0. This avoids building a second music engine in TEUS or stretching Verdandi into a game-specific sound catalogue.

## Music context resolution

The Bevy audio manager resolves the current context from existing authoritative state:

| Context | Resolution | Verdandi arrangement |
|---|---|---|
| Title | No active campaign session or lifecycle outside active play | Ethereal, low intensity |
| Exploration | Active campaign in an ordinary location | Ethereal, moderate intensity |
| Settlement | Active campaign in a location using the authored hollow_market_square layout (or an explicitly supported settlement layout ID) | Contemplative |
| Danger | Active world threat or a location marked dangerous | Warbound, fast pace |
| Combat | Combat presentation is active, including its result screen | Orchestral, fast pace |

The campaign seed is taken from world-generation metadata, then the geography seed, then the saved game RNG state. The title context uses a stable fallback seed. Context changes request a Verdandi preset transition; the current preset API crossfades over two seconds. Context-specific intensity is set independently from the preset.

Combat start plays Verdandi's CombatStart stinger. A successful combat result plays Victory. Defeat and retreat currently have no matching Verdandi cues; see [Verdandi issue #21](https://github.com/Golden-Bastion/verdandi/issues/21). Boss/major-encounter stingers and additional authored event cues remain future work.

## Non-musical cues

AudioCueQueue is the host-side boundary for one-shot and looping cues. It resolves host-provided asset paths through Bevy's AssetServer, selects one-shot despawn or looping playback, and applies the independent sound-effects gain. No sound assets are bundled or automatically downloaded by this foundation.

The queue currently establishes the playback path, but no gameplay systems enqueue cues yet. Bevy's Vorbis feature is enabled for OGG-family assets. Actual footsteps, impacts, UI sounds, rain, wind, fire, cave ambience, and settlement beds still need selected assets and explicit provenance/licensing records. Non-musical playback remains intentionally outside Verdandi's current remit; see [Verdandi issue #20](https://github.com/Golden-Bastion/verdandi/issues/20).

## Preferences and persistence

AudioSettings is stored separately from campaign saves in config/audio_settings.json. Defaults are 70% music and 80% sound effects. Both values are clamped to 0–100% and adjusted in five-point increments from the Options > Audio tab.

Music gain is applied to the Verdandi MusicTrack through Bevy's AudioSink; sound-effects/ambience gain is applied to host-owned cue entities. This keeps the two controls independent. Verdandi's first-class controller-level pause, gain, and fade API is tracked in [Verdandi issue #19](https://github.com/Golden-Bastion/verdandi/issues/19); TEUS currently uses the exposed Bevy sink for music gain and Verdandi's existing preset crossfade for transitions.

## Licensing

No soundtrack or effect files are introduced by this implementation. Before adding audio assets, record the exact source, author, license, attribution requirements, modification terms, and redistribution permissions. Do not treat an asset as distributable merely because it is free to download.
