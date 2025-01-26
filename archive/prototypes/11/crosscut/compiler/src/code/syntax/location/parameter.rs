use std::fmt;

use crate::code::{
    syntax::{Binding, Parameter, SyntaxTree, SyntaxType},
    Index,
};

use super::{located::HasLocation, BranchLocation, Located};

impl HasLocation for Parameter {
    type Location = ParameterLocation;
}

impl<'r> Located<&'r Parameter> {
    pub fn into_binding(
        self,
    ) -> Option<(Located<&'r Binding>, Option<&'r SyntaxType>)> {
        self.fragment.as_binding().map(|(binding, type_)| {
            (
                Located {
                    fragment: binding,
                    location: self.location,
                },
                type_,
            )
        })
    }
}

/// # The location of a branch's parameter
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
pub struct ParameterLocation {
    pub parent: Box<BranchLocation>,
    pub index: Index<Parameter>,
}

impl ParameterLocation {
    /// # Create a helper that implements [`fmt::Display`]
    pub fn display<'r>(
        &'r self,
        syntax_tree: &'r SyntaxTree,
    ) -> ParameterLocationDisplay<'r> {
        ParameterLocationDisplay {
            location: self,
            syntax_tree,
        }
    }
}

/// # Helper struct to display [`ParameterLocation`]
///
/// Implements [`fmt::Display`], which [`ParameterLocation`] itself doesn't.
pub struct ParameterLocationDisplay<'r> {
    location: &'r ParameterLocation,
    syntax_tree: &'r SyntaxTree,
}

impl fmt::Display for ParameterLocationDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "parameter {}\n    in {}",
            self.location.index,
            self.location.parent.display(self.syntax_tree)
        )
    }
}
