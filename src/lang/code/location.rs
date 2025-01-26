use itertools::Itertools;

use super::{Node, NodeId, Nodes};

/// # The unique location of a fragment
///
/// This is distinct from [`FragmentId`], which could can identify multiple
/// identical fragments at different locations in the code.
///
/// ## Implementation Note
///
/// The uniqueness that the text above promises is actually not guaranteed right
/// now. However, with the limited means available, it should be impossible to
/// construct a situation where that matters.
///
/// In any case, this can be fixed by attaching the index of the fragment within
/// its parent's body to each component of the cursor. I intend to do so, as
/// soon as it's possible to write a test that covers this.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location {
    inner: Vec<NodeId>,
}

impl Location {
    pub fn from_component(component: NodeId) -> Self {
        Self {
            inner: vec![component],
        }
    }

    pub fn from_components(inner: Vec<NodeId>) -> Option<Self> {
        if inner.is_empty() {
            // An empty fragment path is not valid, as every path must at least
            // contain the root.
            None
        } else {
            Some(Self { inner })
        }
    }

    pub fn with_component(mut self, component: NodeId) -> Self {
        self.inner.push(component);
        self
    }

    pub fn with_components(
        mut self,
        components: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        self.inner.extend(components);
        self
    }

    pub fn target(&self) -> &NodeId {
        let Some(target) = self.inner.last() else {
            unreachable!(
                "A fragment path must consist of at least one component, the \
                root."
            );
        };

        target
    }

    pub fn parent(&self) -> Option<&NodeId> {
        self.inner.iter().rev().nth(1)
    }

    pub fn components_with_parent(
        &self,
    ) -> impl Iterator<Item = (&NodeId, &NodeId)> {
        self.inner.iter().rev().tuple_windows()
    }
}

pub struct Located<'r> {
    pub location: Location,
    pub node: &'r Node,
}

impl<'r> Located<'r> {
    pub fn body(
        &'r self,
        nodes: &'r Nodes,
    ) -> impl Iterator<Item = Located<'r>> {
        self.node.body.ids().map(|id| {
            let location = self.location.clone().with_component(*id);
            let node = nodes.get(id);

            Located { location, node }
        })
    }
}
