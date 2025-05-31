use std::{fmt, fs::File};

use crate::language::{
    code::{Codebase, NodePath, SyntaxNode},
    compiler::Compiler,
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
    pub fn new(cursor: impl Into<Cursor>, codebase: &Codebase) -> Self {
        let mut cursor = cursor.into();

        // The editor doesn't directly show the `Expressions` node, only its
        // children.
        let located_node = codebase.node_at(&cursor.path);
        if let SyntaxNode::Body { .. } = located_node.node {
            let Some(child) = located_node.children(codebase.nodes()).next()
            else {
                unreachable!(
                    "A body node has at least one child, the node for adding \
                    more children."
                );
            };

            cursor.path = child.path;
        }

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
                NodeAction::Submit => {
                    // This is preparation for letting the user submit new nodes
                    // (as a parent or sibling of the currently selected node)
                    // when this is appropriate in the current context. But this
                    // has turned out to be quite complicated.
                    //
                    // I don't think this is an insurmountable problem, but it's
                    // definitely something that requires some rounds of
                    // iteration and a dedicated test suite. I don't think this
                    // is worth the price right now. The current solution of
                    // having a dedicated "add new node" node in the syntax tree
                    // is weird, but it works.
                    //
                    // The code path that leads here is left in preparation for
                    // making this happen later on.

                    if let Some(next) = layout.node_after(&self.cursor.path) {
                        self.navigate_to(next.clone(), compiler.codebase());
                    }
                }
            }
        }

        let current_node = compiler.codebase().node_at(&self.cursor.path);
        if let SyntaxNode::Add = current_node.node {
            if !self.input.contents().is_empty() {
                let Some((parent, _)) = current_node.path.parent() else {
                    unreachable!(
                        "Current node is a node that is solely dedicated to \
                        adding children to its parent. Thus, it must have a \
                        parent."
                    );
                };

                self.cursor.path = compiler
                    .insert_child(parent.clone(), self.input.contents());
            }
        } else if &compiler
            .codebase()
            .nodes()
            .get(self.cursor.path.hash())
            .to_token()
            != self.input.contents()
        {
            self.cursor.path =
                compiler.replace(&self.cursor.path, self.input.contents());
        }

        let root = compiler.codebase().root().path;
        assert!(
            self.cursor.path == root || root.is_ancestor_of(&self.cursor.path),
            "Editor is no longer editing a current node after update.",
        );

        evaluator.update(compiler.codebase());
    }

    pub fn on_command(
        &mut self,
        command: EditorCommand,
        codebase: &mut Codebase,
        evaluator: &mut Evaluator,
    ) -> anyhow::Result<()> {
        match command {
            EditorCommand::Clear => {
                *codebase = Codebase::new();
                *self = Self::new(codebase.root().path, codebase);
                evaluator.reset(codebase);
            }
            EditorCommand::Dump => {
                for (name, data) in [
                    ("codebase", codebase as &dyn fmt::Debug),
                    ("evaluator", evaluator),
                ] {
                    use std::io::Write;

                    let name = format!("{name}.dump");
                    let mut file = File::create(name)?;

                    write!(file, "{data:#?}")?;
                }
            }
            EditorCommand::Reset => {
                evaluator.reset(codebase);
            }
        }

        Ok(())
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
    ) {
        for ch in code.chars() {
            let event = if ch.is_whitespace() {
                EditorInputEvent::Submit
            } else {
                EditorInputEvent::Insert { ch }
            };

            self.on_input(event, codebase, evaluator);
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
    Dump,
    Reset,
}
