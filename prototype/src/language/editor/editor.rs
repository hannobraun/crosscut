use crate::language::{code::Codebase, interpreter::Value};

use super::{input::UpdateAction, EditorInput, EditorInputEvent};

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
        if let Some(action) = self.input.update(event) {
            match action {
                UpdateAction::SubmitToken => {
                    // We need to create a new token here to edit that, but that
                    // is not supported yet.
                }
            }
        }

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
