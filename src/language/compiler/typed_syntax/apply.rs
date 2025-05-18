use crate::language::code::{ChildOwned, Nodes, SyntaxNode};

use super::{Child, Form, NodeRef, Owned, Ref};

pub struct Apply<T: Form> {
    pub expression: T::Form<SyntaxNode>,
    pub argument: T::Form<SyntaxNode>,
}

impl Apply<Owned> {
    #[cfg(test)]
    pub fn with_expression(mut self, expression: SyntaxNode) -> Self {
        self.expression = expression;
        self
    }

    #[cfg(test)]
    pub fn with_argument(mut self, argument: SyntaxNode) -> Self {
        self.argument = argument;
        self
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let [expression, argument] =
            [self.expression, self.argument].map(|node| nodes.insert(node));

        SyntaxNode::Apply {
            expression: ChildOwned::new(expression, 0),
            argument: ChildOwned::new(argument, 1),
        }
    }
}

impl Apply<NodeRef<'_>> {
    pub fn expression(&self) -> Child<Ref> {
        Child::new(self.expression, 0)
    }

    pub fn argument(&self) -> Child<Ref> {
        Child::new(self.argument, 1)
    }
}

impl Default for Apply<Owned> {
    fn default() -> Self {
        Self {
            expression: SyntaxNode::Empty,
            argument: SyntaxNode::Empty,
        }
    }
}
