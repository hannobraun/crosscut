use crate::language::code::{Children, Nodes, SyntaxNode};

pub struct Function;

impl Function {
    pub fn to_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let [parameter, body] = [nodes.insert(SyntaxNode::Empty); 2];
        SyntaxNode::Function { parameter, body }
    }
}

pub struct Tuple;

impl Tuple {
    pub fn to_node(&self) -> SyntaxNode {
        let values = Children::new([]);
        SyntaxNode::Tuple { values }
    }
}
