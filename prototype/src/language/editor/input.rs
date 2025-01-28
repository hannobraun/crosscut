use std::cmp::min;

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
        use EditorInputEvent::*;

        match event {
            Insert { ch } => {
                self.insert(ch);
            }
            MoveCursorLeft => {
                self.move_cursor_left();
            }
            MoveCursorRight => {
                self.move_cursor_right();
            }
            RemoveCharacterLeft => {
                self.remove_left();
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

    fn move_cursor_right(&mut self) {
        loop {
            self.cursor = min(self.cursor.saturating_add(1), self.buffer.len());

            if self.buffer.is_char_boundary(self.cursor) {
                break;
            }

            assert!(
                self.cursor < self.buffer.len(),
                "Moved cursor right, and not at char boundary. This means \
                cursor must still be in bounds, and we're not risking an \
                endless loop.",
            );
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
    fn remove_left() {
        let mut input = EditorInput::empty();

        input.update(EditorInputEvent::Insert { ch: '1' });
        input.update(EditorInputEvent::Insert { ch: '2' });
        assert_eq!(input.buffer(), "12");

        input.update(EditorInputEvent::RemoveCharacterLeft);
        assert_eq!(input.buffer(), "1");

        input.update(EditorInputEvent::RemoveCharacterLeft);
        assert_eq!(input.buffer(), "");
    }

    #[test]
    fn move_left_while_already_at_leftmost_position() {
        let mut input = EditorInput::empty();

        input.update(EditorInputEvent::MoveCursorLeft);
        input.update(EditorInputEvent::Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_right_while_already_at_rightmost_position() {
        let mut input = EditorInput::empty();

        input.update(EditorInputEvent::MoveCursorRight);
        input.update(EditorInputEvent::Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_cursor_over_non_ascii_characters() {
        let mut input = EditorInput::empty();

        input.update(EditorInputEvent::Insert { ch: '横' });
        assert_eq!(input.buffer(), "横");

        // Inserting involves moving the cursor right. If that wasn't done
        // correctly for the previous insertion, this one will panic.
        //
        // It's a bit weird to only test `MoveCursorRight` implicitly like this,
        // but if we rewrite this test to look more like the `insert_at_cursor`
        // test above, we wouldn't actually test the correct behavior of
        // `MoveCursorLeft`. There, its effect is undone, before inserting a new
        // character would make sure that it actually moved to a character
        // boundary.
        input.update(EditorInputEvent::Insert { ch: '码' });
        assert_eq!(input.buffer(), "横码");

        input.update(EditorInputEvent::MoveCursorLeft);
        input.update(EditorInputEvent::Insert { ch: '切' });
        assert_eq!(input.buffer(), "横切码");
    }
}
