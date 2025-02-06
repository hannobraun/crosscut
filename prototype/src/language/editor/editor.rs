use crate::language::{
    code::{Codebase, Node, NodePath},
    compiler::compile,
    host::Host,
    runtime::Interpreter,
};

use super::{input_buffer::UpdateAction, EditorInputBuffer, EditorInputEvent};

#[derive(Debug)]
pub struct Editor {
    input: EditorInputBuffer,
    editing: NodePath,
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

    pub fn editing(&self) -> &NodePath {
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
            // This code places the cursor at the start, if moving to the next
            // syntax node, or at the end, if moving to the previous one.
            // Regardless of whether the movement was caused via up/down or via
            // left/right.
            //
            // This doesn't feel right when moving up or down, as it seems like
            // the cursor should stay where it already is, horizontally. And
            // fixing that here wouldn't actually be too hard.
            //
            // However, as I'm writing this, all syntax nodes are at the same
            // level of indentation. (Or rather, there is no indentation.) That
            // is going to change, and then it's no longer clear what an
            // implementation should look like. Editor output is handled
            // elsewhere.
            //
            // Maybe this means, that editor output (at least not all of it)
            // actually shouldn't be separate from the input logic here. Maybe
            // this code shouldn't operate on the code database directly, but
            // rather on some intermediate layer, that takes things like levels
            // of indentation into account.
            //
            // Whatever it'll end up being, I think it's better to leave this as
            // it is, for now, until we do have formatting with levels of
            // indentation. Then, it'll be easier to judge what works and what
            // doesn't.
            match action {
                UpdateAction::NavigateToPrevious => {
                    if let Some(location) = codebase.child_of(&self.editing) {
                        self.navigate_to(location, codebase, host);
                        self.input.move_cursor_to_end();
                    }
                }
                UpdateAction::NavigateToNextNode => {
                    if let Some(location) = codebase.parent_of(&self.editing) {
                        self.navigate_to(location, codebase, host);
                    }
                }
                UpdateAction::RemoveToPrevious => {
                    if let Some(to_remove) = codebase.child_of(&self.editing) {
                        let merged = [&to_remove, &self.editing]
                            .map(|path| {
                                codebase.node_at(path).display(host).to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        codebase.remove_node(&to_remove);
                        self.editing = codebase.latest_version_of(self.editing);
                    }
                }
                UpdateAction::RemoveNext => {
                    // Removing the next syntax node is not supported yet.
                }
                UpdateAction::Submit { submitted } => {
                    compile(&submitted, &mut self.editing, host, codebase);

                    let child = Some(*self.editing.hash());
                    self.editing = codebase
                        .insert_as_parent_of(self.editing, Node::empty(child));
                }
            }
        }

        compile(self.input.buffer(), &mut self.editing, host, codebase);

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
        path: NodePath,
        codebase: &Codebase,
        host: &Host,
    ) {
        self.editing = path;

        let node = codebase.node_at(&path);
        self.input = EditorInputBuffer::new(node.display(host).to_string());
    }
}

pub enum EditorCommand {
    Clear,
}
