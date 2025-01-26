#[derive(Debug)]
pub enum TerminalInputEvent {
    Character { ch: char },

    Backspace,
    Enter,
    Left,
    Right,
    Escape,
}
