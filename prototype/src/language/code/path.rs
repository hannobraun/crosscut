use super::FragmentId;

#[derive(Debug)]
pub struct Cursor {
    inner: Vec<FragmentId>,
}

impl Cursor {
    pub fn new(inner: Vec<FragmentId>) -> Option<Self> {
        if inner.is_empty() {
            // An empty fragment path is not valid, as every path must at least
            // contain the root.
            None
        } else {
            Some(Self { inner })
        }
    }

    pub fn id(&self) -> &FragmentId {
        let Some(id) = self.inner.last() else {
            unreachable!(
                "A fragment path must consist of at least one component, the \
                root."
            );
        };
        id
    }

    pub fn into_id_and_path(
        mut self,
    ) -> (FragmentId, impl Iterator<Item = FragmentId>) {
        let Some(id) = self.inner.pop() else {
            unreachable!(
                "A fragment path must consist of at least one component, the \
                root."
            );
        };

        (id, self.inner.into_iter().rev())
    }
}
