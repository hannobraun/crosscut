use crate::{
    language::code::{NodeByHash, Nodes, SyntaxNode},
    util::form::{Form, Owned, Ref, RefMut},
};

use super::TypedChildren;

#[derive(Debug)]
pub struct Tuple<T: Form> {
    pub values: Vec<T::Form<SyntaxNode>>,
    pub add_value: T::Form<SyntaxNode>,
}

impl Tuple<Owned> {
    pub fn empty() -> Self {
        Self {
            values: Vec::new(),
            add_value: SyntaxNode::Add,
        }
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let values = self
            .values
            .into_iter()
            .map(|node| nodes.insert(node))
            .collect();
        let add_value = nodes.insert(self.add_value);

        SyntaxNode::Tuple { values, add_value }
    }
}

impl Tuple<NodeByHash> {
    pub fn values(&self) -> TypedChildren<Ref> {
        TypedChildren::new(&self.values, 0)
    }

    pub fn values_mut(&mut self) -> TypedChildren<RefMut> {
        TypedChildren::new(&mut self.values, 0)
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Tuple {
            values: self.values,
            add_value: self.add_value,
        }
    }
}
