use crate::language::{code::Codebase, interpreter::Value};

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

    pub fn input(&self) -> &EditorInput {
        &self.input
    }

    pub fn on_input(
        &mut self,
        event: EditorInputEvent,
        codebase: &mut Codebase,
    ) {
        self.input.update(event);

        if let Ok(value) = self.input.buffer().parse() {
            codebase.value = Value::Integer { value };
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
