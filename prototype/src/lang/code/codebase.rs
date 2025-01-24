use std::collections::BTreeMap;

use super::{
    Body, CodeError, Fragment, FragmentId, FragmentKind, Fragments, Located,
    Location, Replacements,
};

/// # The complete codebase of the program
///
/// Alternatively, the name can be seen as a contraction of "code database".
/// 
/// This is an append-only data structure. Old fragments are never removed. They
/// are just replaced by new ones, and become inaccessible via the root.
/// 
/// ## Implementation Note
/// 
/// This data structure only ever grows. There needs to be some kind of garbage
/// collection eventually. As well as some way to control what can be collected,
/// and what should be kept as history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    fragments: Fragments,
    root: FragmentId,
    replacements: Replacements,

    pub errors: BTreeMap<FragmentId, CodeError>,
}

impl Codebase {
    pub fn new() -> Self {
        let mut fragments = Fragments::default();

        let root = fragments.insert(Fragment {
            kind: FragmentKind::Root,
            body: Body::default(),
        });

        Self {
            fragments,
            root,
            replacements: Replacements::default(),
            errors: BTreeMap::new(),
        }
    }

    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn root(&self) -> Located {
        Located {
            location: Location::from_component(self.root),
            fragment: self.fragments.get(&self.root),
        }
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

    pub fn latest_version_of(&self, id: &FragmentId) -> FragmentId {
        self.replacements.latest_version_of(id)
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

            self.replacements
                .insert_original_and_replacement(*id, id_of_replacement);

            next_to_replace_with = parent;
            location_components_of_new_fragment_reverse.push(id_of_replacement);
        }

        self.root = self.fragments.insert(next_to_replace_with);

        Location::from_component(self.root).with_components(
            location_components_of_new_fragment_reverse
                .into_iter()
                .rev(),
        )
    }
}

impl Default for Codebase {
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

    use super::{Codebase, Expression, FunctionCallTarget};

    #[test]
    fn append_return_location() {
        let mut code = Codebase::new();

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
