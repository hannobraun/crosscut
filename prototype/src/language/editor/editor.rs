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

        codebase.value = if let Ok(value) = self.input.buffer().parse() {
            Value::Integer { value }
        } else {
            Value::None
        };
    }

    pub fn on_command(
        &mut self,
        command: EditorCommand,
        codebase: &mut Codebase,
    ) {
        match command {
            EditorCommand::Clear => {
                *self = Self::new();
                *codebase = Codebase::new();
            }
        }
    }
}

pub enum EditorCommand {
    Clear,
}
