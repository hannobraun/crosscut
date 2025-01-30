use super::{IntrinsicFunction, Location};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    nodes: Vec<Node>,
}

impl Codebase {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn nodes(&self) -> impl Iterator<Item = (Location, &Node)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (Location { index }, node))
    }

    pub fn entry(&self) -> Location {
        assert!(
            !self.nodes.is_empty(),
            "The editor always creates an empty fragment to edit, so \
            `Codebase` should never be empty.",
        );

        Location { index: 0 }
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

    pub fn push(&mut self, node: Node) -> Location {
        let location = Location {
            index: self.nodes.len(),
        };
        self.nodes.push(node);
        location
    }

    pub fn replace(&mut self, to_replace: &Location, replacement: Node) {
        self.nodes[to_replace.index] = replacement;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
    Empty,
    Expression { expression: Expression },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    HostFunction { id: u32 },
    IntrinsicFunction { function: IntrinsicFunction },
}
