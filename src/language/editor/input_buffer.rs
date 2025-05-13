use std::cmp::min;

use super::EditorInputEvent;

#[derive(Debug, Eq, PartialEq)]
pub struct EditorInputBuffer {
    buffer: String,
}

impl EditorInputBuffer {
    pub fn new(buffer: String, cursor: &mut usize) -> Self {
        *cursor = 0;
        Self { buffer }
    }

    pub fn move_cursor_to_end(&mut self, cursor: &mut usize) {
        // The cursor counts bytes, not characters. So the use of `len` here is
        // correct.
        *cursor = self.buffer.len();
    }

    pub fn empty() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn buffer(&self) -> &String {
        &self.buffer
    }

    pub fn update(
        &mut self,
        event: EditorInputEvent,
        cursor: &mut usize,
    ) -> Option<NodeAction> {
        match event {
            EditorInputEvent::Insert { ch } => {
                assert!(
                    !ch.is_whitespace(),
                    "Expecting whitespace characters to be translated into \
                    other editor input events.",
                );

                self.insert(ch, cursor);
            }
            EditorInputEvent::MoveCursorLeft => {
                return self.move_cursor_left(cursor);
            }
            EditorInputEvent::MoveCursorRight => {
                return self.move_cursor_right(cursor);
            }
            EditorInputEvent::MoveCursorUp => {
                return Some(NodeAction::NavigateToPrevious);
            }
            EditorInputEvent::MoveCursorDown => {
                return Some(NodeAction::NavigateToNext);
            }
            EditorInputEvent::RemoveLeft { whole_node } => {
                if whole_node {
                    self.remove_left_whole_node(cursor);
                } else {
                    self.remove_left(cursor);
                }
            }
            EditorInputEvent::RemoveRight { whole_node } => {
                let _ = whole_node;
                self.remove_right(cursor);
            }
        }

        None
    }

    fn insert(&mut self, ch: char, cursor: &mut usize) {
        self.buffer.insert(*cursor, ch);
        self.move_cursor_right(cursor);
    }

    fn move_cursor_left(&mut self, cursor: &mut usize) -> Option<NodeAction> {
        loop {
            if *cursor > 0 {
                *cursor -= 1;
            } else {
                return Some(NodeAction::NavigateToPrevious);
            }

            if self.buffer.is_char_boundary(*cursor) {
                break;
            }
        }

        None
    }

    fn move_cursor_right(&mut self, cursor: &mut usize) -> Option<NodeAction> {
        loop {
            *cursor = cursor.saturating_add(1);
            if *cursor > self.buffer.len() {
                *cursor = self.buffer.len();
                return Some(NodeAction::NavigateToNext);
            }
            *cursor = min(*cursor, self.buffer.len());

            if self.buffer.is_char_boundary(*cursor) {
                break;
            }

            assert!(
                *cursor < self.buffer.len(),
                "Moved cursor right, and not at char boundary. This means \
                cursor must still be in bounds, and we're not risking an \
                endless loop.",
            );
        }

        None
    }

    fn remove_left(&mut self, cursor: &mut usize) {
        if self.move_cursor_left(cursor).is_none() {
            self.buffer.remove(*cursor);
        }
    }

    fn remove_left_whole_node(&mut self, cursor: &mut usize) {
        while self.move_cursor_left(cursor).is_none() {
            self.buffer.remove(*cursor);
        }
    }

    fn remove_right(&mut self, cursor: &mut usize) {
        if *cursor < self.buffer.len() {
            self.buffer.remove(*cursor);
        }
    }
}

#[derive(Debug)]
pub enum NodeAction {
    NavigateToPrevious,
    NavigateToNext,
}

#[cfg(test)]
mod tests {
    use super::{EditorInputBuffer, EditorInputEvent::*};

    #[test]
    fn insert() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '1' }, &mut cursor);
        assert_eq!(input.buffer(), "1");

        input.update(Insert { ch: '2' }, &mut cursor);
        assert_eq!(input.buffer(), "12");
    }

    #[test]
    fn insert_at_cursor() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '2' }, &mut cursor);
        assert_eq!(input.buffer(), "2");

        input.update(MoveCursorLeft, &mut cursor);
        input.update(Insert { ch: '1' }, &mut cursor);
        assert_eq!(input.buffer(), "12");

        input.update(MoveCursorRight, &mut cursor);
        input.update(Insert { ch: '7' }, &mut cursor);
        assert_eq!(input.buffer(), "127");
    }

    #[test]
    fn remove_left() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '1' }, &mut cursor);
        input.update(Insert { ch: '2' }, &mut cursor);
        assert_eq!(input.buffer(), "12");

        input.update(RemoveLeft { whole_node: false }, &mut cursor);
        assert_eq!(input.buffer(), "1");

        input.update(RemoveLeft { whole_node: false }, &mut cursor);
        assert_eq!(input.buffer(), "");
    }

    #[test]
    fn remove_left_at_cursor() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '1' }, &mut cursor);
        input.update(Insert { ch: '2' }, &mut cursor);
        assert_eq!(input.buffer(), "12");

        input.update(MoveCursorLeft, &mut cursor);
        input.update(RemoveLeft { whole_node: false }, &mut cursor);
        assert_eq!(input.buffer(), "2");
    }

    #[test]
    fn remove_left_while_already_at_leftmost_position() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '1' }, &mut cursor);
        assert_eq!(input.buffer(), "1");

        input.update(MoveCursorLeft, &mut cursor);
        input.update(RemoveLeft { whole_node: false }, &mut cursor);
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn remove_left_whole_node() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '1' }, &mut cursor);
        input.update(Insert { ch: '2' }, &mut cursor);
        input.update(Insert { ch: '7' }, &mut cursor);
        input.update(MoveCursorLeft, &mut cursor);
        assert_eq!(input.buffer(), "127");

        input.update(RemoveLeft { whole_node: true }, &mut cursor);
        assert_eq!(input.buffer(), "7");
    }

    #[test]
    fn remove_right() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '1' }, &mut cursor);
        input.update(Insert { ch: '2' }, &mut cursor);
        assert_eq!(input.buffer(), "12");

        input.update(MoveCursorLeft, &mut cursor);
        input.update(MoveCursorLeft, &mut cursor);
        input.update(RemoveRight { whole_node: false }, &mut cursor);
        assert_eq!(input.buffer(), "2");

        input.update(RemoveRight { whole_node: false }, &mut cursor);
        assert_eq!(input.buffer(), "");
    }

    #[test]
    fn remove_right_while_already_at_rightmost_position() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '1' }, &mut cursor);
        assert_eq!(input.buffer(), "1");

        input.update(RemoveRight { whole_node: false }, &mut cursor);
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_left_while_already_at_leftmost_position() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(MoveCursorLeft, &mut cursor);
        input.update(Insert { ch: '1' }, &mut cursor);
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_right_while_already_at_rightmost_position() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(MoveCursorRight, &mut cursor);
        input.update(Insert { ch: '1' }, &mut cursor);
        assert_eq!(input.buffer(), "1");
    }

    #[test]
    fn move_cursor_over_non_ascii_characters() {
        let mut input = EditorInputBuffer::empty();
        let mut cursor = 0;

        input.update(Insert { ch: '横' }, &mut cursor);
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
        input.update(Insert { ch: '码' }, &mut cursor);
        assert_eq!(input.buffer(), "横码");

        input.update(MoveCursorLeft, &mut cursor);
        input.update(Insert { ch: '切' }, &mut cursor);
        assert_eq!(input.buffer(), "横切码");
    }
}
