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

    pub fn update(&mut self, event: EditorInputEvent) {
        match event {
            EditorInputEvent::Insert { ch } => {
                self.insert(ch);
            }
            EditorInputEvent::MoveCursorLeft => {
                self.move_cursor_left();
            }
            EditorInputEvent::MoveCursorRight => {
                self.move_cursor_right();
            }
            event => {
                todo!("`{event:?}` is not supported yet.");
            }
        }
    }

    fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.move_cursor_right();
    }

    fn move_cursor_left(&mut self) {
        loop {
            self.cursor = self.cursor.saturating_sub(1);

            if self.buffer.is_char_boundary(self.cursor) {
                break;
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        loop {
            self.cursor = self.cursor.saturating_add(1);

            if self.buffer.is_char_boundary(self.cursor) {
                break;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::{EditorInput, EditorInputEvent};

    #[test]
    fn insert() {
        let mut input = EditorInput::empty();

        input.update(EditorInputEvent::Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");

        input.update(EditorInputEvent::Insert { ch: '2' });
        assert_eq!(input.buffer(), "12");
    }

    #[test]
    fn insert_at_cursor() {
        let mut input = EditorInput::empty();

        input.update(EditorInputEvent::Insert { ch: '2' });
        assert_eq!(input.buffer(), "2");

        input.update(EditorInputEvent::MoveCursorLeft);
        input.update(EditorInputEvent::Insert { ch: '1' });
        assert_eq!(input.buffer(), "12");

        input.update(EditorInputEvent::MoveCursorRight);
        input.update(EditorInputEvent::Insert { ch: '7' });
        assert_eq!(input.buffer(), "127");
    }

    #[test]
    fn move_cursor_over_non_ascii_characters() {
        let mut input = EditorInput::empty();

        input.update(EditorInputEvent::Insert { ch: '横' });
        assert_eq!(input.buffer(), "横");

        // Inserting involves moving the cursor right. If that wasn't done
        // correctly for the previous insertion, this one will panic.
        input.update(EditorInputEvent::Insert { ch: '码' });
        assert_eq!(input.buffer(), "横码");

        input.update(EditorInputEvent::MoveCursorLeft);
        input.update(EditorInputEvent::Insert { ch: '切' });
        assert_eq!(input.buffer(), "横切码");
    }
}
