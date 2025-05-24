use crate::{
    language::code::{NodeByHash, NodeHash, Nodes, SyntaxNode},
    util::form::{Form, Owned, Ref, RefMut},
};

use super::TypedChildren;

pub struct Body<T: Form> {
    /// # The children of the body
    ///
    /// This refers to all expressions in the body by hash, regardless of what
    /// [`Form`] is passed as a type parameter. This is required, so function
    /// values can be constructed from this type, at runtime.
    pub children: Vec<T::Form<NodeHash>>,

    pub add: T::Form<SyntaxNode>,
}

impl Body<Owned> {
    #[cfg(test)]
    pub fn with_children(
        mut self,
        children: impl IntoIterator<Item = SyntaxNode>,
        nodes: &mut Nodes,
    ) -> Self {
        self.children = children
            .into_iter()
            .map(|node| nodes.insert(node))
            .collect();

        self
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let children = self.children;
        let add = nodes.insert(self.add);

        SyntaxNode::Body { children, add }
    }
}

impl Body<NodeByHash> {
    pub fn children(&self) -> TypedChildren<Ref> {
        TypedChildren::new(&self.children, 0)
    }

    pub fn children_mut(&mut self) -> TypedChildren<RefMut> {
        TypedChildren::new(&mut self.children, 0)
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        SyntaxNode::Body {
            children: self.children,
            add: self.add,
        }
    }
}

impl Default for Body<Owned> {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            add: SyntaxNode::Add,
        }
    }
}
