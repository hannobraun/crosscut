use std::fmt;

use crate::code::{
    syntax::{Branch, Function, NamedFunction, SyntaxTree},
    Index,
};

use super::{located::HasLocation, BranchLocation, Located, MemberLocation};

impl HasLocation for Function {
    type Location = FunctionLocation;
}

impl<'r> Located<&'r Function> {
    /// # Iterate over the function's branches
    pub fn branches(&self) -> impl Iterator<Item = Located<&'r Branch>> {
        let function = self.fragment;
        let location = self.location.clone();

        function
            .branches
            .iter()
            .map(move |(&index, branch)| Located {
                fragment: branch,
                location: BranchLocation {
                    parent: Box::new(location.clone()),
                    index,
                },
            })
    }

    /// # Iterate over all local functions in this function, recursively
    pub fn all_local_functions(
        self,
    ) -> impl Iterator<Item = Located<&'r Function>> {
        self.branches().flat_map(|branch| {
            Box::new(branch.all_local_functions())
                as Box<dyn Iterator<Item = Located<&Function>>>
        })
    }

    /// # Access the function's single branch
    ///
    /// Returns `None`, if the function does not have exactly one branch.
    pub fn find_single_branch(&self) -> Option<Located<&Branch>> {
        let function = &self.fragment;
        let location = self.location.clone();

        if function.branches.len() > 1 {
            return None;
        }

        function
            .branches
            .first_key_value()
            .map(|(&index, branch)| Located {
                fragment: branch,
                location: BranchLocation {
                    parent: Box::new(location),
                    index,
                },
            })
    }
}

/// # The location of a function in the source code
#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum FunctionLocation {
    Named { index: Index<NamedFunction> },
    Local { location: MemberLocation },
}

impl FunctionLocation {
    /// # Create a helper that implements [`fmt::Display`]
    pub fn display<'r>(
        &'r self,
        syntax_tree: &'r SyntaxTree,
    ) -> FunctionLocationDisplay<'r> {
        FunctionLocationDisplay {
            location: self,
            syntax_tree,
        }
    }
}

impl From<Index<NamedFunction>> for FunctionLocation {
    fn from(index: Index<NamedFunction>) -> Self {
        Self::Named { index }
    }
}

impl From<MemberLocation> for FunctionLocation {
    fn from(location: MemberLocation) -> Self {
        Self::Local { location }
    }
}

/// # Helper struct to display [`FunctionLocation`]
///
/// Implements [`fmt::Display`], which [`FunctionLocation`] itself doesn't.
pub struct FunctionLocationDisplay<'r> {
    location: &'r FunctionLocation,
    syntax_tree: &'r SyntaxTree,
}

impl fmt::Display for FunctionLocationDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.location {
            FunctionLocation::Named { index } => {
                let name = &self
                    .syntax_tree
                    .named_functions
                    .get(index)
                    .expect("Named function referred to be index must exist")
                    .name;

                write!(f, "named function `{name}`")?;
            }
            FunctionLocation::Local { location } => {
                write!(
                    f,
                    "anonymous function at {}",
                    location.display(self.syntax_tree),
                )?;
            }
        }

        Ok(())
    }
}
