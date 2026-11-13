//! Frontend-neutral interaction events.
//!
//! Graphical frontends translate their native input into these semantic events
//! before game and screen flows consume them.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputEvent {
    Up,
    Down,
    Home,
    End,
    SetSelection(usize),
    PageUp,
    PageDown,
    Confirm,
    Cancel,
    OpenSecondaryNavigation,
    OpenDeveloperConsole,
    Tab,
    SelectTab(usize),
    Character(char),
    Backspace,
    Delete,
}
