use std::fmt;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

use super::Node;

/// # The hash of a syntax node
///
/// The purpose of this type is to serve as a building block for identifying
/// syntax nodes in a unique and versioned manner. But it's important to
/// understand that it is not more than a building block. By itself, a hash
/// can't be unique.
///
/// A hash derives from a syntax node's contents, which include its children.
/// This is not unique, because the syntax tree can contain identical sub-trees
/// in various places, each with a root node that would have identical hashes.
///
/// To guarantee uniqueness, we also need the position of the node within the
/// syntax tree, which includes the parent. But a node's parent already includes
/// its own children, which would cause a circular dependency when computing the
/// hash.
///
/// To solve this issue, we have [`NodePath`], which after the hashes have been
/// constructed bottom-up, is constructed top-down, from the root of the syntax
/// tree.
///
/// ## Implementation Note
///
/// Not everything documented here, especially the relation to [`NodePath`], is
/// implemented yet. I wanted to paint a complete picture of how this fits into
/// my current plans, even though those are not fully realized yet.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct NodeHash {
    hash: [u8; 32],
}

impl NodeHash {
    pub(super) fn new(node: &Node) -> Self {
        let hash = udigest::hash::<blake3::Hasher>(&node).into();
        Self { hash }
    }
}

impl fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NodeHash")
            .field("hash", &BASE64_URL_SAFE_NO_PAD.encode(self.hash))
            .finish()
    }
}

impl fmt::Display for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_URL_SAFE_NO_PAD.encode(self.hash))?;
        Ok(())
    }
}
