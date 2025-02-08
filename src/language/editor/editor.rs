use crate::language::{
    code::{Codebase, Node, NodePath},
    compiler::compile_and_replace,
    packages::Package,
    runtime::Evaluator,
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
        evaluator: &mut Evaluator,
        package: &Package,
    ) {
        if let Some(action) = self.input.update(event) {
            // This code results in non-intuitive cursor movement, if using the
            // up and down keys. This is tracked here:
            // https://github.com/hannobraun/crosscut/issues/71
            match action {
                UpdateAction::NavigateToPrevious => {
                    if let Some(location) = codebase.child_of(&self.editing) {
                        self.navigate_to(location, codebase, package);
                        self.input.move_cursor_to_end();
                    }
                }
                UpdateAction::NavigateToNextNode => {
                    if let Some(location) = codebase.parent_of(&self.editing) {
                        self.navigate_to(location, codebase, package);
                    }
                }
                UpdateAction::RemovePrevious => {
                    if let Some(to_remove) = codebase.child_of(&self.editing) {
                        let merged = [&to_remove, &self.editing]
                            .map(|path| {
                                codebase
                                    .node_at(path)
                                    .display(package)
                                    .to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        codebase.remove_node(&to_remove);
                        self.editing = codebase.latest_version_of(self.editing);
                    }
                }
                UpdateAction::RemoveNext => {
                    if let Some(to_remove) = codebase.parent_of(&self.editing) {
                        let merged = [&self.editing, &to_remove]
                            .map(|path| {
                                codebase
                                    .node_at(path)
                                    .display(package)
                                    .to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        codebase.remove_node(&to_remove);
                    }
                }
                UpdateAction::Submit { submitted } => {
                    compile_and_replace(
                        &submitted,
                        &mut self.editing,
                        package,
                        codebase,
                    );

                    let child = Some(*self.editing.hash());
                    self.editing = codebase
                        .insert_as_parent_of(self.editing, Node::empty(child));
                }
            }
        }

        compile_and_replace(
            self.input.buffer(),
            &mut self.editing,
            package,
            codebase,
        );

        // Unconditionally resetting the interpreter like this, is not going to
        // work long-term. It should only be reset, if it's finished.
        //
        // Right now, it doesn't seem to be practical to construct a high-level
        // test where this makes a difference though, and I don't want to fix
        // this until the behavior is covered by such a test.
        *evaluator = Evaluator::new(codebase);
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
                *evaluator = Evaluator::new(codebase);
            }
        }
    }

    fn navigate_to(
        &mut self,
        path: NodePath,
        codebase: &Codebase,
        package: &Package,
    ) {
        self.editing = path;

        let node = codebase.node_at(&path);
        self.input = EditorInputBuffer::new(node.display(package).to_string());
    }
}

pub enum EditorCommand {
    Clear,
}
