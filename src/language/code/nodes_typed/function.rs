use crate::{
    language::code::{Nodes, SyntaxNode},
    util::form::{Form, Owned},
};

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
            .map(|expression| nodes.insert(expression))
            .collect();

        SyntaxNode::Function { parameter, body }
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
