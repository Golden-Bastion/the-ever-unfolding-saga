use super::*;

#[test]
fn semantic_event_queue_is_bounded() {
    let mut queue = AudioEventQueue::default();
    for seed in 0..(MAX_PENDING_EVENTS as u64 + 8) {
        queue.footstep(FootstepSurface::Dirt, seed);
    }

    assert_eq!(queue.0.len(), MAX_PENDING_EVENTS);
}

#[test]
fn queue_preserves_realistic_event_parameters() {
    let mut queue = AudioEventQueue::default();
    queue.footstep(FootstepSurface::ShallowWater, 11);
    queue.weapon_impact(WeaponImpactKind::Block, ImpactMaterial::Shield, 0.65, 12);
    queue.weapon_handling(
        WeaponHandlingKind::Draw,
        WeaponHandlingMaterial::Metal,
        13,
    );

    assert_eq!(
        queue.0.pop_front(),
        Some(AudioEvent::Footstep {
            surface: FootstepSurface::ShallowWater,
            seed: 11,
        })
    );
    assert_eq!(
        queue.0.pop_front(),
        Some(AudioEvent::WeaponImpact {
            kind: WeaponImpactKind::Block,
            material: ImpactMaterial::Shield,
            force: 65,
            seed: 12,
        })
    );
    assert_eq!(
        queue.0.pop_front(),
        Some(AudioEvent::WeaponHandling {
            kind: WeaponHandlingKind::Draw,
            material: WeaponHandlingMaterial::Metal,
            seed: 13,
        })
    );
}
