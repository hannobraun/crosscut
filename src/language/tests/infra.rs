use itertools::Itertools;

use crate::language::code::{
    Children, LocatedNode, NodeHash, Nodes, SyntaxNode,
};

pub fn expression(
    name: &str,
    children: impl IntoIterator<Item = NodeHash>,
) -> SyntaxNode {
    SyntaxNode::Test {
        name: name.to_string(),
        children: Children::new(children),
    }
}

pub fn unresolved(identifier: &str) -> SyntaxNode {
    SyntaxNode::Identifier {
        identifier: identifier.to_string(),
    }
}

pub trait ExpectChildren {
    fn expect_children<'r, const N: usize>(
        &self,
        nodes: &'r Nodes,
    ) -> [LocatedNode<'r>; N];
}

impl ExpectChildren for LocatedNode<'_> {
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
