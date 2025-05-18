use crate::language::code::{Children, Nodes, SyntaxNode};

use super::{Form, Owned};

pub struct Function<T: Form> {
    pub parameter: T::Form<SyntaxNode>,
    pub body: Vec<T::Form<SyntaxNode>>,
}

impl Function<Owned> {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(self.parameter);
        let body = self
            .body
            .into_iter()
            .map(|expression| nodes.insert(expression));

        SyntaxNode::Function {
            parameter,
            body: Children::new(body),
        }
    }
}

impl Default for Function<Owned> {
    fn default() -> Self {
        Self {
            parameter: SyntaxNode::Binding {
                name: "_".to_string(),
            },
            body: vec![SyntaxNode::Empty],
        }
    }
}
