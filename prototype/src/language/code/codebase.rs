use std::collections::BTreeMap;

use super::{
    nodes::{NodeHash, Nodes},
    CodeError, LocatedNode, Node, NodePath,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    nodes: Nodes,
    context: Vec<NodeHash>,
    errors: BTreeMap<NodePath, CodeError>,
}

impl Codebase {
    pub fn new() -> Self {
        let mut nodes = Nodes::new();

        let initial = {
            let input = None;
            let node = Node::empty(input);

            nodes.insert(node)
        };

        Self {
            nodes,
            context: vec![initial],
            errors: BTreeMap::new(),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = LocatedNode> {
        self.context
            .iter()
            .enumerate()
            .map(|(index, id)| LocatedNode {
                node: self.nodes.get(id),
                location: NodePath { index },
            })
    }

    pub fn entry(&self) -> NodePath {
        if !self.context.is_empty() {
            NodePath { index: 0 }
        } else {
            unreachable!(
                "`Codebase` is construction with an initial empty fragment, so \
                it should never be empty."
            );
        }
    }

    pub fn location_before(&self, location: &NodePath) -> Option<NodePath> {
        if location.index == 0 {
            None
        } else {
            let previous_index = location.index - 1;

            Some(NodePath {
                index: previous_index,
            })
        }
    }

    pub fn location_after(&self, location: &NodePath) -> Option<NodePath> {
        let next_index = location.index + 1;
        assert!(
            next_index <= self.context.len(),
            "This is an append-only data structure. Every existing `Location` \
            must be valid, or it wouldn't have been created in the first \
            place.\n\
            \n\
            As a result, incrementing the index of an existing location must \
            result in an index that is either valid, or right next to the \
            valid range.",
        );

        if next_index < self.context.len() {
            Some(NodePath { index: next_index })
        } else {
            None
        }
    }

    pub fn node_at(&self, location: &NodePath) -> &Node {
        let Some(id) = self.context.get(location.index) else {
            unreachable!(
                "This is an append-only data structure. Every existing \
                `Location` must be valid, or it wouldn't have been created in \
                the first place."
            );
        };

        self.nodes.get(id)
    }

    pub fn insert_node_after(
        &mut self,
        after: NodePath,
        node: Node,
    ) -> NodePath {
        let hash = self.nodes.insert(node);
        let at = NodePath {
            index: after.index + 1,
        };
        self.context.insert(at.index, hash);
        at
    }

    pub fn replace_node(&mut self, to_replace: &NodePath, replacement: Node) {
        let hash = self.nodes.insert(replacement);
        self.context[to_replace.index] = hash;
    }

    pub fn error_at(&self, location: &NodePath) -> Option<&CodeError> {
        self.errors.get(location)
    }

    pub fn insert_error(&mut self, location: NodePath, error: CodeError) {
        self.errors.insert(location, error);
    }

    pub fn clear_error(&mut self, location: &NodePath) {
        self.errors.remove(location);
    }
}
