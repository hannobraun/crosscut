use crate::language::{
    code::{Codebase, NodePath, SyntaxNode},
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
}

impl Editor {
    pub fn new(
        cursor: impl Into<Cursor>,
        codebase: &Codebase,
        _: &Packages,
    ) -> Self {
        let cursor = cursor.into();

        let mut editor = Self {
            input: EditorInputBuffer::empty(),
            cursor: cursor.clone(),
        };

        editor.navigate_to(cursor, codebase);

        editor
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn on_input(
        &mut self,
        event: EditorInputEvent,
        codebase: &mut Codebase,
        evaluator: &mut Evaluator,
        packages: &Packages,
    ) {
        let layout = EditorLayout::new(codebase.root(), codebase);
        let mut compiler = Compiler::new(codebase);

        if let Some(action) = self.input.update(event, &mut self.cursor.index) {
            // This code results in non-intuitive cursor movement, if using the
            // up and down keys. This is tracked here:
            // https://github.com/hannobraun/crosscut/issues/71
            match action {
                NodeAction::NavigateToPrevious => {
                    if let Some(previous) =
                        layout.node_before(&self.cursor.path)
                    {
                        self.navigate_to(previous.clone(), compiler.codebase());
                        self.input.move_cursor_to_end(&mut self.cursor.index);
                    }
                }
                NodeAction::NavigateToNext => {
                    if let Some(next) = layout.node_after(&self.cursor.path) {
                        self.navigate_to(next.clone(), compiler.codebase());
                    }
                }
            }
        }

        let current_node = compiler.codebase().node_at(&self.cursor.path);
        self.cursor.path = if let SyntaxNode::AddValue = current_node.node {
            let Some(parent) = current_node.path.parent().cloned() else {
                unreachable!(
                    "Current node is a node that is solely dedicated to adding \
                    children to its parent. Thus, it must have a parent."
                );
            };

            compiler.insert_child(parent, self.input.buffer(), packages)
        } else {
            compiler.replace(&self.cursor.path, self.input.buffer(), packages)
        };

        let root = compiler.codebase().root().path;
        assert!(
            self.cursor.path == root || root.is_ancestor_of(&self.cursor.path),
            "Editor is no longer editing a current node after update.",
        );

        // Unconditionally resetting the interpreter like this, is not going to
        // work long-term. What we actually want to do here, is hot-reload the
        // changed code.
        //
        // For now, it seems like the difference is actually not observable
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
                *self = Self::new(codebase.root().path, codebase, packages);
                evaluator.reset(codebase);
            }
        }
    }

    fn navigate_to(&mut self, cursor: impl Into<Cursor>, codebase: &Codebase) {
        let cursor = cursor.into();

        let node = codebase.node_at(&cursor.path).node;
        self.input =
            EditorInputBuffer::new(node.to_token(), &mut self.cursor.index);

        self.cursor = cursor;
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
            let event = if ch.is_whitespace() {
                EditorInputEvent::MoveCursorDown
            } else {
                EditorInputEvent::Insert { ch }
            };

            self.on_input(event, codebase, evaluator, packages);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cursor {
    pub path: NodePath,
    pub index: usize,
}

impl From<NodePath> for Cursor {
    fn from(path: NodePath) -> Self {
        Self { path, index: 0 }
    }
}

pub enum EditorCommand {
    Clear,
}
