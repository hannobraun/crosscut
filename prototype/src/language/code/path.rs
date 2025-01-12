use super::FragmentId;

/// # A unique identifier for a fragment at a specific location
///
/// This is distinct from [`FragmentId`], which could can identify multiple
/// identical fragments at different locations in the syntax tree.
///
/// ## Implementation Note
///
/// The uniqueness that the text above promises is actually not guaranteed right
/// now. However, it with the limited means available, it should be impossible
/// to construct a situation where that matters.
///
/// In any case, this can be fixed by attaching the index of the fragment within
/// its parent's body to each element of the cursor. I intend to do so, as soon
/// as it's possible to write a test that covers this.
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
