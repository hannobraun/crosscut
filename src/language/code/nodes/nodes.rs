use std::collections::BTreeMap;

use super::{NodeHash, SyntaxNode};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Nodes {
    inner: BTreeMap<NodeHash, SyntaxNode>,
}

impl Nodes {
    pub fn get(&self, hash: &NodeHash) -> &SyntaxNode {
        let Some(node) = self.inner.get(hash) else {
            unreachable!(
                "This is an append-only data structure. All hashes that were \
                ever created must be valid."
            );
        };

        node
    }

    pub fn insert(&mut self, node: SyntaxNode) -> NodeHash {
        let hash = NodeHash::new(&node);
        self.inner.insert(hash, node);
        hash
    }
}
