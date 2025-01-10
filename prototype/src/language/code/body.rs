use super::{Fragment, FragmentId, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Body {
    pub inner: Vec<FragmentId>,
}

impl Body {
    pub fn push(
        &mut self,
        fragment: Fragment,
        fragments: &mut Fragments,
    ) -> FragmentId {
        let id = fragments.insert(fragment);
        self.inner.push(id);
        id
    }

    pub fn ids(&self) -> impl Iterator<Item = &FragmentId> {
        self.inner.iter()
    }

    pub fn fragments<'r>(
        &'r self,
        fragments: &'r Fragments,
    ) -> impl Iterator<Item = &'r Fragment> {
        self.ids().map(|hash| fragments.get(hash))
    }

    /// # Indicate whether this body is complete, i.e. contains an expression
    ///
    /// The presence of errors has no significance for the return value of this
    /// function. Its purpose, rather, is to indicate whether the addition of
    /// more fragments would also result in the addition of _more_ errors.
    pub fn is_complete(&self, fragments: &Fragments) -> bool {
        self.fragments(fragments)
            .any(|fragment| matches!(fragment, Fragment::Expression { .. }))
    }
}
