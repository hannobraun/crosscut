use crate::language::{
    code::{Codebase, NodeKind, NodePath},
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
    pub fn new(
        editing: NodePath,
        codebase: &Codebase,
        packages: &Packages,
    ) -> Self {
        let mut editor = Self {
            // This is just a placeholder value, to be updated below.
            input: EditorInputBuffer::empty(),
            editing,
        };

        editor.navigate_to(editing, codebase, packages);
        editor.input.move_cursor_to_end();

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
        let layout = EditorLayout::new(codebase.root(), codebase.nodes());
        let compiler = &mut Compiler::new(codebase);

        if let Some(action) = self.input.update(event) {
            // This code results in non-intuitive cursor movement, if using the
            // up and down keys. This is tracked here:
            // https://github.com/hannobraun/crosscut/issues/71
            match action {
                NodeAction::NavigateToPrevious => {
                    if let Some(previous) = layout.node_before(&self.editing) {
                        self.navigate_to(
                            previous,
                            compiler.codebase(),
                            packages,
                        );
                        self.input.move_cursor_to_end();
                    }
                }
                NodeAction::NavigateToNext => {
                    if let Some(next) = layout.node_after(&self.editing) {
                        self.navigate_to(next, compiler.codebase(), packages);
                    }
                }
                NodeAction::MergeWithPrevious => {
                    if let Some(to_remove) = layout.node_before(&self.editing) {
                        let merged = [&to_remove, &self.editing]
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

                        compiler.remove(to_remove, packages);
                        self.editing = compiler
                            .codebase()
                            .latest_version_of(&self.editing);
                    }
                }
                NodeAction::MergeWithNext => {
                    if let Some(to_remove) = layout.node_after(&self.editing) {
                        let merged = [&self.editing, &to_remove]
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

                        compiler.remove(to_remove, packages);
                    }
                }
                NodeAction::AddParent { existing_child } => {
                    self.editing = compiler.replace(
                        &self.editing,
                        &existing_child,
                        packages,
                    );

                    let empty_parent = compiler
                        .codebase()
                        .parent_of(&self.editing)
                        .filter(|parent| {
                            let parent_is_empty =
                                matches!(parent.node.kind(), NodeKind::Empty);
                            let parent_is_error_but_empty =
                                if let NodeKind::Error { node } =
                                    parent.node.kind()
                                {
                                    node.is_empty()
                                } else {
                                    false
                                };

                            parent_is_empty || parent_is_error_but_empty
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

        // Unconditionally resetting the interpreter like this, is not going to
        // work long-term. It should only be reset, if it's finished.
        //
        // Right now, it doesn't seem to be practical to construct a high-level
        // test where this makes a difference though, and I don't want to fix
        // this until the behavior is covered by such a test.
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
                *self = Self::new(codebase.root().path, codebase, packages);
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

        let node = codebase.node_at(&path).node;
        self.input = EditorInputBuffer::new(node.to_token(packages));
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
                EditorInputEvent::AddParent
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
