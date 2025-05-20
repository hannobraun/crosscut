#[derive(Debug)]
pub enum EditorInputEvent {
    Insert { ch: char },
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorUp,
    MoveCursorDown,
    RemoveLeft { whole_node: bool },
    RemoveRight { whole_node: bool },
    Submit,
}
