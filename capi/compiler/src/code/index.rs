use std::{collections::BTreeMap, marker::PhantomData};

/// # A collection of values, in a defined order, accessible through their index
pub struct IndexMap<T, I = T> {
    pub inner: IndexMapInner<T, I>,
}

impl<T, I> IntoIterator for IndexMap<T, I> {
    type Item = <IndexMapInner<T, I> as IntoIterator>::Item;
    type IntoIter = <IndexMapInner<T, I> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'r, T, I> IntoIterator for &'r IndexMap<T, I> {
    type Item = <&'r IndexMapInner<T, I> as IntoIterator>::Item;
    type IntoIter = <&'r IndexMapInner<T, I> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

type IndexMapInner<T, I> = BTreeMap<Index<I>, T>;

/// # The index of a named function in the root context
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Index<T> {
    pub value: u32,
    pub t: PhantomData<T>,
}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Index<T> {}

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
