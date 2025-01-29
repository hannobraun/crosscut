use crate::language::{
    code::{Codebase, Expression, IntrinsicFunction, Location, Node},
    interpreter::{Interpreter, Value},
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
        interpreter: &mut Interpreter,
    ) {
        if let Some(action) = self.input.update(event) {
            match action {
                UpdateAction::SubmitToken => {
                    self.editing = codebase.push(Node::Empty);
                }
            }
        }

        let value = if let Ok(value) = self.input.buffer().parse() {
            Expression::IntrinsicFunction {
                function: IntrinsicFunction::Literal {
                    value: Value::Integer { value },
                },
            }
        } else {
            Expression::IntrinsicFunction {
                function: IntrinsicFunction::Literal { value: Value::None },
            }
        };

        codebase.replace(&self.editing, Node::Expression { expression: value });

        // Unconditionally resetting the interpreter like this, is not going to
        // work long-term. It should only be reset, if it's finished.
        //
        // Right now, this makes no difference though, as the interpreter's
        // implementation is too primitive to distinguish between being finished
        // or not.
        *interpreter = Interpreter::new(codebase);
    }

    pub fn on_command(
        &mut self,
        command: EditorCommand,
        codebase: &mut Codebase,
        interpreter: &mut Interpreter,
    ) {
        match command {
            EditorCommand::Clear => {
                *codebase = Codebase::new();
                *interpreter = Interpreter::new(codebase);
                *self = Self::new(codebase);
            }
        }
    }
}

pub enum EditorCommand {
    Clear,
}
