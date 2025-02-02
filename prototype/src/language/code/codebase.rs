use std::collections::BTreeMap;

use super::{nodes::NodeId, CodeError, LocatedNode, Location, Node};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    nodes: BTreeMap<NodeId, Node>,
    context: Vec<NodeId>,
    errors: BTreeMap<Location, CodeError>,
}

impl Codebase {
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        let initial = {
            let node = Node::Empty;
            let id = NodeId::generate_for(&node);

            nodes.insert(id, node);

            id
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
                node: self.nodes.get(id).unwrap(),
                location: Location { index },
            })
    }

    pub fn entry(&self) -> Location {
        if !self.context.is_empty() {
            Location { index: 0 }
        } else {
            unreachable!(
                "`Codebase` is construction with an initial empty fragment, so \
                it should never be empty."
            );
        }
    }

    pub fn location_before(&self, location: &Location) -> Option<Location> {
        if location.index == 0 {
            None
        } else {
            let previous_index = location.index - 1;

            Some(Location {
                index: previous_index,
            })
        }
    }

    pub fn location_after(&self, location: &Location) -> Option<Location> {
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
            Some(Location { index: next_index })
        } else {
            None
        }
    }

    pub fn node_at(&self, location: &Location) -> &Node {
        let Some(id) = self.context.get(location.index) else {
            unreachable!(
                "This is an append-only data structure. Every existing \
                `Location` must be valid, or it wouldn't have been created in \
                the first place."
            );
        };

        self.nodes.get(id).unwrap()
    }

    pub fn insert_node_after(
        &mut self,
        after: Location,
        node: Node,
    ) -> Location {
        let id = NodeId::generate_for(&node);
        let at = Location {
            index: after.index + 1,
        };
        self.nodes.insert(id, node);
        self.context.insert(at.index, id);
        at
    }

    pub fn replace_node(&mut self, to_replace: &Location, replacement: Node) {
        let id = NodeId::generate_for(&replacement);
        self.nodes.insert(id, replacement);
        self.context[to_replace.index] = id;
    }

    pub fn error_at(&self, location: &Location) -> Option<&CodeError> {
        self.errors.get(location)
    }

    pub fn insert_error(&mut self, location: Location, error: CodeError) {
        self.errors.insert(location, error);
    }

    pub fn clear_error(&mut self, location: &Location) {
        self.errors.remove(location);
    }
}
