#[derive(Debug)]
pub enum InputEvent {
    Char { value: char },
    Enter,
}
