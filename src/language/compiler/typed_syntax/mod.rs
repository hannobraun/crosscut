mod apply;
mod node;

pub use self::{apply::Apply, node::TypedNode};

use crate::language::code::{Children, Nodes, SyntaxNode};

pub struct Function;

impl Function {
    pub fn to_syntax_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(SyntaxNode::Binding {
            name: "_".to_string(),
        });
        let body = Children::new([nodes.insert(SyntaxNode::Empty)]);

        SyntaxNode::Function { parameter, body }
    }
}

pub struct Tuple;

impl Tuple {
    pub fn to_syntax_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let values = Children::new([]);
        let add_value = nodes.insert(SyntaxNode::AddNode);

        SyntaxNode::Tuple { values, add_value }
    }
}
