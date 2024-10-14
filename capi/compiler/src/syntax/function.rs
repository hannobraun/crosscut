use std::collections::BTreeSet;

use crate::fragments::{FunctionIndexInCluster, Pattern};

use super::Expression;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Function {
    /// The name of the function, if available
    ///
    /// This is `Some` for named functions, `None` for anonymous ones.
    pub name: Option<String>,

    pub branches: Vec<Branch>,

    /// The environment of the function
    ///
    /// These are the values that the function captured from parent scopes.
    ///
    /// The environment is empty on construction, until it is filled in during
    /// the resolve pass.
    pub environment: BTreeSet<String>,

    /// # The index of this function within its cluster
    ///
    /// This starts out as `None`. For named functions, it is later defined by
    /// the compiler pass that groups functions into clusters. It stays `None`
    /// for anonymous functions.
    pub index_in_cluster: Option<FunctionIndexInCluster>,
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Branch {
    pub parameters: Vec<Pattern>,
    pub body: Vec<Expression>,
}
