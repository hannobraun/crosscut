use std::{collections::BTreeMap, iter};

use super::{FunctionIndexInCluster, FunctionIndexInRootContext};

/// # The program's named functions, organized as a call graph
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct CallGraph {
    clusters: Vec<Cluster>,
}

impl CallGraph {
    /// # Iterate over all named functions, from the leaves up
    ///
    /// Guarantees that any function that is yielded by the iterator only has
    /// non-recursive calls to functions that have already been yielded before.
    pub fn functions_from_leaves(
        &self,
    ) -> impl Iterator<Item = (&FunctionIndexInRootContext, &Cluster)> {
        self.clusters.iter().rev().flat_map(|cluster| {
            cluster.functions.values().zip(iter::repeat(cluster))
        })
    }

    /// # Find the cluster containing a given function
    ///
    /// ## Panics
    ///
    /// Panics, if the provided location does not refer to a named function.
    pub fn find_cluster_by_named_function(
        &self,
        index: &FunctionIndexInRootContext,
    ) -> Option<&Cluster> {
        self.clusters
            .iter()
            .find(|cluster| cluster.functions.values().any(|i| i == index))
    }

    /// # Iterate over the function clusters
    pub fn clusters(&self) -> impl Iterator<Item = &Cluster> {
        self.clusters.iter()
    }
}

impl FromIterator<Cluster> for CallGraph {
    fn from_iter<T: IntoIterator<Item = Cluster>>(clusters: T) -> Self {
        Self {
            clusters: clusters.into_iter().collect(),
        }
    }
}

/// # A cluster of functions
///
/// During compilation, all functions are grouped into clusters. A cluster can
/// consist of a single function, or a group of mutually recursive functions.
///
/// All mutually recursive functions are grouped into a single clusters with the
/// other functions in their recursive group.
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
pub struct Cluster {
    /// # Indices that refer to the functions in the cluster
    ///
    /// The indices refer to the functions in their original order within the
    /// list of all named functions.
    pub functions: BTreeMap<FunctionIndexInCluster, FunctionIndexInRootContext>,
}