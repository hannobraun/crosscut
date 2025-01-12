#[derive(Debug)]
pub enum EditorInput {
    Char { value: char },
    Enter,
}
