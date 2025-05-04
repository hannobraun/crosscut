use itertools::Itertools;

use crate::language::code::{
    ChildOfExpression, Children, Expression, LocatedNode, NodeHash, Nodes,
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
    ) -> [LocatedNode<ChildOfExpression<'r>>; N];
}

impl ExpectChildren for LocatedNode<&Expression> {
    #[track_caller]
    fn expect_children<'r, const N: usize>(
        &self,
        nodes: &'r Nodes,
    ) -> [LocatedNode<ChildOfExpression<'r>>; N] {
        let Some(children) = self.children(nodes).collect_array() else {
            panic!(
                "Expected {N} children but got {}.",
                self.children(nodes).count(),
            );
        };

        children
    }
}

pub trait ExpectExpression<'r> {
    fn expect_expression(self) -> LocatedNode<&'r Expression>;
}

impl<'r> ExpectExpression<'r> for LocatedNode<ChildOfExpression<'r>> {
    fn expect_expression(self) -> LocatedNode<&'r Expression> {
        let ChildOfExpression::Expression(node) = self.node;

        LocatedNode {
            node,
            path: self.path,
        }
    }
}
