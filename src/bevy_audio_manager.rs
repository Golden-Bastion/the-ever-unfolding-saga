//! State-driven audio integration. Verdandi owns music, effects, ambience,
//! procedural generation, mixing, and playback; TEUS maps game state to controls.

use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use verdandi::prelude::{
    AudioBus, AudioController, MusicConfig, MusicPace, MusicPreset, MusicStinger,
    ProceduralGenerator, ProceduralMusicPlugin, AudioClip, LoopKey, SoundCategory,
    realistic::{WindConfig, WindGenerator},
};

use crate::audio_settings::AudioSettings;
use crate::bevy_combat::CombatState;
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{NavigationState, ScreenId};
use crate::bevy_spatial::SpatialExplorationState;
use crate::game::combat::CombatResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MusicContext {
    Title,
    Exploration,
    Settlement,
    Danger,
    Combat,
}

impl MusicContext {
    fn arrangement(self) -> (MusicPreset, MusicPace, f32) {
        match self {
            Self::Title => (MusicPreset::MythicAmbient, MusicPace::Standard, 0.22),
            Self::Exploration => (MusicPreset::MythicAmbient, MusicPace::Standard, 0.48),
            Self::Settlement => (MusicPreset::ClassicalAmbient, MusicPace::Standard, 0.32),
            Self::Danger => (MusicPreset::HeavyMetal, MusicPace::Fast, 0.78),
            Self::Combat => (MusicPreset::LegendaryMetal, MusicPace::Fast, 1.0),
        }
    }
}

#[derive(Resource)]
struct AudioRuntimeState {
    context: Option<MusicContext>,
    seed: Option<u64>,
    last_combat_result: Option<CombatResult>,
}

impl Default for AudioRuntimeState {
    fn default() -> Self {
        Self {
            context: Some(MusicContext::Title),
            seed: Some(1),
            last_combat_result: None,
        }
    }
}

#[derive(Resource, Default)]
struct WindAmbienceRuntime {
    active_seed: Option<u64>,
    prepared_seed: Option<u64>,
    clip: Option<Arc<AudioClip>>,
}

const WIND_LOOP_KEY: LoopKey = LoopKey(0x5445_5553_5749_4E44);
const OUTPUT_SAMPLE_RATE: u32 = 44_100;
const WIND_LOOP_SECONDS: usize = 10;
const WIND_CROSSFADE_FRAMES: usize = OUTPUT_SAMPLE_RATE as usize / 2;

pub(crate) fn install(app: &mut App) {
    app.add_plugins(ProceduralMusicPlugin::new(MusicConfig {
        auto_intensity: false,
        intensity: 0.22,
        ..default()
    }))
    .init_resource::<AudioRuntimeState>()
    .init_resource::<WindAmbienceRuntime>()
    .add_systems(
        Update,
        (sync_music_context, apply_audio_buses, sync_outdoor_wind_ambience).chain(),
    );
}

fn sync_music_context(
    lifecycle: Res<LifecycleState>,
    combat: Res<CombatState>,
    controller: Res<AudioController>,
    mut runtime: ResMut<AudioRuntimeState>,
) {
    match combat.result_outcome() {
        Some(outcome) if runtime.last_combat_result != Some(outcome) => {
            if outcome == CombatResult::Victory {
                controller.play_stinger(MusicStinger::Victory);
            }
            runtime.last_combat_result = Some(outcome);
        }
        None => runtime.last_combat_result = None,
        _ => {}
    }

    let (context, seed) = resolve_context(&lifecycle, &combat);
    if runtime.seed != Some(seed) {
        controller.set_seed(seed);
        runtime.seed = Some(seed);
    }
    if runtime.context == Some(context) {
        return;
    }

    let previous = runtime.context;
    let (preset, pace, intensity) = context.arrangement();
    controller.set_auto_intensity(false);
    controller.set_intensity(intensity);
    controller.set_preset_immediate(preset, pace);
    if context == MusicContext::Combat && previous != Some(MusicContext::Combat) {
        controller.play_stinger(MusicStinger::CombatStart);
    }
    runtime.context = Some(context);
}

fn apply_audio_buses(settings: Res<AudioSettings>, controller: Res<AudioController>) {
    if !settings.is_changed() {
        return;
    }

    controller.set_bus_volume(AudioBus::Music, settings.music_gain());
    controller.set_bus_volume(AudioBus::Effects, settings.sfx_gain());
    controller.set_bus_volume(AudioBus::Ambience, settings.ambience_gain());
}

fn sync_outdoor_wind_ambience(
    navigation: Res<NavigationState>,
    gameplay: Res<crate::bevy_gameplay::GameplayRuntimeState>,
    spatial: Res<SpatialExplorationState>,
    lifecycle: Res<LifecycleState>,
    controller: Res<AudioController>,
    mut runtime: ResMut<WindAmbienceRuntime>,
) {
    let should_play = navigation.current_screen == Some(ScreenId::SpatialExploration)
        && !gameplay.is_paused()
        && spatial.outdoor_ambience_active();
    if !should_play {
        if runtime.active_seed.is_some() {
            match controller.stop_loop(WIND_LOOP_KEY, Duration::from_millis(1_500)) {
                Ok(()) => runtime.active_seed = None,
                Err(error) => debug!("Verdandi could not stop outdoor ambience yet: {error:?}"),
            }
        }
        return;
    }

    let Some(session) = lifecycle.session.as_ref() else {
        return;
    };
    let seed = session
        .state
        .world
        .generation
        .map(|generation| generation.seed)
        .or(session.state.world.geography_seed)
        .unwrap_or(session.state.rng_state);
    if runtime.active_seed == Some(seed) {
        return;
    }

    if runtime.prepared_seed != Some(seed) {
        let Some(clip) = create_wind_loop(seed) else {
            warn!("Could not prepare realistic outdoor wind ambience");
            return;
        };
        runtime.clip = Some(clip);
        runtime.prepared_seed = Some(seed);
    }
    let Some(clip) = runtime.clip.clone() else {
        return;
    };
    match controller.play_loop(
        WIND_LOOP_KEY,
        clip,
        SoundCategory::Ambience,
        0.24,
        Duration::from_millis(1_800),
    ) {
        Ok(()) => runtime.active_seed = Some(seed),
        Err(error) => debug!("Verdandi rejected outdoor ambience: {error:?}"),
    }
}

fn create_wind_loop(seed: u64) -> Option<Arc<AudioClip>> {
    let config = WindConfig {
        sample_rate: OUTPUT_SAMPLE_RATE,
        intensity: 0.28,
        brightness: 0.32,
    };
    let mut generator = WindGenerator::new(config, seed).ok()?;
    let mut source = vec![0.0_f32; WIND_LOOP_SECONDS * OUTPUT_SAMPLE_RATE as usize * 2];
    generator.render(&mut source).ok()?;
    let samples = crossfade_loop(&source, WIND_CROSSFADE_FRAMES)?;
    AudioClip::from_samples(OUTPUT_SAMPLE_RATE, 2, samples)
        .ok()
        .map(Arc::new)
}

fn crossfade_loop(source: &[f32], fade_frames: usize) -> Option<Vec<f32>> {
    if !source.len().is_multiple_of(2) || fade_frames < 2 {
        return None;
    }
    let frame_count = source.len() / 2;
    if frame_count <= fade_frames * 2 {
        return None;
    }

    let mut output = Vec::with_capacity((frame_count - fade_frames) * 2);
    output.extend_from_slice(&source[fade_frames * 2..(frame_count - fade_frames) * 2]);
    for frame in 0..fade_frames {
        let blend = frame as f32 / (fade_frames - 1) as f32;
        let tail = (frame_count - fade_frames + frame) * 2;
        let head = frame * 2;
        for channel in 0..2 {
            output.push(source[tail + channel] * (1.0 - blend) + source[head + channel] * blend);
        }
    }
    Some(output)
}

fn resolve_context(lifecycle: &LifecycleState, combat: &CombatState) -> (MusicContext, u64) {
    let Some(session) = lifecycle.session.as_ref() else {
        return (MusicContext::Title, 1);
    };
    if !matches!(
        lifecycle.phase,
        LifecyclePhase::Complete | LifecyclePhase::QuitConfirm
    ) {
        return (MusicContext::Title, 1);
    }

    let game = &session.state;
    let seed = game
        .world
        .generation
        .map(|generation| generation.seed)
        .or(game.world.geography_seed)
        .unwrap_or(game.rng_state);
    let context = if combat.is_presenting() {
        MusicContext::Combat
    } else if game.threat.active
        || game
            .world
            .location_by_id(game.character.location_id)
            .is_some_and(|location| location.dangerous)
    {
        MusicContext::Danger
    } else if game
        .world
        .location_by_id(game.character.location_id)
        .is_some_and(|location| is_settlement_layout(location.spatial_layout.as_deref()))
    {
        MusicContext::Settlement
    } else {
        MusicContext::Exploration
    };
    (context, seed)
}

fn is_settlement_layout(layout: Option<&str>) -> bool {
    matches!(
        layout,
        Some("settlement" | "village" | "town" | "city" | "hollow_market_square")
    )
}

#[cfg(test)]
#[path = "../tests/unit/bevy_audio_manager.rs"]
mod tests;
