use std::{cmp::min, mem};

#[derive(Debug, Eq, PartialEq)]
pub struct EditorInputBuffer {
    buffer: String,
    cursor: usize,
}

impl EditorInputBuffer {
    pub fn new(buffer: String) -> Self {
        Self { buffer, cursor: 0 }
    }

    pub fn move_cursor_to_end(&mut self) {
        // The cursor counts bytes, not characters. So the use of `len` here is
        // correct.
        self.cursor = self.buffer.len();
    }

    pub fn empty() -> Self {
        Self::new(String::new())
    }

    pub fn buffer(&self) -> &String {
        &self.buffer
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn update(&mut self, event: EditorInputEvent) -> Option<UpdateAction> {
        use EditorInputEvent::*;

        match event {
            Insert { ch } => {
                assert!(
                    !ch.is_whitespace(),
                    "Expecting whitespace characters to be translated into \
                    `SubmitNode`.",
                );

                self.insert(ch);
            }
            MoveCursorLeft => {
                return self.move_cursor_left();
            }
            MoveCursorRight => {
                self.move_cursor_right();
            }
            MoveCursorUp => {
                return Some(UpdateAction::NavigateToPrevious);
            }
            MoveCursorDown => {
                return Some(UpdateAction::NavigateToNextNode);
            }
            RemoveLeft => {
                self.remove_left();
            }
            RemoveRight => {
                self.remove_right();
            }
            SubmitNode => {
                return Some(self.submit());
            }
        }

        None
    }

    fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.move_cursor_right();
    }

    fn move_cursor_left(&mut self) -> Option<UpdateAction> {
        loop {
            if self.cursor > 0 {
                self.cursor -= 1;
            } else {
                return Some(UpdateAction::NavigateToPrevious);
            }

            if self.buffer.is_char_boundary(self.cursor) {
                break;
            }
        }

        None
    }

    fn move_cursor_right(&mut self) -> Option<UpdateAction> {
        loop {
            self.cursor = self.cursor.saturating_add(1);
            self.cursor = min(self.cursor, self.buffer.len());

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

        None
    }

    fn remove_left(&mut self) {
        if self.move_cursor_left().is_none() {
            self.buffer.remove(self.cursor);
        }
    }

    fn remove_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
    }

    fn submit(&mut self) -> UpdateAction {
        let mut old_buffer = mem::take(&mut self.buffer);
        let new_buffer = old_buffer.split_off(self.cursor);

        *self = Self::new(new_buffer);

        UpdateAction::Submit {
            submitted: old_buffer,
        }
    }
}

pub enum UpdateAction {
    NavigateToPrevious,
    NavigateToNextNode,
    Submit { submitted: String },
}

#[derive(Debug)]
pub enum EditorInputEvent {
    Insert { ch: char },
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorUp,
    MoveCursorDown,
    RemoveLeft,
    RemoveRight,
    SubmitNode,
}

#[cfg(test)]
mod tests {
    use super::{EditorInputBuffer, EditorInputEvent::*};

    #[test]
    fn insert() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");

        input.update(Insert { ch: '2' });
        assert_eq!(input.buffer(), "12");
    }

    #[test]
    fn insert_at_cursor() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '2' });
        assert_eq!(input.buffer(), "2");

        input.update(MoveCursorLeft);
        input.update(Insert { ch: '1' });
        assert_eq!(input.buffer(), "12");

        input.update(MoveCursorRight);
        input.update(Insert { ch: '7' });
        assert_eq!(input.buffer(), "127");
    }

    #[test]
    fn remove_left() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        input.update(Insert { ch: '2' });
        assert_eq!(input.buffer(), "12");

        input.update(RemoveLeft);
        assert_eq!(input.buffer(), "1");

        input.update(RemoveLeft);
        assert_eq!(input.buffer(), "");
    }

    #[test]
    fn remove_left_at_cursor() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        input.update(Insert { ch: '2' });
        assert_eq!(input.buffer(), "12");

        input.update(MoveCursorLeft);
        input.update(RemoveLeft);
        assert_eq!(input.buffer(), "2");
    }

    #[test]
    fn remove_left_while_already_at_leftmost_position() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");

        input.update(MoveCursorLeft);
        input.update(RemoveLeft);
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn remove_right() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        input.update(Insert { ch: '2' });
        assert_eq!(input.buffer(), "12");

        input.update(MoveCursorLeft);
        input.update(MoveCursorLeft);
        input.update(RemoveRight);
        assert_eq!(input.buffer(), "2");

        input.update(RemoveRight);
        assert_eq!(input.buffer(), "");
    }

    #[test]
    fn remove_right_while_already_at_rightmost_position() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");

        input.update(RemoveRight);
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_left_while_already_at_leftmost_position() {
        let mut input = EditorInputBuffer::empty();

        input.update(MoveCursorLeft);
        input.update(Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_right_while_already_at_rightmost_position() {
        let mut input = EditorInputBuffer::empty();

        input.update(MoveCursorRight);
        input.update(Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_cursor_over_non_ascii_characters() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '横' });
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
        input.update(Insert { ch: '码' });
        assert_eq!(input.buffer(), "横码");

        input.update(MoveCursorLeft);
        input.update(Insert { ch: '切' });
        assert_eq!(input.buffer(), "横切码");
    }

    #[test]
    fn submit() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        assert_eq!(input.buffer(), "1");

        input.update(SubmitNode);
        assert_eq!(input.buffer(), "");
    }

    #[test]
    fn submit_at_cursor() {
        let mut input = EditorInputBuffer::empty();

        input.update(Insert { ch: '1' });
        input.update(Insert { ch: '2' });
        assert_eq!(input.buffer(), "12");

        input.update(MoveCursorLeft);
        input.update(SubmitNode);
        assert_eq!(input.buffer(), "2");
    }
}
