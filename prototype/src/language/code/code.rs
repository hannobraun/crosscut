use std::collections::BTreeSet;

use super::{Body, Fragment, FragmentId, FragmentKind, Fragments};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Code {
    fragments: Fragments,

    pub root: Body,
    pub errors: BTreeSet<FragmentId>,
}

impl Code {
    pub fn new() -> Self {
        Self {
            fragments: Fragments::default(),
            root: Body::default(),
            errors: BTreeSet::default(),
        }
    }

    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn find_innermost_fragment_with_valid_body(&self) -> FragmentPath {
        let mut path = FragmentPath { inner: Vec::new() };
        let mut current_body = &self.root;

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
        // This function is less regular than it could be, if the root where
        // another kind of fragment. Then it wouldn't need special handling
        // here.
        //
        // However, I think that would be problematic in different ways. Not the
        // least, by adding another kind of fragment that is only allowed to be
        // used in a single place.
        //
        // I'm not sure that it would be worth it, especially since I've already
        // got this working, it seems. It's something to keep an eye on though,
        // for sure.

        let Some(to_update_id) = path.inner.pop() else {
            return self.root.push(to_append, &mut self.fragments);
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

        self.root
            .replace(id_before_update, updated, &mut self.fragments);

        appended
    }
}

impl Default for Code {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FragmentPath {
    inner: Vec<FragmentId>,
}

impl FragmentPath {
    pub fn id(&self) -> Option<&FragmentId> {
        self.inner.last()
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
