use crate::language::code::{ChildrenOwned, Nodes, SyntaxNode};

pub struct Tuple {
    pub values: Vec<SyntaxNode>,
    pub add_value: SyntaxNode,
}

impl Tuple {
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

impl Default for Tuple {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            add_value: SyntaxNode::AddNode,
        }
    }
}
