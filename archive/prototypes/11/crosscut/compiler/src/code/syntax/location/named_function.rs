use crate::code::{
    syntax::{Function, NamedFunction},
    Index,
};

use super::{located::HasLocation, FunctionLocation, Located};

impl HasLocation for NamedFunction {
    type Location = Index<NamedFunction>;
}

impl<'r> Located<&'r NamedFunction> {
    /// # Access the index of the found function
    ///
    /// This is a convenience accessor, to make code that would otherwise access
    /// `metadata` directly more readable.
    pub fn index(&self) -> Index<NamedFunction> {
        self.location
    }

    /// # Access the location of the found function
    pub fn location(&self) -> FunctionLocation {
        let index = self.location;
        index.into()
    }

    /// # Convert this located named function to a located function
    pub fn into_located_function(self) -> Located<&'r Function> {
        Located {
            fragment: &self.fragment.inner,
            location: FunctionLocation::Named {
                index: self.location,
            },
        }
    }
}
