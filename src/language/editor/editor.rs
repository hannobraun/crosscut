use crate::language::{
    code::{Codebase, NodePath},
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

        editor.navigate_to(editor.editing, codebase, packages);
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
                    if let Some(to_remove) = compiler
                        .codebase()
                        .children_of(&self.editing)
                        .to_paths()
                        .last()
                    {
                        let merged = [&to_remove, &self.editing]
                            .map(|path| {
                                compiler
                                    .codebase()
                                    .node_at(path)
                                    .display(packages)
                                    .to_string()
                            })
                            .join("");
                        self.input = EditorInputBuffer::new(merged);

                        compiler.remove(to_remove, packages);
                        self.editing =
                            compiler.codebase().latest_version_of(self.editing);
                    }
                }
                NodeAction::MergeWithNext => {
                    if let Some(to_remove) =
                        compiler.codebase().parent_of(&self.editing)
                    {
                        let merged = [&self.editing, &to_remove]
                            .map(|path| {
                                compiler
                                    .codebase()
                                    .node_at(path)
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

                    self.editing = compiler.insert_parent(
                        &self.editing,
                        self.input.buffer(),
                        packages,
                    );
                }
                NodeAction::AddSibling { previous } => {
                    self.editing =
                        compiler.replace(&self.editing, &previous, packages);

                    let parent = compiler
                        .codebase()
                        .parent_of(&self.editing)
                        .unwrap_or_else(|| {
                            // The node we're adding a sibling for has no
                            // parent, meaning it is the root of the syntax
                            // tree.
                            //
                            // The syntax tree always needs a single root. So we
                            // can't add a sibling to the root node, without a
                            // new root node that can serve as both of their
                            // parent.
                            //
                            // Adding this new root node is what we're doing
                            // here.
                            compiler.insert_parent(&self.editing, "", packages)
                        });

                    self.editing = compiler.insert_child(
                        parent,
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

        let node = codebase.node_at(&path);
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
