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

        let Some(location) = Location::from_components(location) else {
            unreachable!(
                "It should be impossible to construct an invalid path here, as \
                the root fragment has a valid body. We _must_ have added it in \
                the loop above.",
            );
        };

        location
    }

    pub fn append_to(
        &mut self,
        location: &Location,
        to_append: Fragment,
    ) -> Location {
        // Append the new fragment where we're supposed to append it.
        let mut append_to = self.fragments.get(location.target()).clone();
        let appended =
            append_to.body.push_fragment(to_append, &mut self.fragments);

        // And now, update all of its parents, down to the root.
        let location = self.replace(location, append_to);

        location.with_component(appended)
    }

    pub fn replace(
        &mut self,
        location: &Location,
        replace_with: Fragment,
    ) -> Location {
        let mut next_to_replace_with = replace_with;
        let mut location_components_of_new_fragment_reverse = Vec::new();

        for (id, parent) in location.components_with_parent() {
            let mut parent = self.fragments.get(parent).clone();
            let id_of_replacement = parent.body.replace(
                id,
                next_to_replace_with,
                &mut self.fragments,
            );

            next_to_replace_with = parent;
            location_components_of_new_fragment_reverse.push(id_of_replacement);
        }

        self.root = self.fragments.insert(next_to_replace_with);
        let location = Location::from_component(self.root);

        location.with_components(
            location_components_of_new_fragment_reverse
                .into_iter()
                .rev(),
        )
    }
}

impl Default for Code {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    FunctionCall { target: FunctionCallTarget },
    Literal { literal: Literal },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum FunctionCallTarget {
    HostFunction { id: usize },
    IntrinsicFunction,
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Literal {
    Integer { value: u32 },
}

#[cfg(test)]
mod tests {
    use crate::lang::code::{Body, Fragment, FragmentKind, Location};

    use super::{Code, Expression, FunctionCallTarget};

    #[test]
    fn append_return_location() {
        let mut code = Code::new();

        let a = call(0);
        let b = call(1);

        assert_eq!(
            code.append_to(
                &code.find_innermost_fragment_with_valid_body(),
                a.clone(),
            ),
            Location::from_component(code.root).with_component(a.id()),
        );

        assert_eq!(
            code.append_to(
                &code.find_innermost_fragment_with_valid_body(),
                b.clone(),
            ),
            Location::from_component(code.root)
                .with_component(a.with_child(b.id()).id())
                .with_component(b.id()),
        );
    }

    fn call(id: usize) -> Fragment {
        Fragment {
            kind: FragmentKind::Expression {
                expression: Expression::FunctionCall {
                    target: FunctionCallTarget::HostFunction { id },
                },
            },
            body: Body::default(),
        }
    }
}
