use super::{CharacterView, NpcView};

#[test]
fn display_name_includes_nonempty_title_only() {
    let character = CharacterView {
        name: "Ash".to_string(),
        title: "Wanderer".to_string(),
        ..Default::default()
    };
    let npc = NpcView {
        name: "Mara".to_string(),
        title: "Keeper".to_string(),
        ..Default::default()
    };
    let untitled_character = CharacterView {
        name: "Ash".to_string(),
        ..Default::default()
    };
    let untitled_npc = NpcView {
        name: "Mara".to_string(),
        ..Default::default()
    };

    assert_eq!(character.display_name(), "Ash the Wanderer");
    assert_eq!(npc.display_name(), "Mara the Keeper");
    assert_eq!(untitled_character.display_name(), "Ash");
    assert_eq!(untitled_npc.display_name(), "Mara");
}
