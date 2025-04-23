use itertools::Itertools;

use crate::language::code::{Children, LocatedNode, Node, NodeHash, Nodes};

pub fn node(name: &str, children: impl IntoIterator<Item = NodeHash>) -> Node {
    let children = Children::new(children);

    Node::Error {
        node: name.to_string(),
        children,
    }
}

pub trait NodeExt: Sized {
    fn expect_error(&self, expected: &str) -> Node;
}

impl NodeExt for Node {
    #[track_caller]
    fn expect_error(&self, expected: &str) -> Node {
        if let Node::Error { node, .. } = self {
            assert_eq!(node, expected);
            self.clone()
        } else {
            panic!("Expected error.");
        }
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
