use crate::language::{
    code::{Codebase, Node, NodeKind, NodePath},
    compiler::Compiler,
    packages::Packages,
    runtime::Evaluator,
};

use super::{EditorInputBuffer, EditorInputEvent, input_buffer::UpdateAction};

#[derive(Debug)]
pub struct Editor {
    input: EditorInputBuffer,
    editing: NodePath,
}

impl Editor {
    pub fn new(codebase: &Codebase) -> Self {
        Self {
            input: EditorInputBuffer::empty(),
            editing: codebase.root().path,
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
        compiler: &mut Compiler,
        evaluator: &mut Evaluator,
        packages: &Packages,
    ) {
        if let Some(action) = self.input.update(event) {
            // This code results in non-intuitive cursor movement, if using the
            // up and down keys. This is tracked here:
            // https://github.com/hannobraun/crosscut/issues/71
            match action {
                UpdateAction::NavigateToPrevious => {
                    if let Some(location) =
                        compiler.children_of(&self.editing).to_paths().last()
                    {
                        self.navigate_to(location, compiler, packages);
                        self.input.move_cursor_to_end();
                    }
                }
                UpdateAction::NavigateToNextNode => {
                    if let Some(location) = compiler.parent_of(&self.editing) {
                        self.navigate_to(location, compiler, packages);
                    }
                }
                UpdateAction::MergeWithPrevious => {
                    if let Some(to_remove) =
                        compiler.children_of(&self.editing).to_paths().last()
                    {
                        let merged = [&to_remove, &self.editing]
                            .map(|path| {
                                compiler
                                    .node_at(path)
                                    .display(packages)
                                    .to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        compiler.remove(&to_remove);
                        self.editing = compiler.latest_version_of(self.editing);
                    }
                }
                UpdateAction::MergeWithNext => {
                    if let Some(to_remove) = compiler.parent_of(&self.editing) {
                        let merged = [&self.editing, &to_remove]
                            .map(|path| {
                                compiler
                                    .node_at(path)
                                    .display(packages)
                                    .to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        compiler.remove(&to_remove);
                    }
                }
                UpdateAction::AddParent { previous } => {
                    self.editing =
                        compiler.replace(&self.editing, &previous, packages);

                    let child = Some(*self.editing.hash());
                    self.editing = compiler.insert_parent(
                        &self.editing,
                        // Depending on where the cursor was, the input buffer
                        // might already contain characters that should make up
                        // the new node. So the empty node we insert here is
                        // just a placeholder, which might get replaced by the
                        // unconditional compilation of the current input buffer
                        // contents below.
                        Node::new(NodeKind::Empty, child),
                    );
                }
            }
        }

        self.editing =
            compiler.replace(&self.editing, self.input.buffer(), packages);

        // Unconditionally resetting the interpreter like this, is not going to
        // work long-term. It should only be reset, if it's finished.
        //
        // Right now, it doesn't seem to be practical to construct a high-level
        // test where this makes a difference though, and I don't want to fix
        // this until the behavior is covered by such a test.
        evaluator.reset(compiler);
    }

    pub fn on_command(
        &mut self,
        command: EditorCommand,
        codebase: &mut Codebase,
        evaluator: &mut Evaluator,
    ) {
        match command {
            EditorCommand::Clear => {
                *codebase = Codebase::new();
                *self = Self::new(codebase);
                evaluator.reset(codebase);
            }
        }
    }

    fn navigate_to(
        &mut self,
        path: NodePath,
        codebase: &Codebase,
        packages: &Packages,
    ) {
        self.editing = path;

        let node = codebase.node_at(&path);
        self.input = EditorInputBuffer::new(node.display(packages).to_string());
    }
}

pub enum EditorCommand {
    Clear,
}
