use std::{cmp, fmt, marker::PhantomData};

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
pub struct NodeHash {
    hash: RawHash,
    t: PhantomData<()>,
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
            hash: RawHash::new(node),
            t: PhantomData,
        }
    }

    pub fn raw(&self) -> &RawHash {
        &self.hash
    }
}

impl Copy for NodeHash {}

impl Clone for NodeHash {
    fn clone(&self) -> Self {
        *self
    }
}

impl Eq for NodeHash {}

impl Ord for NodeHash {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { hash, t } = self;

        match hash.cmp(&other.hash) {
            cmp::Ordering::Equal => {}
            ordering => {
                return ordering;
            }
        }
        t.cmp(&other.t)
    }
}

impl PartialEq for NodeHash {
    fn eq(&self, other: &Self) -> bool {
        let Self { hash, t } = self;

        hash == &other.hash && t == &other.t
    }
}

impl PartialOrd for NodeHash {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { hash, t: _ } = self;

        f.debug_struct("NodeHash")
            .field("hash", &hash.to_string())
            .finish()
    }
}

impl udigest::Digestable for NodeHash {
    fn unambiguously_encode<B: udigest::Buffer>(
        &self,
        encoder: udigest::encoding::EncodeValue<B>,
    ) {
        let Self { hash, t } = self;

        let mut encoder = encoder.encode_struct();

        {
            let encoder = encoder.add_field("hash");
            hash.unambiguously_encode(encoder);
        }
        {
            let encoder = encoder.add_field("t");
            t.unambiguously_encode(encoder);
        }

        encoder.finish();
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct RawHash {
    inner: [u8; 32],
}

impl RawHash {
    pub fn new<T>(value: &T) -> Self
    where
        T: udigest::Digestable,
    {
        Self {
            inner: udigest::hash::<blake3::Hasher>(value).into(),
        }
    }
}

impl fmt::Display for RawHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_URL_SAFE_NO_PAD.encode(self.inner))?;
        Ok(())
    }
}
