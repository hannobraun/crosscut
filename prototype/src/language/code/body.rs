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

    /// # Indicate whether this is complete, meaning contains an expression
    ///
    /// The presence of errors has no significance for the return value of this
    /// function. Its purpose, rather, is to indicate whether the addition of
    /// more fragments would also result in the addition of _more_ errors.
    ///
    /// ## Implementation Note
    ///
    /// Eventually, this method would move to a type representing something like
    /// a branch, or branch body. That it is defined on `Code` is only a
    /// consequence of the current state of development.
    pub fn is_complete(&self, fragments: &Fragments) -> bool {
        self.fragments(fragments)
            .any(|fragment| matches!(fragment, Fragment::Expression { .. }))
    }
}
