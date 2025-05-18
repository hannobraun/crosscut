use crate::language::code::{Children, Nodes, SyntaxNode};

#[derive(Default)]
pub struct Function {}

impl Function {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(SyntaxNode::Binding {
            name: "_".to_string(),
        });
        let body = Children::new([nodes.insert(SyntaxNode::Empty)]);

        SyntaxNode::Function { parameter, body }
    }
}
