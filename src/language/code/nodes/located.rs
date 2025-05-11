use super::{
    Borrowed, ChildOfExpression, Expression, NodePath, Nodes, SiblingIndex,
};

#[derive(Debug, Eq, PartialEq)]
pub struct LocatedNode<T> {
    pub node: T,
    pub path: NodePath<Expression>,
}

impl LocatedNode<&Expression> {
    pub fn children<'r>(
        &self,
        nodes: &'r Nodes,
    ) -> impl DoubleEndedIterator<Item = LocatedNode<ChildOfExpression<Borrowed<'r>>>>
    {
        self.node
            .children()
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                let ChildOfExpression::Expression(hash) = child;

                let node = {
                    let node = nodes.get(&hash);
                    ChildOfExpression::Expression(node)
                };
                let path = NodePath::new(
                    hash,
                    Some((self.path.clone(), SiblingIndex { index })),
                    nodes,
                );

                LocatedNode { node, path }
            })
    }
}
