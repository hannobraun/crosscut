use std::vec;

use crate::language::{
    code::{
        Expression, IntrinsicFunction, Literal, LocatedNode, Node, NodeKind,
        NodePath, Nodes,
    },
    runtime::{Effect, Value},
};

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
        if let NodeKind::Expression {
            expression:
                Expression::IntrinsicFunction {
                    intrinsic:
                        IntrinsicFunction::Literal {
                            literal: Literal::Integer { value },
                        },
                },
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
            .has_one()
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

pub trait StepUntilFinishedResultExt {
    fn expect_value(self) -> Value;

    fn into_function_body(self) -> Result<NodePath, Self>
    where
        Self: Sized;
}

impl StepUntilFinishedResultExt for Result<Value, Effect> {
    fn expect_value(self) -> Value {
        self.expect("Expected value")
    }

    fn into_function_body(self) -> Result<NodePath, Self> {
        self.map_err(Err).and_then(|value| {
            value
                .into_function_body()
                .map(|body| NodePath { hash: body })
                .map_err(Ok)
        })
    }
}
