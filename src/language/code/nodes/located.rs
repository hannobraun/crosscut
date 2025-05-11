use super::{ChildOfExpression, Expression, NodePath, Nodes, SiblingIndex};

#[derive(Debug, Eq, PartialEq)]
pub struct LocatedNode<'r> {
    pub node: &'r Expression,
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
                let ChildOfExpression::Expression(hash) = child;

                let node = nodes.get(&hash);
                let path = NodePath::new(
                    hash,
                    Some((self.path.clone(), SiblingIndex { index })),
                    nodes,
                );

                LocatedNode { node, path }
            })
    }
}
