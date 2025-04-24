use std::{fmt, marker::PhantomData};

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
/// If you need to uniquely identify a node within a syntax tree, please use
/// [`NodePath`].
///
/// [`NodePath`]: super::NodePath
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct NodeHash<T> {
    hash: [u8; 32],
    _t: PhantomData<T>,
}

impl<T> NodeHash<T> {
    pub(super) fn new(node: &T) -> Self
    where
        T: udigest::Digestable,
    {
        let hash = udigest::hash::<blake3::Hasher>(&node).into();
        Self {
            hash,
            _t: PhantomData,
        }
    }
}

impl<T> Copy for NodeHash<T> where T: Clone {}

impl fmt::Debug for NodeHash<Node> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NodeHash")
            .field("hash", &BASE64_URL_SAFE_NO_PAD.encode(self.hash))
            .finish()
    }
}

impl fmt::Display for NodeHash<Node> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_URL_SAFE_NO_PAD.encode(self.hash))?;
        Ok(())
    }
}
