use std::collections::BTreeMap;

use super::NodePath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Changes {
    pub inner: BTreeMap<NodePath, NodePath>,
}
