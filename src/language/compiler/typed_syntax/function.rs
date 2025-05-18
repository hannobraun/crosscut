use crate::language::code::{Children, Nodes, SyntaxNode};

pub struct Function {
    pub parameter: SyntaxNode,
}

impl Function {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(self.parameter);
        let body = Children::new([nodes.insert(SyntaxNode::Empty)]);

        SyntaxNode::Function { parameter, body }
    }
}

impl Default for Function {
    fn default() -> Self {
        Self {
            parameter: SyntaxNode::Binding {
                name: "_".to_string(),
            },
        }
    }
}
