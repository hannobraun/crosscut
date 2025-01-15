use std::collections::BTreeMap;

use super::{
    Body, CodeError, Fragment, FragmentId, FragmentKind, Fragments, Location,
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

    pub fn find_innermost_fragment_with_valid_body(&self) -> Location {
        let mut next = self.root;
        let mut location = Vec::new();

        loop {
            let Some(body) = self.fragments.get(&next).valid_body() else {
                // The next fragment has no valid body. Which means the most
                // recent one we added is already is the innermost one!
                break;
            };

            location.push(next);

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

        let Some(location) = Location::new(location) else {
            unreachable!(
                "It should be impossible to construct an invalid path here, as \
                the root fragment has a valid body. We _must_ have added it in \
                the loop above.",
            );
        };

        location
    }

    pub fn append_to_body_at(
        &mut self,
        cursor: Location,
        to_append: Fragment,
    ) -> FragmentId {
        let (to_update_id, parent_ids) = cursor.into_target_and_parents();

        let mut to_update = self.fragments.get(&to_update_id).clone();
        let appended = to_update.body.push(to_append, &mut self.fragments);

        let mut id_before_update = to_update_id;
        let mut updated = to_update;

        for to_update_id in parent_ids {
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
    Literal { literal: Literal },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Literal {
    Integer { value: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
