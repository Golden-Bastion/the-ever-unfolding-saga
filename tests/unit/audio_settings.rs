use super::*;

#[test]
fn volume_adjustments_are_bounded_and_independent() {
    let mut settings = AudioSettings::default();
    for _ in 0..30 {
        settings.adjust_music_volume(VOLUME_STEP);
        settings.adjust_sfx_volume(-VOLUME_STEP);
    }
    assert_eq!(settings.music_volume_percent(), MAX_VOLUME_PERCENT);
    assert_eq!(settings.sfx_volume_percent(), MIN_VOLUME_PERCENT);
    assert!(!settings.adjust_music_volume(VOLUME_STEP));
    assert!(!settings.adjust_sfx_volume(-VOLUME_STEP));
}

#[test]
fn reset_restores_both_audio_preferences() {
    let mut settings = AudioSettings {
        music_volume_percent: 10,
        sfx_volume_percent: 95,
    };
    settings.reset();
    assert_eq!(settings, AudioSettings::default());
}

#[test]
fn loaded_values_are_clamped_to_supported_ranges() {
    let mut settings = AudioSettings {
        music_volume_percent: 255,
        sfx_volume_percent: 0,
    };
    settings.normalize();
    assert_eq!(settings.music_volume_percent(), MAX_VOLUME_PERCENT);
    assert_eq!(settings.sfx_volume_percent(), MIN_VOLUME_PERCENT);
}

#[test]
fn category_gains_remain_independent() {
    let settings = AudioSettings {
        music_volume_percent: 25,
        sfx_volume_percent: 80,
    };
    assert!((settings.music_gain() - 0.25).abs() < f32::EPSILON);
    assert!((settings.sfx_gain() - 0.8).abs() < f32::EPSILON);
}

#[test]
fn settings_round_trip_through_json() {
    let settings = AudioSettings {
        music_volume_percent: 35,
        sfx_volume_percent: 65,
    };
    let encoded = serde_json::to_vec(&settings).expect("audio settings should serialize");
    let decoded: AudioSettings =
        serde_json::from_slice(&encoded).expect("audio settings should deserialize");
    assert_eq!(decoded, settings);
}

#[test]
fn defaults_keep_music_and_effects_independent() {
    let settings = AudioSettings::default();
    assert_eq!(settings.music_volume_percent(), 70);
    assert_eq!(settings.sfx_volume_percent(), 80);
}
