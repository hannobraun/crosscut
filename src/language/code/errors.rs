use std::collections::BTreeMap;

use super::NodeHash;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Errors {
    inner: BTreeMap<NodeHash, CodeError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {}
