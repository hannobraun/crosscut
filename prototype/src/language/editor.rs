use super::code::Codebase;

#[derive(Debug)]
pub struct Editor {
    input: EditorInput,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            input: EditorInput::new(String::new()),
        }
    }

    pub fn on_input(
        &mut self,
        event: EditorInputEvent,
        codebase: &mut Codebase,
    ) {
        match event {
            EditorInputEvent::Character { ch } => {
                self.input.insert(ch);
            }
            EditorInputEvent::MoveCursorLeft => {
                self.input.move_cursor_left();
            }
            EditorInputEvent::MoveCursorRight => {
                self.input.move_cursor_right();
            }
            EditorInputEvent::RemoveCharacterLeft => {
                self.input.remove_left();
            }
        }

        if let Ok(value) = self.input.buffer.parse() {
            codebase.value = Some(value);
        }
    }

    pub fn on_command(&mut self, command: EditorCommand, _: &mut Codebase) {
        match command {
            EditorCommand::Clear => {
                *self = Self::new();
            }
        }
    }
}

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
    Character { ch: char },
    MoveCursorLeft,
    MoveCursorRight,
    RemoveCharacterLeft,
}

pub enum EditorCommand {
    Clear,
}
