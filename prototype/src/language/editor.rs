#[derive(Debug)]
pub enum EditorInputEvent {
    Character { ch: char },
    MoveCursorLeft,
    MoveCursorRight,
    RemoveCharacterLeft,
}