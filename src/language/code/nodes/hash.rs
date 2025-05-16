use std::fmt;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

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
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct NodeHash {
    inner: [u8; 32],
}

impl NodeHash {
    /// # Compute the hash of a node
    ///
    /// This must not be available outside of `super`, since `Nodes` relies on
    /// the fact that no hashes can get created for nodes that have not been
    /// inserted.
    pub(super) fn new<T>(node: &T) -> Self
    where
        T: udigest::Digestable,
    {
        Self {
            inner: udigest::hash::<blake3::Hasher>(node).into(),
        }
    }
}

impl fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NodeHash")
            .field("inner", &self.to_string())
            .finish()
    }
}

impl fmt::Display for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_URL_SAFE_NO_PAD.encode(self.inner))?;
        Ok(())
    }
}
