use std::collections::BTreeMap;

use super::syntax::{Function, FunctionLocation, Located};

/// # All functions in the program
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Functions {
    /// # The functions
    pub inner: BTreeMap<FunctionLocation, Function>,
}

impl Functions {
    /// # Access the function at the given location
    ///
    /// This includes both named and anonymous functions.
    ///
    /// Returns `None`, if the given location does not identify a function.
    pub fn by_location(
        &self,
        location: &FunctionLocation,
    ) -> Option<Located<&Function>> {
        self.inner.get(location).map(|function| Located {
            fragment: function,
            location: location.clone(),
        })
    }

    /// # Iterate over all functions, both named and anonymous
    pub fn all_functions(&self) -> impl Iterator<Item = Located<&Function>> {
        self.inner.iter().map(|(location, function)| Located {
            fragment: function,
            location: location.clone(),
        })
    }
}
