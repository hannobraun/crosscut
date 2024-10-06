use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use super::{FragmentId, FragmentMap, FragmentsByLocation};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Fragments {
    /// # The first fragment in the root context
    ///
    /// This indirectly points to all existing fragments.
    pub root: Option<FragmentId>,

    /// # The function clusters
    pub clusters: Vec<Cluster>,

    pub map: FragmentMap,

    /// # All code fragments, tracked by their location
    ///
    /// ## Implementation Note
    ///
    /// This is a placeholder. The refactoring to fill it out is currently in
    /// progress.
    pub by_location: FragmentsByLocation,
}

impl Fragments {
    /// # Find a cluster, by the ID of a function within it
    ///
    /// Returns `None`, if the provided `FragmentId` does not refer to a named
    /// function. All named functions are grouped into clusters, so this should
    /// always return `Some` otherwise.
    pub fn find_cluster_by_function_id(
        &self,
        function_id: &FragmentId,
    ) -> Option<&Cluster> {
        self.clusters.iter().find(|cluster| {
            cluster.functions.values().any(|id| id == function_id)
        })
    }
}

impl Deref for Fragments {
    type Target = FragmentMap;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Fragments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Cluster {
    pub functions: BTreeMap<FunctionIndexInCluster, FragmentId>,
}

/// # An index into the list of functions in a cluster
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct FunctionIndexInCluster(pub u32);
