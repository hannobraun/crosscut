use crate::language::code::{Codebase, NodePath, SyntaxNode};

use super::{TypedNode, expression, replace::replace_node_and_update_parents};

pub struct Compiler<'r> {
    codebase: &'r mut Codebase,
}

impl<'r> Compiler<'r> {
    pub fn new(codebase: &'r mut Codebase) -> Self {
        Self { codebase }
    }

    pub fn codebase(&self) -> &Codebase {
        self.codebase
    }

    pub fn insert_child(
        &mut self,
        parent: NodePath,
        child_token: &str,
    ) -> NodePath {
        self.codebase.make_change(|change_set| {
            {
                let parent = change_set.nodes.get(parent.hash());
                assert!(
                    matches!(
                        parent,
                        SyntaxNode::Tuple { .. } | SyntaxNode::Test { .. },
                    ),
                    "Trying to insert child for node that doesn't support it:\n\
                    {parent:#?}",
                );
            }

            // Compiling the child as an expression is correct for all the
            // parents that we check for above.
            let child = expression::compile(child_token, change_set.nodes);

            let (parent_path, sibling_index) = {
                let mut node = change_set.nodes.get(parent.hash()).clone();

                let sibling_index = match &mut node {
                    SyntaxNode::AddNode
                    | SyntaxNode::Apply { .. }
                    | SyntaxNode::Binding { .. }
                    | SyntaxNode::Empty
                    | SyntaxNode::Function { .. }
                    | SyntaxNode::Identifier { .. }
                    | SyntaxNode::Number { .. }
                    | SyntaxNode::Recursion => {
                        panic!(
                            "Can't add child to this node:\n\
                            {node:#?}"
                        );
                    }

                    SyntaxNode::Tuple {
                        values: children, ..
                    } => {
                        let index = children.next_index();
                        children.inner.push(child);
                        index
                    }
                    SyntaxNode::Test { children, .. } => children.add(child),
                };

                let hash = change_set.nodes.insert(node);

                let path =
                    replace_node_and_update_parents(parent, hash, change_set);

                (path, sibling_index)
            };

            NodePath::new(
                child,
                Some((parent_path, sibling_index)),
                change_set.nodes,
            )
        })
    }

    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
    ) -> NodePath {
        self.codebase.make_change(|change_set| {
            let node = change_set.nodes.get(to_replace.hash());

            let replacement = match TypedNode::from_syntax_node(node) {
                TypedNode::Expression => {
                    expression::compile(replacement_token, change_set.nodes)
                }
                TypedNode::Pattern => {
                    change_set.nodes.insert(SyntaxNode::Binding {
                        name: replacement_token.to_string(),
                    })
                }
                TypedNode::Other => {
                    panic!(
                        "Trying to replace unexpected node:\n\
                        {node:#?}"
                    );
                }
            };

            replace_node_and_update_parents(
                to_replace.clone(),
                replacement,
                change_set,
            )
        })
    }
}
