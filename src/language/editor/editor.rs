use crate::language::{
    code::{Codebase, Node, NodePath},
    compiler::Compiler,
    packages::Packages,
    runtime::Evaluator,
};

use super::{
    EditorInputBuffer, EditorInputEvent, EditorLayout, input_buffer::NodeAction,
};

#[derive(Debug)]
pub struct Editor {
    input: EditorInputBuffer,
    editing: NodePath,
}

impl Editor {
    #[cfg(test)]
    pub fn new(
        editing: NodePath,
        codebase: &Codebase,
        packages: &Packages,
    ) -> Self {
        Self::postfix(editing, codebase, packages)
    }

    pub fn postfix(
        editing: NodePath,
        codebase: &Codebase,
        packages: &Packages,
    ) -> Self {
        let mut editor = Self {
            // This is just a placeholder value, to be updated below.
            input: EditorInputBuffer::empty(),
            editing: editing.clone(),
        };

        editor.navigate_to(editing, codebase, packages);

        editor
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
        packages: &Packages,
    ) {
        let layout = EditorLayout::postfix(codebase.root(), codebase.nodes());
        let mut compiler = Compiler::new(codebase);

        if let Some(action) = self.input.update(event) {
            // This code results in non-intuitive cursor movement, if using the
            // up and down keys. This is tracked here:
            // https://github.com/hannobraun/crosscut/issues/71
            match action {
                NodeAction::NavigateToPrevious => {
                    if let Some(previous) = layout.node_before(&self.editing) {
                        self.navigate_to(
                            previous.clone(),
                            compiler.codebase(),
                            packages,
                        );
                        self.input.move_cursor_to_end();
                    }
                }
                NodeAction::NavigateToNext => {
                    if let Some(next) = layout.node_after(&self.editing) {
                        self.navigate_to(
                            next.clone(),
                            compiler.codebase(),
                            packages,
                        );
                    }
                }
                NodeAction::MergeWithPrevious => {
                    if let Some(to_remove) = layout.node_before(&self.editing) {
                        let merged = [to_remove, &self.editing]
                            .map(|path| {
                                compiler
                                    .codebase()
                                    .node_at(path)
                                    .node
                                    .display(packages)
                                    .to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        compiler.remove(to_remove, &mut self.editing, packages);
                    }
                }
                NodeAction::MergeWithNext => {
                    if let Some(to_remove) = layout.node_after(&self.editing) {
                        let merged = [&self.editing, to_remove]
                            .map(|path| {
                                compiler
                                    .codebase()
                                    .node_at(path)
                                    .node
                                    .display(packages)
                                    .to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        compiler.remove(to_remove, &mut self.editing, packages);
                    }
                }
                NodeAction::AddParent { existing_child } => {
                    self.editing = compiler.replace(
                        &self.editing,
                        &existing_child,
                        packages,
                    );

                    let empty_parent =
                        self.editing.parent().and_then(|parent| {
                            let parent = compiler.codebase().node_at(parent);

                            let parent_is_empty =
                                matches!(parent.node, Node::Empty);
                            let parent_is_error_but_empty =
                                if let Node::Error { node, .. } = parent.node {
                                    node.is_empty()
                                } else {
                                    false
                                };

                            (parent_is_empty || parent_is_error_but_empty)
                                .then_some(parent)
                        });

                    self.editing = if let Some(parent) = empty_parent {
                        // If the parent node is empty, we re-use it instead of
                        // adding a new parent in between. This leads to a
                        // smoother editing experience.

                        compiler.replace(
                            &parent.path,
                            self.input.buffer(),
                            packages,
                        )
                    } else {
                        compiler.insert_parent(
                            &self.editing,
                            self.input.buffer(),
                            packages,
                        )
                    };
                }
                NodeAction::AddSibling { existing_sibling } => {
                    self.editing = compiler.replace(
                        &self.editing,
                        &existing_sibling,
                        packages,
                    );

                    self.editing = compiler.insert_sibling(
                        &self.editing,
                        self.input.buffer(),
                        packages,
                    );
                }
            }
        }

        self.editing =
            compiler.replace(&self.editing, self.input.buffer(), packages);

        let root = compiler.codebase().root().path;
        assert!(
            self.editing == root || root.is_ancestor_of(self.editing()),
            "Editor is no longer editing a current node after update.",
        );

        // Unconditionally resetting the interpreter like this, is not going to
        // work long-term. What we actually want to do here, is hot-reload the
        // changed code.
        //
        // For now, it doesn't seem like the difference is actually observable
        // though, due to the limited nature of the language.
        evaluator.reset(compiler.codebase());
    }

    pub fn on_command(
        &mut self,
        command: EditorCommand,
        codebase: &mut Codebase,
        evaluator: &mut Evaluator,
        packages: &Packages,
    ) {
        match command {
            EditorCommand::Clear => {
                *codebase = Codebase::new();
                *self = Self::postfix(codebase.root().path, codebase, packages);
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
        let node = codebase.node_at(&path).node;
        self.input = EditorInputBuffer::new(node.to_token(packages));

        self.editing = path;
    }
}

#[cfg(test)]
impl Editor {
    pub fn on_code(
        &mut self,
        code: &str,
        codebase: &mut Codebase,
        evaluator: &mut Evaluator,
        packages: &Packages,
    ) {
        for ch in code.chars() {
            let event = if ch == ' ' {
                EditorInputEvent::AddChildOrParent
            } else if ch == '\n' {
                EditorInputEvent::AddSibling
            } else {
                EditorInputEvent::Insert { ch }
            };

            self.on_input(event, codebase, evaluator, packages);
        }
    }
}

pub enum EditorCommand {
    Clear,
}
