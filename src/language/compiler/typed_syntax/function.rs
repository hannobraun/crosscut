use crate::language::code::{NodeHash, Nodes, SiblingIndex, SyntaxNode};

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
            .map(|expression| nodes.insert(expression))
            .collect();

        SyntaxNode::Function { parameter, body }
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

    pub fn body_mut(&mut self) -> Children<RefMut> {
        Children::new(&mut self.body, 1)
    }

    pub fn replace_child(
        &mut self,
        replace_hash: &NodeHash,
        replace_index: &SiblingIndex,
        replacement: NodeHash,
    ) -> bool {
        let replaced_parameter = self.parameter_mut().replace(
            replace_hash,
            replace_index,
            replacement,
        );
        let replaced_in_body =
            self.body_mut()
                .replace(replace_hash, replace_index, replacement);

        replaced_parameter || replaced_in_body
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Function {
            parameter: self.parameter,
            body: self.body,
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
