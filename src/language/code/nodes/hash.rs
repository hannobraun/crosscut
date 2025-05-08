use std::{any::type_name, cmp, fmt, marker::PhantomData};

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
pub struct NodeHash<T> {
    hash: RawHash,
    t: PhantomData<T>,
}

impl<T> NodeHash<T> {
    pub fn new(node: &T) -> Self
    where
        T: udigest::Digestable,
    {
        let hash = RawHash {
            inner: udigest::hash::<blake3::Hasher>(&node).into(),
        };

        Self {
            hash,
            t: PhantomData,
        }
    }

    pub fn raw(&self) -> &RawHash {
        &self.hash
    }
}

impl<T> Copy for NodeHash<T> {}

impl<T> Clone for NodeHash<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for NodeHash<T> {}

impl<T> Ord for NodeHash<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl<T> PartialEq for NodeHash<T> {
    fn eq(&self, other: &Self) -> bool {
        let Self { hash, t } = self;

        hash == &other.hash && t == &other.t
    }
}

impl<T> PartialOrd for NodeHash<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> fmt::Debug for NodeHash<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_parameter = {
            let type_name = type_name::<T>();

            if let Some((_, type_parameter)) = type_name.rsplit_once("::") {
                type_parameter
            } else {
                type_name
            }
        };

        let Self { hash, t: _ } = self;

        f.debug_struct(&format!("NodeHash<{type_parameter}>"))
            .field("hash", &hash.to_string())
            .finish()
    }
}

impl<T> udigest::Digestable for NodeHash<T> {
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

impl fmt::Display for RawHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_URL_SAFE_NO_PAD.encode(self.inner))?;
        Ok(())
    }
}
