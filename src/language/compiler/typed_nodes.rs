use crate::language::code::{Children, Nodes, SyntaxNode};

pub struct Function;

impl Function {
    pub fn to_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(SyntaxNode::Binding {
            identifier: "_".to_string(),
        });
        let body = nodes.insert(SyntaxNode::Empty);

        SyntaxNode::Function { parameter, body }
    }
}

pub struct Tuple;

impl Tuple {
    pub fn to_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let values = Children::new([]);
        let add_value = nodes.insert(SyntaxNode::AddValue);

        SyntaxNode::Tuple { values, add_value }
    }
}
