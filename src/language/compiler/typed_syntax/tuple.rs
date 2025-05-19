use crate::language::code::{ChildrenOwned, Nodes, SyntaxNode};

use super::{Form, Owned};

pub struct Tuple<T: Form> {
    pub values: Vec<T::Form<SyntaxNode>>,
    pub add_value: T::Form<SyntaxNode>,
}

impl Tuple<Owned> {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let values = self
            .values
            .into_iter()
            .map(|node| nodes.insert(node))
            .collect::<Vec<_>>();
        let add_value = nodes.insert(self.add_value);

        SyntaxNode::Tuple {
            values: ChildrenOwned::new(values),
            add_value,
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
