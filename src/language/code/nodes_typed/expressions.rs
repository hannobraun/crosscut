use crate::{
    language::code::{NodeByHash, Nodes, SyntaxNode},
    util::form::{Form, Owned, Ref, RefMut},
};

use super::Children;

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

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let expressions = self
            .expressions
            .into_iter()
            .map(|node| nodes.insert(node))
            .collect();
        let add = nodes.insert(self.add);

        SyntaxNode::Expressions {
            children: expressions,
            add,
        }
    }
}

impl Expressions<NodeByHash> {
    pub fn expressions(&self) -> Children<Ref> {
        Children::new(&self.expressions)
    }

    pub fn expressions_mut(&mut self) -> Children<RefMut> {
        Children::new(&mut self.expressions)
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Expressions {
            children: self.expressions,
            add: self.add,
        }
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
