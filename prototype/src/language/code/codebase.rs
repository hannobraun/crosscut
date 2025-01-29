use super::IntrinsicFunction;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    nodes: Vec<Node>,
}

impl Codebase {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn entry(&self) -> Option<Location> {
        // This happens to work right now, because the editor happens to always
        // create an initial fragment, so `Codebase` is never empty.
        Some(Location { index: 0 })
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
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

#[derive(Debug)]
pub struct Location {
    index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    IntrinsicFunction { function: IntrinsicFunction },
}
