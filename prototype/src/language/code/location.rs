use super::Node;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    pub(super) index: usize,
}

pub struct LocatedNode<'r> {
    pub node: &'r Node,
    pub location: Location,
}
