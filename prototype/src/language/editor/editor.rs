use crate::language::code::Codebase;

use super::{EditorInput, EditorInputEvent};

#[derive(Debug)]
pub struct Editor {
    input: EditorInput,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            input: EditorInput::empty(),
        }
    }

    pub fn on_input(
        &mut self,
        event: EditorInputEvent,
        codebase: &mut Codebase,
    ) {
        match event {
            EditorInputEvent::Insert { ch } => {
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

        if let Ok(value) = self.input.buffer().parse() {
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

pub enum EditorCommand {
    Clear,
}
