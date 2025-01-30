use crate::language::{
    code::{Codebase, Expression, IntrinsicFunction, Location, Node},
    host::Host,
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

    pub fn editing(&self) -> &Location {
        &self.editing
    }

    pub fn on_input(
        &mut self,
        event: EditorInputEvent,
        codebase: &mut Codebase,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        if let Some(action) = self.input.update(event) {
            match action {
                UpdateAction::SubmitToken => {
                    self.editing = codebase.push(Node::Empty);
                }
            }
        }

        let node = if self.input.buffer().is_empty() {
            Node::Empty
        } else if let Ok(value) = self.input.buffer().parse() {
            Node::Expression {
                expression: Expression::IntrinsicFunction {
                    function: IntrinsicFunction::Literal {
                        value: Value::Integer { value },
                    },
                },
            }
        } else if let Some(id) = host.function_id_by_name(self.input.buffer()) {
            Node::Expression {
                expression: Expression::HostFunction { id },
            }
        } else if self.input.buffer() == "identity" {
            Node::Expression {
                expression: Expression::IntrinsicFunction {
                    function: IntrinsicFunction::Identity,
                },
            }
        } else {
            Node::UnresolvedIdentifier {
                name: self.input.buffer().clone(),
            }
        };

        codebase.replace(&self.editing, node);

        // Unconditionally resetting the interpreter like this, is not going to
        // work long-term. It should only be reset, if it's finished.
        //
        // Right now, it doesn't seem to be practical to construct a high-level
        // test where this makes a difference though, and I don't want to fix
        // this until the behavior is covered by such a test.
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
                *self = Self::new(codebase);
                *interpreter = Interpreter::new(codebase);
            }
        }
    }
}

pub enum EditorCommand {
    Clear,
}
