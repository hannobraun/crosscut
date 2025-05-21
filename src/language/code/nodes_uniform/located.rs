use super::{ChildIndex, NodeHash, NodePath, Nodes, SyntaxNode};

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
        hashes_to_located_nodes(self.node.children().hashes, &self.path, nodes)
    }
}

fn hashes_to_located_nodes<'r>(
    hashes: Vec<&NodeHash>,
    parent: &NodePath,
    nodes: &'r Nodes,
) -> impl DoubleEndedIterator<Item = LocatedNode<'r>> {
    hashes
        .into_iter()
        .copied()
        .enumerate()
        .map(|(index, child)| {
            let node = nodes.get(&child);
            let path = NodePath::new(
                child,
                Some((parent.clone(), ChildIndex { index })),
                nodes,
            );

            LocatedNode { node, path }
        })
}
