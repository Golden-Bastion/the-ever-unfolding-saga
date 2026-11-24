//! State-driven audio integration. Verdandi owns procedural music; Bevy owns
//! host-supplied one-shot and looping audio assets.

use std::collections::HashMap;

use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, PlaybackSettings, Volume};
use bevy::prelude::*;
use verdandi::prelude::{
    MusicConfig, MusicController, MusicPace, MusicPreset, MusicStinger, ProceduralMusicPlugin,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AudioCueRequest {
    OneShot { asset_path: String },
    StartAmbience { key: String, asset_path: String },
    StopAmbience { key: String },
}

#[derive(Resource, Default)]
pub(crate) struct AudioCueQueue {
    pending: Vec<AudioCueRequest>,
    active_ambience: HashMap<String, Entity>,
}

impl AudioCueQueue {
    #[allow(dead_code)]
    pub(crate) fn play_effect(&mut self, asset_path: impl Into<String>) {
        self.pending.push(AudioCueRequest::OneShot {
            asset_path: asset_path.into(),
        });
    }

    #[allow(dead_code)]
    pub(crate) fn start_ambience(&mut self, key: impl Into<String>, asset_path: impl Into<String>) {
        self.pending.push(AudioCueRequest::StartAmbience {
            key: key.into(),
            asset_path: asset_path.into(),
        });
    }

    #[allow(dead_code)]
    pub(crate) fn stop_ambience(&mut self, key: impl Into<String>) {
        self.pending
            .push(AudioCueRequest::StopAmbience { key: key.into() });
    }
}

#[derive(Component)]
struct AudioBusMember;

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
    .init_resource::<AudioCueQueue>()
    .add_systems(
        Update,
        (sync_music_context, process_audio_cues, apply_audio_volumes).chain(),
    );
}

fn sync_music_context(
    lifecycle: Res<LifecycleState>,
    combat: Res<CombatState>,
    controller: Res<MusicController>,
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

fn process_audio_cues(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    mut queue: ResMut<AudioCueQueue>,
) {
    for request in std::mem::take(&mut queue.pending) {
        match request {
            AudioCueRequest::OneShot { asset_path } => {
                let source = asset_server.load(asset_path);
                commands.spawn((
                    AudioPlayer::new(source),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(settings.sfx_gain())),
                    AudioBusMember,
                ));
            }
            AudioCueRequest::StartAmbience { key, asset_path } => {
                if let Some(previous) = queue.active_ambience.remove(&key) {
                    commands.entity(previous).despawn();
                }
                let source = asset_server.load(asset_path);
                let entity = commands
                    .spawn((
                        AudioPlayer::new(source),
                        PlaybackSettings::LOOP.with_volume(Volume::Linear(settings.sfx_gain())),
                        AudioBusMember,
                    ))
                    .id();
                queue.active_ambience.insert(key, entity);
            }
            AudioCueRequest::StopAmbience { key } => {
                if let Some(entity) = queue.active_ambience.remove(&key) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn apply_audio_volumes(
    settings: Res<AudioSettings>,
    mut sinks: Query<(
        &mut AudioSink,
        Option<&verdandi::prelude::MusicTrack>,
        Option<&AudioBusMember>,
    )>,
) {
    for (mut sink, music_track, audio_bus_member) in &mut sinks {
        let gain = if music_track.is_some() {
            settings.music_gain()
        } else if audio_bus_member.is_some() {
            settings.sfx_gain()
        } else {
            continue;
        };
        sink.set_volume(Volume::Linear(gain));
    }
}

#[cfg(test)]
#[path = "../tests/unit/bevy_audio_manager.rs"]
mod tests;
