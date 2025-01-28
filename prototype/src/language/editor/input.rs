#[derive(Debug, Eq, PartialEq)]
pub struct EditorInput {
    buffer: String,
    cursor: usize,
}

impl EditorInput {
    pub fn new(buffer: String) -> Self {
        let cursor = buffer.chars().count();
        Self { buffer, cursor }
    }

    pub fn empty() -> Self {
        Self::new(String::new())
    }

    pub fn buffer(&self) -> &String {
        &self.buffer
    }

    pub fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.move_cursor_right();
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor = self.cursor.saturating_add(1);
    }

    pub fn remove_left(&mut self) {
        self.buffer.pop();
    }
}

#[derive(Debug)]
pub enum EditorInputEvent {
    Insert { ch: char },
    MoveCursorLeft,
    MoveCursorRight,
    RemoveCharacterLeft,
}
