use std::vec;

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
    fn expect_empty(&self) -> Node;
    fn expect_error(&self, expected: &str) -> Node;
    fn expect_integer_literal(&self, value: i32) -> Node;
    fn expect_single_child(&self, nodes: &Nodes) -> Node;
}

impl NodeExt for Node {
    #[track_caller]
    fn expect_empty(&self) -> Node {
        if let Node::Empty = self {
            self.clone()
        } else {
            panic!("Expected empty node.");
        }
    }

    #[track_caller]
    fn expect_error(&self, expected: &str) -> Node {
        if let Node::Error { node, .. } = self {
            assert_eq!(node, expected);
            self.clone()
        } else {
            panic!("Expected error.");
        }
    }

    #[track_caller]
    fn expect_integer_literal(&self, expected: i32) -> Node {
        if let Node::LiteralNumber { value } = self {
            assert_eq!(value, &expected);
            self.clone()
        } else {
            panic!("Expected integer literal.");
        }
    }

    #[track_caller]
    fn expect_single_child(&self, nodes: &Nodes) -> Node {
        let hash = self
            .has_single_child()
            .expect("Expected node to have single child");
        nodes.get(hash).clone()
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

pub trait NodesExt {
    fn expect_errors(self) -> vec::IntoIter<String>;
}

impl<'r, T> NodesExt for T
where
    T: Iterator<Item = LocatedNode<'r>>,
{
    fn expect_errors(self) -> vec::IntoIter<String> {
        self.map(|located_node| {
            let Node::Error { node, .. } = located_node.node else {
                panic!("Expected error, got {:?}", located_node.node);
            };

            node.clone()
        })
        .collect::<Vec<_>>()
        .into_iter()
    }
}
