use std::collections::BTreeMap;

use super::{CodeError, LocatedNode, Location, Node};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    nodes: Vec<Node>,
    errors: BTreeMap<Location, CodeError>,
}

impl Codebase {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            errors: BTreeMap::new(),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = LocatedNode> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| LocatedNode {
                node,
                location: Location { index },
            })
    }

    pub fn entry(&self) -> Location {
        assert!(
            !self.nodes.is_empty(),
            "The editor always creates an empty fragment to edit, so \
            `Codebase` should never be empty.",
        );

        Location { index: 0 }
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
            next_index <= self.nodes.len(),
            "This is an append-only data structure. Every existing `Location` \
            must be valid, or it wouldn't have been created in the first \
            place.\n\
            \n\
            As a result, incrementing the index of an existing location must \
            result in an index that is either valid, or right next to the \
            valid range.",
        );

        if next_index < self.nodes.len() {
            Some(Location { index: next_index })
        } else {
            None
        }
    }

    pub fn node_at(&self, location: &Location) -> &Node {
        let Some(node) = self.nodes.get(location.index) else {
            unreachable!(
                "This is an append-only data structure. Every existing \
                `Location` must be valid, or it wouldn't have been created in \
                the first place."
            );
        };

        node
    }

    pub fn push_node(&mut self, node: Node) -> Location {
        let location = Location {
            index: self.nodes.len(),
        };
        self.nodes.push(node);
        location
    }

    pub fn replace_node(&mut self, to_replace: &Location, replacement: Node) {
        self.nodes[to_replace.index] = replacement;
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
