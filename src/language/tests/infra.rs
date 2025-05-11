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

pub fn unresolved(identifier: &str) -> Expression {
    Expression::UnresolvedIdentifier {
        identifier: identifier.to_string(),
    }
}

pub trait ExpectChildren {
    fn expect_children<'r, const N: usize>(
        &self,
        nodes: &'r Nodes,
    ) -> [LocatedNode<&'r Expression>; N];
}

impl ExpectChildren for LocatedNode<&Expression> {
    #[track_caller]
    fn expect_children<'r, const N: usize>(
        &self,
        nodes: &'r Nodes,
    ) -> [LocatedNode<&'r Expression>; N] {
        let Some(children) = self.children(nodes).collect_array() else {
            panic!(
                "Expected {N} children but got {}.",
                self.children(nodes).count(),
            );
        };

        children.map(|child| LocatedNode {
            node: child.node,
            path: child.path,
        })
    }
}
