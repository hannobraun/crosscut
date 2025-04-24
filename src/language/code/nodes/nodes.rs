use std::collections::BTreeMap;

use super::{Expression, NodeHash};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Nodes {
    inner: BTreeMap<NodeHash<Expression>, Expression>,
}

impl Nodes {
    pub fn get(&self, hash: &NodeHash<Expression>) -> &Expression {
        let Some(node) = self.inner.get(hash) else {
            unreachable!(
                "This is an append-only data structure. All hashes that were \
                ever created must be valid."
            );
        };

        node
    }

    pub fn insert(&mut self, node: Expression) -> NodeHash<Expression> {
        let hash = NodeHash::new(&node);
        self.inner.insert(hash, node);
        hash
    }
}
