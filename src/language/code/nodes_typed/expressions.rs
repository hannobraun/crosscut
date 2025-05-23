use crate::{
    language::code::{NodeByHash, Nodes, SyntaxNode},
    util::form::{Form, Owned, Ref, RefMut},
};

use super::Children;

pub struct Expressions<T: Form> {
    pub children: Vec<T::Form<SyntaxNode>>,
    pub add: T::Form<SyntaxNode>,
}

impl Expressions<Owned> {
    #[cfg(test)]
    pub fn with_children(
        mut self,
        children: impl IntoIterator<Item = SyntaxNode>,
    ) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let children = self
            .children
            .into_iter()
            .map(|node| nodes.insert(node))
            .collect();
        let add = nodes.insert(self.add);

        SyntaxNode::Expressions { children, add }
    }
}

impl Expressions<NodeByHash> {
    pub fn children(&self) -> Children<Ref> {
        Children::new(&self.children)
    }

    pub fn expressions_mut(&mut self) -> Children<RefMut> {
        Children::new(&mut self.children)
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Expressions {
            children: self.children,
            add: self.add,
        }
    }
}

impl Default for Expressions<Owned> {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            add: SyntaxNode::Add,
        }
    }
}
