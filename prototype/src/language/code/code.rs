use std::collections::BTreeSet;

use super::{Body, Fragment, FragmentId, FragmentKind, Fragments};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Code {
    fragments: Fragments,

    pub root: FragmentId,
    pub errors: BTreeSet<FragmentId>,
}

impl Code {
    pub fn new() -> Self {
        let mut fragments = Fragments::default();
        let root = fragments.insert(Fragment {
            kind: FragmentKind::Root,
            body: Body::default(),
        });

        Self {
            fragments,
            root,
            errors: BTreeSet::default(),
        }
    }

    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn find_innermost_fragment_with_valid_body(&self) -> FragmentPath {
        let mut path = FragmentPath {
            inner: vec![self.root],
        };
        let mut current_body = &self.fragments().get(&self.root).body;

        loop {
            let Some(id) = current_body.ids().next_back().copied() else {
                // The body we're currently looking at, is the innermost valid
                // one that we have found so far. If it doesn't have any
                // children, then it is the innermost valid one, period.
                //
                // If that's the case, we're done.
                break;
            };

            let fragment @ Fragment {
                kind:
                    FragmentKind::Expression {
                        expression: Expression::FunctionCall { .. },
                    },
                ..
            } = self.fragments.get(&id)
            else {
                // The body we're currently looking at does have children, and
                // we've been looking at the last of those. That child is not an
                // expression though, which means it has no valid body. We're
                // done with our search.
                //
                // (In principle, we'd need to look at _all_ the children, to
                // see of any of them has a valid body. But as long as we're
                // just pushing new stuff to the end of the innermost body, I
                // don't think it's possible to construct a case where this
                // makes a difference.)
                break;
            };

            path.inner.push(id);
            current_body = &fragment.body;
        }

        path
    }

    pub fn append(
        &mut self,
        to_append: Fragment,
        mut path: FragmentPath,
    ) -> FragmentId {
        let Some(to_update_id) = path.inner.pop() else {
            unreachable!(
                "A fragment path must consist of at least one component, the \
                root. This one doesn't: `{path:#?}`"
            );
        };

        let mut to_update = self.fragments.get(&to_update_id).clone();
        let appended = to_update.body.push(to_append, &mut self.fragments);

        let mut id_before_update = to_update_id;
        let mut updated = to_update;

        for to_update_id in path.inner.into_iter().rev() {
            let mut to_update = self.fragments.get(&to_update_id).clone();
            to_update.body.replace(
                id_before_update,
                updated,
                &mut self.fragments,
            );

            id_before_update = to_update_id;
            updated = to_update;
        }

        self.root = self.fragments.insert(updated);

        appended
    }
}

impl Default for Code {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct FragmentPath {
    inner: Vec<FragmentId>,
}

impl FragmentPath {
    pub fn id(&self) -> &FragmentId {
        let Some(id) = self.inner.last() else {
            unreachable!(
                "A fragment path must consist of at least one component, the \
                root."
            );
        };
        id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    FunctionCall { target: usize },
    LiteralValue { value: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
