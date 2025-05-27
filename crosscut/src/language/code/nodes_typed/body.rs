use crate::{
    language::code::{NodeByHash, NodeHash, Nodes, SyntaxNode},
    util::form::{Form, Owned, Ref, RefMut},
};

use super::TypedChildren;

#[derive(Debug)]
pub struct Body<T: Form> {
    pub children: Vec<T::Form<SyntaxNode>>,
    pub add: T::Form<SyntaxNode>,
}

impl Body<Owned> {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            add: SyntaxNode::Add,
        }
    }

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

        SyntaxNode::Body { children, add }
    }
}

impl Body<NodeByHash> {
    pub fn from_hash(hash: &NodeHash, nodes: &Nodes) -> Self {
        let SyntaxNode::Body { children, add } = nodes.get(hash) else {
            panic!("Expected body.");
        };

        let children = children.clone();
        let add = *add;

        Self { children, add }
    }

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
