use std::iter;

use crate::code::{
    syntax::{
        parse::parse, BranchLocation, FunctionLocation, Located,
        ParameterLocation,
    },
    IndexMap, Tokens,
};

use super::function::{Binding, Branch, Function, NamedFunction, Parameter};

/// # The syntax tree
///
/// See [parent module](super).
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SyntaxTree {
    pub named_functions: IndexMap<NamedFunction>,
}

impl SyntaxTree {
    pub fn parse(tokens: Tokens) -> Self {
        let named_functions = parse(tokens);
        Self { named_functions }
    }

    /// # Find the function at the provided location
    ///
    /// Returns `None`, if no function can be found at this location.
    pub fn function_by_location<'r>(
        &'r self,
        location: &FunctionLocation,
    ) -> Option<Located<&'r Function>> {
        let function = match location {
            FunctionLocation::Named { index } => {
                let named_function = self.named_functions.get(index)?;
                &named_function.inner
            }
            FunctionLocation::Local { location } => {
                let branch_location = &location.parent;
                let parent_location = &branch_location.parent;

                let parent = self.function_by_location(parent_location)?;
                let branch = parent.branches.get(&branch_location.index)?;
                let member = branch.body.get(&location.index)?;

                member.as_expression()?.as_local_function()?
            }
        };

        Some(Located {
            fragment: function,
            location: location.clone(),
        })
    }

    /// # Find the branch at the provided location
    ///
    /// Returns `None`, if no branch can be found at this location.
    pub fn branch_by_location<'r>(
        &'r self,
        location: &BranchLocation,
    ) -> Option<Located<&'r Branch>> {
        let function = self.function_by_location(&location.parent)?;
        let branch = function.branches.get(&location.index)?;

        Some(Located {
            fragment: branch,
            location: location.clone(),
        })
    }

    /// # Find the binding at the provided location
    ///
    /// Returns `None`, if no binding can be found at this location.
    pub fn binding_by_location<'r>(
        &'r self,
        location: &ParameterLocation,
    ) -> Option<Located<&'r Binding>> {
        let branch = self.branch_by_location(&location.parent)?;
        let parameter = branch.parameters.get(&location.index)?;

        let Parameter::Binding { binding, .. } = parameter else {
            return None;
        };

        Some(Located {
            fragment: binding,
            location: location.clone(),
        })
    }

    /// # Find the function with the provided name
    ///
    /// Returns `None`, if no function with this name can be found.
    pub fn function_by_name(
        &self,
        name: &str,
    ) -> Option<Located<&NamedFunction>> {
        self.named_functions()
            .find(|function| function.name == name)
    }

    /// # Find the top-level parent of a given function
    ///
    /// If the function at the provided location has no parent, the function
    /// itself is returned.
    ///
    /// Returns `None`, if there is no function at the provided location.
    pub fn find_top_level_parent_function(
        &self,
        location: &FunctionLocation,
    ) -> Option<Located<&NamedFunction>> {
        let index = match location {
            FunctionLocation::Named { index } => index,
            FunctionLocation::Local { location } => {
                return self
                    .find_top_level_parent_function(&location.parent.parent);
            }
        };

        let named_function = self.named_functions.get(index)?;

        Some(Located {
            fragment: named_function,
            location: *index,
        })
    }

    /// # Iterate over the named functions
    pub fn named_functions(
        &self,
    ) -> impl Iterator<Item = Located<&NamedFunction>> {
        self.named_functions
            .iter()
            .map(|(&index, function)| Located {
                fragment: function,
                location: index,
            })
    }

    /// # Iterate over all functions, both named and anonymous
    pub fn all_functions(&self) -> impl Iterator<Item = Located<&Function>> {
        self.named_functions().flat_map(|named_function| {
            let function = named_function.into_located_function();
            iter::once(function.clone()).chain(function.all_local_functions())
        })
    }
}
