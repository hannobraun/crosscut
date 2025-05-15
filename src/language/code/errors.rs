use std::collections::BTreeMap;

use super::NodeHash;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Errors {
    inner: BTreeMap<NodeHash, CodeError>,
}

impl Errors {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {}
