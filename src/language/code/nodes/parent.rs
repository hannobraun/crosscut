use std::{cmp, fmt};

use super::{NodeHash, RawHash};

pub struct Parent<T> {
    pub hash: NodeHash<T>,
    pub sibling_index: SiblingIndex,
    pub parent: RawHash,
}

impl<T> Copy for Parent<T> {}

impl<T> Clone for Parent<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for Parent<T> {}

impl<T> Ord for Parent<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl<T> PartialEq for Parent<T> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            hash,
            sibling_index,
            parent,
        } = self;

        hash == &other.hash
            && sibling_index == &other.sibling_index
            && parent == &other.parent
    }
}

impl<T> PartialOrd for Parent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> fmt::Debug for Parent<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            hash,
            sibling_index,
            parent,
        } = self;

        f.debug_struct("Parent")
            .field("hash", &hash)
            .field("sibling_index", &sibling_index)
            .field("parent", &parent)
            .finish()
    }
}

impl<T> udigest::Digestable for Parent<T> {
    fn unambiguously_encode<B: udigest::Buffer>(
        &self,
        encoder: udigest::encoding::EncodeValue<B>,
    ) {
        let Self {
            hash,
            sibling_index,
            parent,
        } = self;

        let mut encoder = encoder.encode_struct();

        {
            let encoder = encoder.add_field("hash");
            hash.unambiguously_encode(encoder);
        }
        {
            let encoder = encoder.add_field("sibling_index");
            sibling_index.unambiguously_encode(encoder);
        }
        {
            let encoder = encoder.add_field("parent");
            parent.unambiguously_encode(encoder);
        }

        encoder.finish();
    }
}

/// # The index of a node among its siblings
///
/// ## Implementation Note
///
/// I'm a bit concerned with the use of `usize` here, as it could lead to
/// problems when serializing `Codebase`. But using something else makes some
/// other code much harder to write. I'd basically have to re-implement
/// `iter::Enumerate`, including its implementation of `DoubleEndedIterator, for
/// `u32` or whatever.
///
/// For now, this works. But it might have to change going forward.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct SiblingIndex {
    pub index: usize,
}
