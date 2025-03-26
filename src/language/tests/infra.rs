use std::vec;

use crate::language::code::{Literal, LocatedNode, Node, NodeKind, Nodes};

pub trait NodeExt: Sized {
    fn expect_empty(&self) -> Node;
    fn expect_integer_literal(&self, value: i32) -> Node;
    fn expect_single_child(&self, nodes: &Nodes) -> Node;
}

impl NodeExt for Node {
    fn expect_empty(&self) -> Node {
        if let NodeKind::Empty = self.kind() {
            self.clone()
        } else {
            panic!("Expected empty node.");
        }
    }

    fn expect_integer_literal(&self, expected: i32) -> Node {
        if let NodeKind::Literal {
            literal: Literal::Integer { value },
        } = self.kind()
        {
            assert_eq!(value, &expected);
            self.clone()
        } else {
            panic!("Expected integer literal.");
        }
    }

    fn expect_single_child(&self, nodes: &Nodes) -> Node {
        let hash = self
            .children()
            .is_single_child()
            .expect("Expected node to have single child");
        nodes.get(hash).clone()
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
            let NodeKind::Error { node } = located_node.node.kind() else {
                panic!("Expected error, got {:?}", located_node.node.kind());
            };

            node.clone()
        })
        .collect::<Vec<_>>()
        .into_iter()
    }
}
