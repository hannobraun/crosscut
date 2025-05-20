use crate::{
    language::code::{ChildIndex, NodeByHash, NodeHash, Nodes, SyntaxNode},
    util::form::{Form, Owned, Ref, RefMut},
};

use super::Child;

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
            expression,
            argument,
        }
    }
}

impl Apply<NodeByHash> {
    pub fn expression(&self) -> Child<Ref> {
        Child::new(&self.expression, 0)
    }

    pub fn argument(&self) -> Child<Ref> {
        Child::new(&self.argument, 1)
    }

    pub fn has_child(&self, hash: &NodeHash, index: &ChildIndex) -> bool {
        self.expression().is(hash, index) || self.argument().is(hash, index)
    }

    pub fn expression_mut(&mut self) -> Child<RefMut> {
        Child::new(&mut self.expression, 0)
    }

    pub fn argument_mut(&mut self) -> Child<RefMut> {
        Child::new(&mut self.argument, 1)
    }

    pub fn replace_child(
        &mut self,
        replace_hash: &NodeHash,
        replace_index: &ChildIndex,
        replacement: NodeHash,
    ) -> bool {
        let replaced_expression = self.expression_mut().replace(
            replace_hash,
            replace_index,
            replacement,
        );
        let replaced_argument = self.argument_mut().replace(
            replace_hash,
            replace_index,
            replacement,
        );

        replaced_expression || replaced_argument
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Apply {
            expression: self.expression,
            argument: self.argument,
        }
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
