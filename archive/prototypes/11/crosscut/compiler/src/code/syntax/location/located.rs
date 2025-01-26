use std::ops::{Deref, DerefMut};

/// # A fragment of code, with its location attached
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Located<T: HasLocation> {
    /// # The code fragment
    pub fragment: T,

    /// # The location of the code fragment
    pub location: T::Location,
}

impl<T: HasLocation> Located<T> {
    /// # Maps a `&Located<T>` to a `Located<&T>`
    pub fn as_ref(&self) -> Located<&T>
    where
        T::Location: Clone,
    {
        Located {
            fragment: &self.fragment,
            location: self.location.clone(),
        }
    }
}

impl<T: HasLocation> Located<&T> {
    /// # Maps a `Located<&T>` to a `Located<T>` by cloning the fragment
    pub fn cloned(self) -> Located<T>
    where
        T: Clone,
    {
        Located {
            fragment: self.fragment.clone(),
            location: self.location,
        }
    }
}

impl<T: HasLocation> Deref for Located<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.fragment
    }
}

impl<T: HasLocation> DerefMut for Located<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fragment
    }
}

/// # Implemented by all code fragments, to abstract over their location
pub trait HasLocation {
    /// # The location of this fragment
    type Location;
}

impl<T> HasLocation for &T
where
    T: HasLocation,
{
    type Location = T::Location;
}

impl<T> HasLocation for &mut T
where
    T: HasLocation,
{
    type Location = T::Location;
}
