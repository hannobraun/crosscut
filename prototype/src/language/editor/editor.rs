use crate::language::{
    code::{Codebase, Expression, Location, Node},
    interpreter::Value,
};

use super::{input::UpdateAction, EditorInput, EditorInputEvent};

#[derive(Debug)]
pub struct Editor {
    input: EditorInput,
    editing: Location,
}

impl Editor {
    pub fn new(codebase: &mut Codebase) -> Self {
        let editing = codebase.push(Node::Empty);

        Self {
            input: EditorInput::empty(),
            editing,
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

        codebase.replace(&self.editing, Node::Expression { expression: value });
    }

    pub fn on_command(
        &mut self,
        command: EditorCommand,
        codebase: &mut Codebase,
    ) {
        match command {
            EditorCommand::Clear => {
                *codebase = Codebase::new();
                *self = Self::new(codebase);
            }
        }
    }
}

pub enum EditorCommand {
    Clear,
}
