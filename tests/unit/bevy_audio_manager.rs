use super::*;
use std::path::PathBuf;

fn active_lifecycle() -> LifecycleState {
    let state = crate::model::create_new_state(
        "Audio Test World",
        crate::model::WorldMode::New,
        "Test Walker".into(),
        "the Listener".into(),
    );
    let mut lifecycle = LifecycleState::default();
    lifecycle.phase = LifecyclePhase::Complete;
    lifecycle.session = Some(crate::bevy_lifecycle::GameSession {
        state,
        save_path: PathBuf::new(),
    });
    lifecycle
}

#[test]
fn contexts_select_distinct_musical_arrangements() {
    let title = MusicContext::Title.arrangement();
    let exploration = MusicContext::Exploration.arrangement();
    let settlement = MusicContext::Settlement.arrangement();
    let danger = MusicContext::Danger.arrangement();
    let combat = MusicContext::Combat.arrangement();

    assert_eq!(title.0, MusicPreset::MythicAmbient);
    assert_eq!(exploration.0, MusicPreset::MythicAmbient);
    assert_eq!(settlement.0, MusicPreset::ClassicalAmbient);
    assert_eq!(danger.0, MusicPreset::HeavyMetal);
    assert_eq!(combat.0, MusicPreset::LegendaryMetal);
    assert!(title.2 < exploration.2);
    assert_ne!(exploration.2, settlement.2);
    assert!(danger.2 < combat.2);
    assert_eq!(danger.1, MusicPace::Fast);
    assert_eq!(combat.1, MusicPace::Fast);
}

#[test]
fn settlement_layout_matching_is_explicit() {
    assert!(is_settlement_layout(Some("settlement")));
    assert!(is_settlement_layout(Some("village")));
    assert!(is_settlement_layout(Some("town")));
    assert!(is_settlement_layout(Some("hollow_market_square")));
    assert!(!is_settlement_layout(Some("wilderness")));
    assert!(!is_settlement_layout(None));
}

#[test]
fn context_resolution_uses_existing_lifecycle_and_world_state() {
    let title = resolve_context(&LifecycleState::default(), &CombatState::default());
    assert_eq!(title, (MusicContext::Title, 1));

    let mut lifecycle = active_lifecycle();
    let combat = CombatState::default();
    let expected_seed = lifecycle
        .session
        .as_ref()
        .expect("session exists")
        .state
        .world
        .generation
        .expect("new world has generation metadata")
        .seed;
    assert_eq!(
        resolve_context(&lifecycle, &combat),
        (MusicContext::Exploration, expected_seed)
    );
    lifecycle.phase = LifecyclePhase::QuitConfirm;
    assert_eq!(
        resolve_context(&lifecycle, &combat).0,
        MusicContext::Exploration
    );
    lifecycle.phase = LifecyclePhase::Complete;

    let state = &mut lifecycle.session.as_mut().expect("session exists").state;
    let current_location = state.character.location_id;
    let location = state
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == current_location)
        .expect("new character starts in a world location");
    location.dangerous = false;
    location.spatial_layout = Some("hollow_market_square".into());
    assert_eq!(
        resolve_context(&lifecycle, &combat).0,
        MusicContext::Settlement
    );

    let state = &mut lifecycle.session.as_mut().expect("session exists").state;
    let current_location = state.character.location_id;
    state
        .world
        .locations
        .iter_mut()
        .find(|location| location.id == current_location)
        .expect("current location exists")
        .dangerous = true;
    assert_eq!(resolve_context(&lifecycle, &combat).0, MusicContext::Danger);
}

#[test]
fn outdoor_wind_loop_crossfade_keeps_the_loop_boundary_continuous() {
    let source = (0..10)
        .flat_map(|frame| [frame as f32, frame as f32])
        .collect::<Vec<_>>();
    let looped = crossfade_loop(&source, 2).expect("valid stereo loop");

    assert_eq!(looped.len() / 2, 8);
    assert_eq!(&looped[..2], &[2.0, 2.0]);
    assert_eq!(&looped[looped.len() - 2..], &[1.0, 1.0]);
    assert!((looped[looped.len() - 2] - looped[0]).abs() <= 1.0);
}

#[test]
fn outdoor_wind_loop_rejects_invalid_crossfade_windows() {
    assert!(crossfade_loop(&[0.0; 12], 1).is_none());
    assert!(crossfade_loop(&[0.0; 8], 0).is_none());
    assert!(crossfade_loop(&[0.0; 8], 2).is_none());
}
