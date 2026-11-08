use super::*;

#[test]
fn interaction_targets_have_stable_distinct_keys() {
    assert_eq!(InteractionTarget::Npc(7).stable_key(), "npc:7");
    assert_eq!(InteractionTarget::Corpse(7).stable_key(), "corpse:7");
    assert_eq!(
        InteractionTarget::PointOfInterest("poi.shrine".into()).stable_key(),
        "poi:poi.shrine"
    );
    assert_eq!(
        InteractionTarget::Passage {
            id: "exit.north".into(),
            destination_id: 12
        }
        .stable_key(),
        "passage:exit.north:12"
    );
}
