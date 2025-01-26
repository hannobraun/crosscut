use std::{fmt, marker::PhantomData};

#[derive(
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Hash<T> {
    hash: [u8; 32],
    _t: PhantomData<T>,
}

impl<T> Hash<T> {
    pub fn new(value: &T) -> Self
    where
        T: udigest::Digestable,
    {
        let hash = udigest::hash::<blake3::Hasher>(value).into();
        Self {
            hash,
            _t: PhantomData,
        }
    }
}

impl<T> Clone for Hash<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Hash<T> {}

impl<T> fmt::Display for Hash<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for b in &self.hash {
            write!(f, "{b:x}")?;
        }

        writeln!(f)
    }
}
