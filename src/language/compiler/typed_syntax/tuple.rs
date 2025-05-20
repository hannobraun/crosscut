use crate::{
    language::code::{ChildIndex, NodeByHash, NodeHash, Nodes, SyntaxNode},
    util::form::{Form, Owned, Ref, RefMut},
};

use super::{Child, Children};

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

    pub fn has_child(&self, hash: &NodeHash, index: &ChildIndex) -> bool {
        self.values().contains(hash, index) || self.add_value().is(hash, index)
    }

    pub fn values_mut(&mut self) -> Children<RefMut> {
        Children::new(&mut self.values, 0)
    }

    pub fn replace_child(
        &mut self,
        replace_hash: &NodeHash,
        replace_index: &ChildIndex,
        replacement: NodeHash,
    ) -> bool {
        self.values_mut()
            .replace(replace_hash, replace_index, replacement)
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
