use std::{cmp, fmt};

use super::{NodeHash, RawHash};

pub struct Parent<T> {
    hash: NodeHash<T>,
    parent: RawHash,
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
        let Self { hash, parent } = self;

        hash == &other.hash && parent == &other.parent
    }
}

impl<T> PartialOrd for Parent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> fmt::Debug for Parent<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { hash, parent } = self;

        f.debug_struct("Parent")
            .field("hash", &hash)
            .field("parent", &parent)
            .finish()
    }
}

impl<T> udigest::Digestable for Parent<T> {
    fn unambiguously_encode<B: udigest::Buffer>(
        &self,
        encoder: udigest::encoding::EncodeValue<B>,
    ) {
        let Self { hash, parent } = self;

        let mut encoder = encoder.encode_struct();

        {
            let encoder = encoder.add_field("hash");
            hash.unambiguously_encode(encoder);
        }
        {
            let encoder = encoder.add_field("parent");
            parent.unambiguously_encode(encoder);
        }

        encoder.finish();
    }
}
