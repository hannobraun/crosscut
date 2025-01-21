use itertools::Itertools;

use super::{Fragment, FragmentId, Fragments};

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
    inner: Vec<FragmentId>,
}

impl Location {
    pub fn from_component(component: FragmentId) -> Self {
        Self {
            inner: vec![component],
        }
    }

    pub fn from_components(inner: Vec<FragmentId>) -> Option<Self> {
        if inner.is_empty() {
            // An empty fragment path is not valid, as every path must at least
            // contain the root.
            None
        } else {
            Some(Self { inner })
        }
    }

    pub fn with_component(mut self, component: FragmentId) -> Self {
        self.inner.push(component);
        self
    }

    pub fn with_components(
        mut self,
        components: impl IntoIterator<Item = FragmentId>,
    ) -> Self {
        self.inner.extend(components);
        self
    }

    pub fn target(&self) -> &FragmentId {
        let Some(target) = self.inner.last() else {
            unreachable!(
                "A fragment path must consist of at least one component, the \
                root."
            );
        };

        target
    }

    pub fn parent(&self) -> Option<&FragmentId> {
        self.inner.iter().rev().nth(1)
    }

    pub fn components_with_parent(
        &self,
    ) -> impl Iterator<Item = (&FragmentId, &FragmentId)> {
        self.inner.iter().rev().tuple_windows()
    }
}

pub struct Located<'r> {
    pub location: Location,
    pub fragment: &'r Fragment,
}

impl<'r> Located<'r> {
    pub fn body(
        &'r self,
        fragments: &'r Fragments,
    ) -> impl Iterator<Item = Located<'r>> {
        self.fragment.body.ids().map(|id| {
            let location = self.location.clone().with_component(*id);
            let fragment = fragments.get(id);

            Located { location, fragment }
        })
    }
}
