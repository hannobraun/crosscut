use crate::language::code::{Children, Nodes, SyntaxNode};

pub struct Tuple;

impl Tuple {
    pub fn to_syntax_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let values = Children::new([]);
        let add_value = nodes.insert(SyntaxNode::AddNode);

        SyntaxNode::Tuple { values, add_value }
    }
}
