//! State-driven audio integration. Verdandi owns music, effects, ambience,
//! procedural generation, mixing, and playback; TEUS maps game state to controls.

use bevy::prelude::*;
use verdandi::prelude::{
    AudioBus, AudioController, MusicConfig, MusicPace, MusicPreset, MusicStinger,
    ProceduralMusicPlugin,
};

use crate::audio_settings::AudioSettings;
use crate::bevy_combat::CombatState;
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
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

pub(crate) fn install(app: &mut App) {
    app.add_plugins(ProceduralMusicPlugin::new(MusicConfig {
        auto_intensity: false,
        intensity: 0.22,
        ..default()
    }))
    .init_resource::<AudioRuntimeState>()
    .add_systems(Update, (sync_music_context, apply_audio_buses).chain());
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
