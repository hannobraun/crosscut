use crate::language::{
    code::{Codebase, Expression},
    interpreter::Value,
};

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

        let value = if let Ok(value) = self.input.buffer().parse() {
            Expression::LiteralValue {
                value: Value::Integer { value },
            }
        } else {
            Expression::LiteralValue { value: Value::None }
        };

        codebase.expressions = vec![value];
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
