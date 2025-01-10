use super::{Fragment, FragmentId, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Body {
    pub inner: Vec<FragmentId>,
}

impl Body {
    pub fn ids(&self) -> impl Iterator<Item = &FragmentId> {
        self.inner.iter()
    }

    pub fn fragments<'r>(
        &'r self,
        fragments: &'r Fragments,
    ) -> impl Iterator<Item = &'r Fragment> {
        self.ids().map(|hash| fragments.get(hash))
    }
}
