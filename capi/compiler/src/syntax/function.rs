use std::collections::{BTreeMap, BTreeSet};

use crate::fragments::{
    BranchIndex, FragmentIndexInBranchBody, FunctionIndexInCluster, Pattern,
};

use super::Fragment;

#[derive(
    Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct Function {
    /// The name of the function, if available
    ///
    /// This is `Some` for named functions, `None` for anonymous ones.
    pub name: Option<String>,

    pub branches: BTreeMap<BranchIndex, Branch>,

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

impl Function {
    pub fn add_branch(&mut self, branch: Branch) {
        let index = self
            .branches
            .last_key_value()
            .map(|(&BranchIndex(index), _)| index)
            .unwrap_or(0);

        self.branches.insert(BranchIndex(index + 1), branch);
    }
}

#[derive(
    Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct Branch {
    pub parameters: Vec<Pattern>,
    pub body: BTreeMap<FragmentIndexInBranchBody, Fragment>,
}

impl Branch {
    pub fn add_fragment(&mut self, fragment: Fragment) {
        let index = self
            .body
            .last_key_value()
            .map(|(&FragmentIndexInBranchBody(index), _)| index)
            .unwrap_or(0);

        self.body
            .insert(FragmentIndexInBranchBody(index + 1), fragment);
    }
}
