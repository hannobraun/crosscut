use crate::language::code::{Children, Nodes, SyntaxNode};

pub struct Function {
    pub parameter: SyntaxNode,
}

impl Function {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(self.parameter);
        let body = [SyntaxNode::Empty]
            .into_iter()
            .map(|expression| nodes.insert(expression));

        SyntaxNode::Function {
            parameter,
            body: Children::new(body),
        }
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
