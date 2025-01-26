use std::{
    collections::{btree_map, BTreeMap},
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// # A collection of values, in a defined order, accessible through their index
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct IndexMap<T> {
    inner: IndexMapInner<T>,
}

impl<T> IndexMap<T> {
    /// # Compute the index of the next value that's going to be pushed
    pub fn next_index(&self) -> Index<T> {
        let index = self
            .inner
            .last_key_value()
            .map(|(&Index { value: index, .. }, _)| index + 1)
            .unwrap_or(0);

        Index {
            value: index,
            t: PhantomData,
        }
    }

    /// # Add another value to the map
    ///
    /// Creates an index based on the index of the last value in the map. Please
    /// note that this is not guaranteed to be a unique index, if you have
    /// previously removed the last entry.
    pub fn push(&mut self, value: T) -> Index<T> {
        let index = self.next_index();

        self.inner.insert(index, value);

        index
    }

    /// # Consume the map and return an iterator over its values
    pub fn into_values(self) -> btree_map::IntoValues<Index<T>, T> {
        self.inner.into_values()
    }
}

impl<T> Default for IndexMap<T> {
    fn default() -> Self {
        Self {
            inner: IndexMapInner::default(),
        }
    }
}

impl<T> Deref for IndexMap<T> {
    type Target = IndexMapInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for IndexMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> FromIterator<T> for IndexMap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut index_map = Self::default();

        for item in iter {
            index_map.push(item);
        }

        index_map
    }
}

impl<T> IntoIterator for IndexMap<T> {
    type Item = <IndexMapInner<T> as IntoIterator>::Item;
    type IntoIter = <IndexMapInner<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'r, T> IntoIterator for &'r IndexMap<T> {
    type Item = <&'r IndexMapInner<T> as IntoIterator>::Item;
    type IntoIter = <&'r IndexMapInner<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

type IndexMapInner<T> = BTreeMap<Index<T>, T>;

/// # The index of a named function in the root context
#[derive(Debug)]
pub struct Index<T> {
    value: u32,
    t: PhantomData<T>,
}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Index<T> {}

impl<T> Eq for Index<T> {}

impl<T> From<u32> for Index<T> {
    fn from(value: u32) -> Self {
        Self {
            value,
            t: PhantomData,
        }
    }
}

impl<T> Ord for Index<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T> PartialOrd for Index<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> fmt::Display for Index<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.value)
    }
}

impl<'de, T> serde::Deserialize<'de> for Index<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            value: u32::deserialize(deserializer)?,
            t: PhantomData,
        })
    }
}

impl<T> serde::Serialize for Index<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<T> udigest::Digestable for Index<T> {
    fn unambiguously_encode<B: udigest::Buffer>(
        &self,
        encoder: udigest::encoding::EncodeValue<B>,
    ) {
        self.value.unambiguously_encode(encoder);
    }
}
