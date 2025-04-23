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
    cursor: Cursor,
    postfix: bool,
}

impl Editor {
    pub fn new(
        editing: NodePath,
        codebase: &Codebase,
        packages: &Packages,
    ) -> Self {
        let mut editor = Self {
            input: EditorInputBuffer::empty(),
            cursor: Cursor {
                path: editing.clone(),
            },
            postfix: false,
        };

        editor.navigate_to(editing, codebase, packages);

        editor
    }

    pub fn postfix(
        editing: NodePath,
        codebase: &Codebase,
        packages: &Packages,
    ) -> Self {
        let mut editor = Self::new(editing, codebase, packages);
        editor.postfix = true;

        editor
    }

    pub fn input(&self) -> &EditorInputBuffer {
        &self.input
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn on_input(
        &mut self,
        events: impl IntoIterator<Item = EditorInputEvent>,
        codebase: &mut Codebase,
        evaluator: &mut Evaluator,
        packages: &Packages,
    ) {
        let layout = if self.postfix {
            EditorLayout::postfix(codebase.root(), codebase.nodes())
        } else {
            EditorLayout::new(codebase.root(), codebase.nodes())
        };
        let mut compiler = Compiler::new(codebase);

        for event in events {
            if let Some(action) = self.input.update(event) {
                // This code results in non-intuitive cursor movement, if using
                // the up and down keys. This is tracked here:
                // https://github.com/hannobraun/crosscut/issues/71
                match action {
                    NodeAction::NavigateToPrevious => {
                        if let Some(previous) =
                            layout.node_before(&self.cursor.path)
                        {
                            self.navigate_to(
                                previous.clone(),
                                compiler.codebase(),
                                packages,
                            );
                            self.input.move_cursor_to_end();
                        }
                    }
                    NodeAction::NavigateToNext => {
                        if let Some(next) = layout.node_after(&self.cursor.path)
                        {
                            self.navigate_to(
                                next.clone(),
                                compiler.codebase(),
                                packages,
                            );
                        }
                    }
                    NodeAction::MergeWithPrevious => {
                        if let Some(to_remove) =
                            layout.node_before(&self.cursor.path)
                        {
                            let merged = [to_remove, &self.cursor.path]
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

                            compiler.remove(
                                to_remove,
                                &mut self.cursor.path,
                                packages,
                            );
                        }
                    }
                    NodeAction::MergeWithNext => {
                        if let Some(to_remove) =
                            layout.node_after(&self.cursor.path)
                        {
                            let merged = [&self.cursor.path, to_remove]
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

                            compiler.remove(
                                to_remove,
                                &mut self.cursor.path,
                                packages,
                            );
                        }
                    }
                    NodeAction::AddChildOrParent {
                        existing_child_or_parent,
                    } => {
                        self.cursor.path = compiler.replace(
                            &self.cursor.path,
                            &existing_child_or_parent,
                            packages,
                        );

                        self.cursor.path = if self.postfix {
                            let empty_parent =
                                self.cursor.path.parent().and_then(|parent| {
                                    let parent =
                                        compiler.codebase().node_at(parent);

                                    let parent_is_empty =
                                        matches!(parent.node, Node::Empty);
                                    let parent_is_error_but_empty =
                                        if let Node::Error { node, .. } =
                                            parent.node
                                        {
                                            node.is_empty()
                                        } else {
                                            false
                                        };

                                    (parent_is_empty
                                        || parent_is_error_but_empty)
                                        .then_some(parent)
                                });

                            if let Some(parent) = empty_parent {
                                // If the parent node is empty, we re-use it
                                // instead of adding a new parent in between.
                                // This leads to a smoother editing experience.

                                compiler.replace(
                                    &parent.path,
                                    self.input.buffer(),
                                    packages,
                                )
                            } else {
                                compiler.insert_parent(
                                    &self.cursor.path,
                                    self.input.buffer(),
                                    packages,
                                )
                            }
                        } else {
                            compiler.insert_child(
                                self.cursor.path.clone(),
                                self.input.buffer(),
                                packages,
                            )
                        };
                    }
                    NodeAction::AddSibling { existing_sibling } => {
                        self.cursor.path = compiler.replace(
                            &self.cursor.path,
                            &existing_sibling,
                            packages,
                        );

                        self.cursor.path = compiler.insert_sibling(
                            &self.cursor.path,
                            self.input.buffer(),
                            packages,
                        );
                    }
                }
            }

            self.cursor.path = compiler.replace(
                &self.cursor.path,
                self.input.buffer(),
                packages,
            );

            let root = compiler.codebase().root().path;
            assert!(
                self.cursor.path == root
                    || root.is_ancestor_of(&self.cursor.path),
                "Editor is no longer editing a current node after update.",
            );
        }

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

        self.cursor = Cursor { path };
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

            self.on_input([event], codebase, evaluator, packages);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cursor {
    pub path: NodePath,
}

pub enum EditorCommand {
    Clear,
}
