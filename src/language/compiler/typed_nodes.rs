use crate::language::code::{Children, SyntaxNode};

pub struct Tuple;

impl Tuple {
    pub fn to_node(&self) -> SyntaxNode {
        let values = Children::new([]);
        SyntaxNode::Tuple { values }
    }
}
