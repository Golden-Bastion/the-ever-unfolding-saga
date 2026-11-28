# Audio System

## Responsibility

The audio manager is a Bevy presentation adapter. It consumes authoritative lifecycle, combat, threat, and location state, then selects audio presentation. It does not own gameplay rules or alter simulation state.

The ownership boundary is now:

- **Verdandi** owns procedural music, procedural sound generation, the sound mixer, one-shots, keyed loops, fades, and the independent music/effects/ambience buses.
- **Bevy** supplies application audio output through Verdandi's Bevy 0.20 plugin. TEUS does not create separate Bevy audio players, maintain a parallel cue queue, or implement its own sound mixer.
- **TEUS** resolves game context, decides when gameplay events should produce audio, owns persistent player preferences, and supplies any future decoded PCM clips to Verdandi.

The integrated dependency is Verdandi **0.9.0**, pinned to commit `d93a3cd` and licensed under Apache-2.0. The plugin exposes one `AudioController` for music, effects, and ambience. Verdandi's procedural generators are engine-independent and do not require recorded sound assets.

## Music context resolution

The manager resolves the current context from existing authoritative state:

| Context | Resolution | Verdandi arrangement |
|---|---|---|
| Title | No active campaign session or lifecycle outside active play | Ethereal, low intensity |
| Exploration | Active campaign in an ordinary location | Ethereal, moderate intensity |
| Settlement | Active campaign in a location using an authored settlement layout ID | Contemplative |
| Danger | Active world threat or a location marked dangerous | Warbound, fast pace |
| Combat | Combat presentation is active, including its result screen | Orchestral, fast pace |

The campaign seed is taken from world-generation metadata, then the geography seed, then the saved game RNG state. The title context uses a stable fallback seed. Context changes request Verdandi preset transitions and set context-specific intensity.

Combat start plays Verdandi's `CombatStart` stinger. A successful combat result plays `Victory`. Defeat/retreat and major-encounter stingers remain future event mappings.

## Effects and environmental ambience

Verdandi 0.9.0 supplies Retro and Realistic procedural sound families, including footsteps, impacts, breakage, weapon/equipment handling, object/tool/mechanical actions, wind, and rain. Its `AudioClip`, `AudioController`, and `SoundCategory` APIs provide resident one-shots, keyed loops, fades, and bounded voice management. Environmental acoustics can add distance, occlusion, and room reflections.

The previous TEUS-owned `AudioCueQueue`, `AudioPlayer` spawning, `AudioSink` volume routing, and separate Bevy ambience lifecycle have been removed. Music, effects, and ambience now use Verdandi's single controller and mixer.

The procedural generators and mixer are available, but gameplay cue production is not yet connected to footsteps, combat events, weather, or location ambience. Gameplay systems should emit semantic audio requests; the audio adapter will select the appropriate Verdandi generator/profile and submit prepared PCM. Do not reintroduce direct Bevy audio playback into gameplay systems.

For authored audio in the future, the host remains responsible for loading/decoding source files off the audio callback, constructing Verdandi `AudioClip` values, and retaining their `Arc` ownership while queued or playing. No recorded audio assets or external sample libraries are bundled.

## Preferences and persistence

Audio preferences are stored separately from campaign saves in `config/audio_settings.json`. Music defaults to 70%; effects and ambience each default to 80%. Values are clamped to 0–100% and adjusted in five-point increments from Options > Audio.

Existing settings files containing only music and SFX values remain valid. The new ambience preference uses the established 80% default when absent. All three values are applied through Verdandi's `AudioBus::{Music, Effects, Ambience}` controls.

## Build environment

Verdandi's Bevy plugin uses Bevy's audio output backend. Linux CI therefore still needs ALSA development headers even though TEUS no longer owns Bevy one-shot/loop playback. The cache-warmup workflow installs the same native headers as PR validation; this is a platform build prerequisite, not a second audio implementation.
