use crate::language::{
    code::{Codebase, Location, Node},
    compiler::compile,
    host::Host,
    runtime::Interpreter,
};

use super::{input_buffer::UpdateAction, EditorInputBuffer, EditorInputEvent};

#[derive(Debug)]
pub struct Editor {
    input: EditorInputBuffer,
    editing: Location,
}

impl Editor {
    pub fn new(codebase: &mut Codebase) -> Self {
        let editing = codebase.push_node(Node::Empty);

        Self {
            input: EditorInputBuffer::empty(),
            editing,
        }
    }

    pub fn input(&self) -> &EditorInputBuffer {
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
                UpdateAction::SubmitToken { submitted } => {
                    let node =
                        compile(&submitted, &self.editing, host, codebase);
                    codebase.replace_node(&self.editing, node);

                    self.editing = codebase.push_node(Node::Empty);
                }
            }
        }

        let node = compile(self.input.buffer(), &self.editing, host, codebase);
        codebase.replace_node(&self.editing, node);

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
