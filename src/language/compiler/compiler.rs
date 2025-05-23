use crate::language::code::{
    Codebase, Expressions, NodePath, SyntaxNode, Tuple, TypedNode,
};

use super::{expression, replace::replace_node_and_update_parents};

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
            // Compiling the child as an expression is correct for all the
            // parents that we check for above.
            let child = expression::compile(child_token, change_set.nodes);

            let (parent_path, index) = {
                let node = change_set.nodes.get(parent.hash()).clone();

                let (node, index) = match &node {
                    SyntaxNode::Expressions {
                        children: expressions,
                        add,
                    } => {
                        let mut expressions = Expressions {
                            expressions: expressions.clone(),
                            add: *add,
                        };

                        let index = expressions.expressions_mut().add(child);
                        let node = expressions.into_syntax_node();

                        (node, index)
                    }
                    SyntaxNode::Tuple { values, add_value } => {
                        let mut tuple = Tuple {
                            values: values.clone(),
                            add_value: *add_value,
                        };

                        let index = tuple.values_mut().add(child);
                        let node = tuple.into_syntax_node();

                        (node, index)
                    }
                    node => {
                        panic!(
                            "Can't add child to this node:\n\
                            {node:#?}"
                        );
                    }
                };

                let hash = change_set.nodes.insert(node);

                let path =
                    replace_node_and_update_parents(parent, hash, change_set);

                (path, index)
            };

            NodePath::new(child, Some((parent_path, index)), change_set.nodes)
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
