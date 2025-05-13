use super::{NodePath, Nodes, SiblingIndex, SyntaxNode};

#[derive(Debug, Eq, PartialEq)]
pub struct LocatedNode<'r> {
    pub node: &'r SyntaxNode,
    pub path: NodePath,
}

impl LocatedNode<'_> {
    pub fn children<'r>(
        &self,
        nodes: &'r Nodes,
    ) -> impl DoubleEndedIterator<Item = LocatedNode<'r>> {
        self.node
            .children()
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                let node = nodes.get(&child);
                let path = NodePath::new(
                    child,
                    Some((self.path.clone(), SiblingIndex { index })),
                    nodes,
                );

                LocatedNode { node, path }
            })
    }

    /// # The children of the node that are expressions and its inputs
    ///
    /// These are the children that must be evaluated before the node can be
    /// evaluated.
    pub fn inputs<'r>(
        &self,
        nodes: &'r Nodes,
    ) -> impl DoubleEndedIterator<Item = LocatedNode<'r>> {
        self.children(nodes)
    }
}
