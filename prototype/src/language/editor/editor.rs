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
    pub fn new(codebase: &Codebase) -> Self {
        Self {
            input: EditorInputBuffer::empty(),
            editing: codebase.entry(),
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
                UpdateAction::NavigateToPrevious => {
                    if let Some(location) =
                        codebase.location_before(&self.editing)
                    {
                        self.navigate_to(location, codebase, host);
                    }
                }
                UpdateAction::NavigateToNextNode => {
                    if let Some(location) =
                        codebase.location_after(&self.editing)
                    {
                        self.navigate_to(location, codebase, host);
                    }
                }
                UpdateAction::Submit { submitted } => {
                    compile(&submitted, self.editing, host, codebase);
                    self.editing =
                        codebase.insert_node_after(self.editing, Node::Empty);
                }
            }
        }

        compile(self.input.buffer(), self.editing, host, codebase);

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

    fn navigate_to(
        &mut self,
        location: Location,
        codebase: &Codebase,
        host: &Host,
    ) {
        self.editing = location;

        let node = codebase.node_at(&location);
        self.input = EditorInputBuffer::new(node.display(host).to_string());
    }
}

pub enum EditorCommand {
    Clear,
}
