use itertools::Itertools;

use crate::language::code::{
    Children, Expression, LocatedNode, NodeHash, Nodes,
};

pub fn node(
    name: &str,
    children: impl IntoIterator<Item = NodeHash<Expression>>,
) -> Expression {
    let children = Children::new(children);

    Expression::Error {
        node: name.to_string(),
        children,
    }
}

pub trait LocatedNodeExt {
    fn expect_children<'r, const N: usize>(
        &self,
        nodes: &'r Nodes,
    ) -> [LocatedNode<'r>; N];
}

impl LocatedNodeExt for LocatedNode<'_> {
    #[track_caller]
    fn expect_children<'r, const N: usize>(
        &self,
        nodes: &'r Nodes,
    ) -> [LocatedNode<'r>; N] {
        self.children(nodes).collect_array().unwrap()
    }
}
