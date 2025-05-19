use crate::language::code::{Nodes, SyntaxNode};

use super::{Child, Children, Form, NodeByHash, Owned, Ref, RefMut};

pub struct Tuple<T: Form> {
    pub values: Vec<T::Form<SyntaxNode>>,
    pub add_value: T::Form<SyntaxNode>,
}

impl Tuple<Owned> {
    #[cfg(test)]
    pub fn with_values(
        mut self,
        values: impl IntoIterator<Item = SyntaxNode>,
    ) -> Self {
        self.values = values.into_iter().collect();
        self
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
    pub fn values(&self) -> Children<Ref> {
        Children::new(&self.values, 0)
    }

    pub fn add_value(&self) -> Child<Ref> {
        Child::new(&self.add_value, self.values.len())
    }

    pub fn values_mut(&mut self) -> Children<RefMut> {
        Children::new(&mut self.values, 0)
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Tuple {
            values: self.values,
            add_value: self.add_value,
        }
    }
}

impl Default for Tuple<Owned> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            add_value: SyntaxNode::AddNode,
        }
    }
}
