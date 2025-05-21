#![cfg(test)]

use crate::{
    language::code::SyntaxNode,
    util::form::{Form, Owned},
};

#[cfg(test)]
use crate::language::code::Nodes;

pub struct Expressions<T: Form> {
    pub expressions: Vec<T::Form<SyntaxNode>>,
    pub add: T::Form<SyntaxNode>,
}

impl Expressions<Owned> {
    #[cfg(test)]
    pub fn with_expressions(
        mut self,
        values: impl IntoIterator<Item = SyntaxNode>,
    ) -> Self {
        self.expressions = values.into_iter().collect();
        self
    }

    #[cfg(test)]
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let expressions = self
            .expressions
            .into_iter()
            .map(|node| nodes.insert(node))
            .collect();
        let add = nodes.insert(self.add);

        SyntaxNode::Expressions { expressions, add }
    }
}

impl Default for Expressions<Owned> {
    fn default() -> Self {
        Self {
            expressions: Vec::new(),
            add: SyntaxNode::Add,
        }
    }
}
