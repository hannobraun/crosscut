use crate::language::code::{ChildrenOwned, Nodes, SyntaxNode};

use super::{Child, Children, Form, NodeByHash, Owned, Ref, RefMut};

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
            body: ChildrenOwned::new(body),
        }
    }
}

impl Function<NodeByHash> {
    pub fn parameter(&self) -> Child<Ref> {
        Child::new(&self.parameter, 0)
    }

    pub fn body(&self) -> Children<Ref> {
        Children::new(&self.body, 1)
    }

    pub fn parameter_mut(&mut self) -> Child<RefMut> {
        Child::new(&mut self.parameter, 0)
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Function {
            parameter: self.parameter,
            body: ChildrenOwned::new(self.body),
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
