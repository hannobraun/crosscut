use std::collections::BTreeSet;

use super::{Body, Fragment, FragmentId, FragmentKind, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Code {
    fragments: Fragments,

    pub root: Body,
    pub errors: BTreeSet<FragmentId>,
}

impl Code {
    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn find_innermost_fragment_with_valid_body(&self) -> FragmentPath {
        let mut path = FragmentPath { inner: Vec::new() };
        let mut body = &self.root;

        // Eventually, this method is going to take a parameter that tells it
        // exactly where to push the provided fragment. But for now, it just
        // always pushes it to the innermost valid expression.
        //
        // This loop is responsible for finding that.
        loop {
            let Some(id) = body.ids().next_back().copied() else {
                // The body we're currently looking at, `body`, is the innermost
                // valid one that we have found so far. If it doesn't have any
                // children, then it is the innermost valid one, period. We can
                // stop.
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
                // Our best candidate for the innermost valid body, `body`, does
                // have children, and we've been looking at the last of those.
                //
                // That child is not an expression though, which means it has no
                // valid body. We're done with our search.
                //
                // (In principle, we'd need to look at _all_ the children, to
                // see of any of them has a valid body. But as long as we're
                // just pushing new stuff to the end of the innermost body, I
                // don't think it's possible to construct a case where this
                // makes a difference.)
                break;
            };

            path.inner.push(id);
            body = &fragment.body;
        }

        path
    }

    pub fn append(&mut self, to_append: Fragment) -> FragmentId {
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

        let mut innermost_valid_body =
            self.find_innermost_fragment_with_valid_body();

        let Some(to_update_id) = innermost_valid_body.inner.pop() else {
            return self.root.push(to_append, &mut self.fragments);
        };

        let mut to_update = self.fragments.get(&to_update_id).clone();
        let appended = to_update.body.push(to_append, &mut self.fragments);

        let id_before_update = to_update_id;
        let updated = to_update;

        // We're missing code here that handles the rest of the fragments to
        // update. The test suite doesn't cover this yet, though, so I've
        // decided not to add it, for the time being.

        self.root
            .replace(id_before_update, updated, &mut self.fragments);

        appended
    }
}

pub struct FragmentPath {
    inner: Vec<FragmentId>,
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
