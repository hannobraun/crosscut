use crate::{
    language::code::{NodeAsUniform, NodeHash, Nodes, SyntaxNode},
    util::form::Form,
};

use super::Binding;

pub struct Function<T: Form> {
    pub parameter: T::Form<Binding>,
    pub body: Vec<T::Form<NodeHash>>,
}

impl Function<NodeAsUniform> {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(self.parameter);
        let body = self
            .body
            .into_iter()
            .map(|expression| nodes.insert(expression))
            .collect();

        SyntaxNode::Function { parameter, body }
    }
}

impl Default for Function<NodeAsUniform> {
    fn default() -> Self {
        Self {
            parameter: SyntaxNode::Binding {
                name: "_".to_string(),
            },
            body: vec![SyntaxNode::Empty],
        }
    }
}
