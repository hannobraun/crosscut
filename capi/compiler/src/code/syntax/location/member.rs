use std::fmt;

use crate::code::{
    syntax::{Expression, Member, SyntaxTree},
    Index,
};

use super::{located::HasLocation, BranchLocation, Located};

impl HasLocation for Member {
    type Location = MemberLocation;
}

impl<'r> Located<&'r Member> {
    pub fn into_expression(self) -> Option<Located<&'r Expression>> {
        let Member::Expression { expression, .. } = self.fragment;

        Some(Located {
            fragment: expression,
            location: self.location,
        })
    }
}

/// # The location of a member of a branch body
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
pub struct MemberLocation {
    pub parent: Box<BranchLocation>,
    pub index: Index<Member>,
}

impl MemberLocation {
    /// # Create a helper that implements [`fmt::Display`]
    pub fn display<'r>(
        &'r self,
        syntax_tree: &'r SyntaxTree,
    ) -> MemberLocationDisplay<'r> {
        MemberLocationDisplay {
            location: self,
            syntax_tree,
        }
    }
}

/// # Helper struct to display [`ExpressionLocation`]
///
/// Implements [`fmt::Display`], which [`ExpressionLocation`] itself doesn't.
pub struct MemberLocationDisplay<'r> {
    location: &'r MemberLocation,
    syntax_tree: &'r SyntaxTree,
}

impl fmt::Display for MemberLocationDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "expression {}\n    in {}",
            self.location.index,
            self.location.parent.display(self.syntax_tree)
        )
    }
}