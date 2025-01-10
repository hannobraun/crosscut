use super::FragmentId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Body {
    pub inner: Vec<FragmentId>,
}

impl Body {
    pub fn ids(&self) -> impl Iterator<Item = &FragmentId> {
        self.inner.iter()
    }
}
