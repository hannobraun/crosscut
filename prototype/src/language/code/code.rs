use std::collections::BTreeMap;

use super::{
    Body, CodeError, Fragment, FragmentId, FragmentKind, FragmentPath,
    Fragments,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Code {
    fragments: Fragments,

    pub root: FragmentId,
    pub errors: BTreeMap<FragmentId, CodeError>,
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
            errors: BTreeMap::default(),
        }
    }

    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn find_innermost_fragment_with_valid_body(&self) -> FragmentPath {
        let mut next = self.root;
        let mut path = Vec::new();

        loop {
            let Some(body) = self.fragments.get(&next).valid_body() else {
                // The next fragment has no valid body. Which means the most
                // recent one we added is already is the innermost one!
                break;
            };

            path.push(next);

            let Some(id) = body.ids().next_back().copied() else {
                // The body we're currently looking at, is the innermost valid
                // one that we have found so far. If it doesn't have any
                // children, then it is the innermost valid one, period.
                //
                // If that's the case, we're done.
                break;
            };

            // We have found a nested fragment, but are only considering the
            // _last_ fragment in the body. In principle, we'd need to look at
            // _all_ of them.
            //
            // But as long as `Code` is only capable of pushing new fragments to
            // the end of the innermost body, I don't think it's possible to
            // construct a case where this makes a difference.

            next = id;
        }

        let Some(path) = FragmentPath::new(path) else {
            unreachable!(
                "It should be impossible to construct an invalid path here, as \
                the root fragment has a valid body. We _must_ have added it in \
                the loop above.",
            );
        };

        path
    }

    pub fn append(
        &mut self,
        to_append: Fragment,
        path: FragmentPath,
    ) -> FragmentId {
        let (to_update_id, path) = path.into_id_and_path();

        let mut to_update = self.fragments.get(&to_update_id).clone();
        let appended = to_update.body.push(to_append, &mut self.fragments);

        let mut id_before_update = to_update_id;
        let mut updated = to_update;

        for to_update_id in path {
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

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    FunctionCall { target: usize },
    LiteralInteger { value: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Token {
    Identifier { name: String },
    LiteralInteger { value: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
