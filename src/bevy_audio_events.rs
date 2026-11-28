//! Semantic gameplay audio requests and Verdandi-backed realistic sound generation.
//!
//! Gameplay adapters enqueue events; this module selects Realistic generators, prepares short
//! PCM clips, and submits them to Verdandi's unified controller. No gameplay system spawns audio.

use std::collections::VecDeque;
use std::sync::Arc;

use bevy::prelude::*;
use verdandi::prelude::{
    realistic::{
        DamageEffectConfig, DamageEffectGenerator, FootstepConfig, FootstepGenerator,
        FootstepSurface, WeaponHandlingConfig, WeaponHandlingGenerator, WeaponImpactConfig,
        WeaponImpactGenerator,
    },
    AudioClip, AudioController, DamageEffectKind, ImpactMaterial, ProceduralGenerator,
    SoundCategory, WeaponHandlingKind, WeaponHandlingMaterial, WeaponImpactKind,
};

const OUTPUT_SAMPLE_RATE: u32 = 44_100;
const MAX_PENDING_EVENTS: usize = 64;
const MAX_GENERATION_CHUNKS: usize = 100;
const GENERATION_CHUNK_SAMPLES: usize = 1_024;
const CLIP_RETENTION_MARGIN_SECONDS: f32 = 0.35;

struct RetainedAudioClip {
    _clip: Arc<AudioClip>,
    remaining_seconds: f32,
}

#[derive(Resource, Default)]
struct AudioClipRetention(VecDeque<RetainedAudioClip>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioEvent {
    Footstep {
        surface: FootstepSurface,
        seed: u64,
    },
    WeaponImpact {
        kind: WeaponImpactKind,
        material: ImpactMaterial,
        force: u8,
        seed: u64,
    },
    DamageEffect {
        kind: DamageEffectKind,
        force: u8,
        size: u8,
        seed: u64,
    },
    WeaponHandling {
        kind: WeaponHandlingKind,
        material: WeaponHandlingMaterial,
        seed: u64,
    },
}

/// Bounded queue shared by gameplay event producers and the Verdandi adapter.
#[derive(Resource, Default)]
pub(crate) struct AudioEventQueue(VecDeque<AudioEvent>);

impl AudioEventQueue {
    pub(crate) fn footstep(&mut self, surface: FootstepSurface, seed: u64) {
        self.push(AudioEvent::Footstep { surface, seed });
    }

    pub(crate) fn weapon_impact(
        &mut self,
        kind: WeaponImpactKind,
        material: ImpactMaterial,
        force: f32,
        seed: u64,
    ) {
        self.push(AudioEvent::WeaponImpact {
            kind,
            material,
            force: (force.clamp(0.0, 1.0) * 100.0).round() as u8,
            seed,
        });
    }

    pub(crate) fn damage_effect(
        &mut self,
        kind: DamageEffectKind,
        force: f32,
        size: f32,
        seed: u64,
    ) {
        self.push(AudioEvent::DamageEffect {
            kind,
            force: (force.clamp(0.0, 1.0) * 100.0).round() as u8,
            size: (size.clamp(0.0, 1.0) * 100.0).round() as u8,
            seed,
        });
    }

    pub(crate) fn weapon_handling(
        &mut self,
        kind: WeaponHandlingKind,
        material: WeaponHandlingMaterial,
        seed: u64,
    ) {
        self.push(AudioEvent::WeaponHandling {
            kind,
            material,
            seed,
        });
    }

    fn push(&mut self, event: AudioEvent) {
        if self.0.len() < MAX_PENDING_EVENTS {
            self.0.push_back(event);
        } else {
            warn!("Gameplay audio event queue is full; dropping the newest cue");
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<AudioEventQueue>()
        .init_resource::<AudioClipRetention>()
        .add_systems(Update, process_audio_events);
}

fn process_audio_events(
    mut events: ResMut<AudioEventQueue>,
    controller: Res<AudioController>,
    time: Res<Time>,
    mut retention: ResMut<AudioClipRetention>,
) {
    let delta = time.delta_secs().max(0.0);
    for retained in &mut retention.0 {
        retained.remaining_seconds -= delta;
    }
    retention
        .0
        .retain(|retained| retained.remaining_seconds > 0.0);

    for event in std::mem::take(&mut events.0) {
        match event {
            AudioEvent::Footstep { surface, seed } => {
                let config = FootstepConfig {
                    sample_rate: OUTPUT_SAMPLE_RATE,
                    intensity: 0.68,
                    brightness: 0.46,
                };
                match FootstepGenerator::new_for_surface(config, surface, seed) {
                    Ok(generator) => play_generated(generator, &controller, &mut retention),
                    Err(error) => warn!("Could not prepare realistic footstep: {error}"),
                }
            }
            AudioEvent::WeaponImpact {
                kind,
                material,
                force,
                seed,
            } => {
                let config = WeaponImpactConfig {
                    sample_rate: OUTPUT_SAMPLE_RATE,
                    intensity: 0.76,
                    force: f32::from(force) / 100.0,
                    material,
                    kind,
                };
                match WeaponImpactGenerator::new(config, seed) {
                    Ok(generator) => play_generated(generator, &controller, &mut retention),
                    Err(error) => warn!("Could not prepare realistic weapon impact: {error}"),
                }
            }
            AudioEvent::DamageEffect {
                kind,
                force,
                size,
                seed,
            } => {
                let config = DamageEffectConfig {
                    sample_rate: OUTPUT_SAMPLE_RATE,
                    intensity: 0.66,
                    force: f32::from(force) / 100.0,
                    size: f32::from(size) / 100.0,
                    kind,
                };
                match DamageEffectGenerator::new(config, seed) {
                    Ok(generator) => play_generated(generator, &controller, &mut retention),
                    Err(error) => warn!("Could not prepare realistic damage effect: {error}"),
                }
            }
            AudioEvent::WeaponHandling {
                kind,
                material,
                seed,
            } => {
                let config = WeaponHandlingConfig {
                    sample_rate: OUTPUT_SAMPLE_RATE,
                    intensity: 0.62,
                    weight: 0.62,
                    speed: 0.58,
                    material,
                    kind,
                };
                match WeaponHandlingGenerator::new(config, seed) {
                    Ok(generator) => play_generated(generator, &controller, &mut retention),
                    Err(error) => warn!("Could not prepare realistic weapon handling: {error}"),
                }
            }
        }
    }
}

fn play_generated<G: ProceduralGenerator>(
    mut generator: G,
    controller: &AudioController,
    retention: &mut AudioClipRetention,
) {
    let mut samples = Vec::with_capacity(OUTPUT_SAMPLE_RATE as usize);
    let mut chunk = [0.0_f32; GENERATION_CHUNK_SAMPLES];

    for _ in 0..MAX_GENERATION_CHUNKS {
        if generator.is_finished() {
            break;
        }
        if let Err(error) = generator.render(&mut chunk) {
            warn!("Realistic sound generation failed: {error}");
            return;
        }
        samples.extend_from_slice(&chunk);
    }

    if samples.is_empty() {
        return;
    }

    let clip = match AudioClip::from_samples(OUTPUT_SAMPLE_RATE, 2, samples) {
        Ok(clip) => Arc::new(clip),
        Err(error) => {
            warn!("Could not create Verdandi audio clip: {error}");
            return;
        }
    };
    match controller.play_one_shot(clip.clone(), SoundCategory::Effects, 1.0) {
        Ok(()) => retention.0.push_back(RetainedAudioClip {
            remaining_seconds: clip.duration().as_secs_f32() + CLIP_RETENTION_MARGIN_SECONDS,
            _clip: clip,
        }),
        Err(error) => debug!("Verdandi rejected a gameplay sound request: {error:?}"),
    }
}

#[cfg(test)]
#[path = "../tests/unit/bevy_audio_events.rs"]
mod tests;
