use crate::language::code::{ChildrenOwned, Nodes, SyntaxNode};

pub struct Tuple;

impl Tuple {
    pub fn to_syntax_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let values = ChildrenOwned::new([]);
        let add_value = nodes.insert(SyntaxNode::AddNode);

        SyntaxNode::Tuple { values, add_value }
    }
}
