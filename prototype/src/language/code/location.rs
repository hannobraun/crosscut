use super::{Location, Node};

#[derive(Debug, Eq, PartialEq)]
pub struct LocatedNode<'r> {
    pub node: &'r Node,
    pub location: Location,
}
