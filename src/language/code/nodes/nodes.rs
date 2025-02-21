use std::collections::BTreeMap;

use super::{Node, NodeHash};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Nodes {
    inner: BTreeMap<NodeHash, Node>,
}

impl Nodes {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn get(&self, hash: &NodeHash) -> &Node {
        let Some(node) = self.inner.get(hash) else {
            unreachable!(
                "This is an append-only data structure. All hashes that were \
                ever created must be valid."
            );
        };

        node
    }

    pub fn insert(&mut self, node: Node) -> NodeHash {
        let hash = NodeHash::new(&node);
        self.inner.insert(hash, node);
        hash
    }
}
