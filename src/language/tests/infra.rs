use itertools::Itertools;

use crate::language::code::{
    Children, Expression, LocatedNode, NodeHash, Nodes,
};

pub fn expression(
    name: &str,
    children: impl IntoIterator<Item = NodeHash<Expression>>,
) -> Expression {
    Expression::Test {
        name: name.to_string(),
        children: Children::new(children),
    }
}

pub fn tuple(
    values: impl IntoIterator<Item = NodeHash<Expression>>,
) -> Expression {
    Expression::Tuple {
        values: Children::new(values),
    }
}

pub fn error(name: &str) -> Expression {
    Expression::UnresolvedIdentifier {
        node: name.to_string(),
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
        let Some(children) = self.children(nodes).collect_array() else {
            panic!(
                "Expected {N} children but got {}.",
                self.children(nodes).count(),
            );
        };

        children
    }
}
