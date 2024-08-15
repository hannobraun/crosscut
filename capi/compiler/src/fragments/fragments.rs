use std::{collections::BTreeMap, iter};

use super::{
    Branch, Fragment, FragmentExpression, FragmentId, FragmentPayload, Function,
};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Fragments {
    /// The root fragment that indirectly points to all other fragments
    pub root: FragmentId,

    pub inner: FragmentMap,
}

impl Fragments {
    /// Find the function that contains the provided fragment
    ///
    /// Any fragment that is syntactically a part of the function body will do.
    /// This specifically includes fragments within blocks that are defined in
    /// the function.
    pub fn find_function_by_fragment_in_body(
        &self,
        fragment_id: &FragmentId,
    ) -> Option<(&Function, &Branch)> {
        let mut fragment_id = *fragment_id;

        loop {
            let previous = self
                .inner
                .inner
                .values()
                .find(|fragment| fragment.next() == Some(fragment_id));

            if let Some(previous) = previous {
                // There's a previous fragment. Continue the search there.
                fragment_id = previous.id();
                continue;
            }

            // If there's no previous fragment, this might be the first fragment
            // in a branch of a function.
            let function = self
                .inner
                .inner
                .values()
                .filter_map(|fragment| match &fragment.payload {
                    FragmentPayload::Expression {
                        expression: FragmentExpression::Function { function },
                        ..
                    } => Some((fragment.id(), function)),
                    _ => None,
                })
                .find_map(|(id, function)| {
                    let branch = function
                        .branches
                        .iter()
                        .find(|branch| branch.start == fragment_id)?;
                    Some((id, function, branch))
                });

            if let Some((id, function, branch)) = function {
                // We have found a function!

                if function.name.is_some() {
                    // It's a named function! Exactly what we've been looking
                    // for.
                    return Some((function, branch));
                } else {
                    // An anonymous function. Let's continue our search in the
                    // context where it was defined.
                    fragment_id = id;
                    continue;
                }
            }

            // If there's no previous fragment, nor a block where this is the
            // first fragment, it's probably the first fragment in the function
            // we're looking for.
            let function = self
                .inner
                .inner
                .values()
                .filter_map(|fragment| match &fragment.payload {
                    FragmentPayload::Function { function, .. } => {
                        Some(function)
                    }
                    _ => None,
                })
                .find_map(|cluster| {
                    let function = cluster
                        .branches
                        .iter()
                        .find(|function| function.start == fragment_id)?;
                    Some((cluster, function))
                });

            // And this is our result. If it's not the function we're looking
            // for, the fragment was part of the root context and there's no
            // function to be found.
            return function;
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct FragmentMap {
    pub inner: BTreeMap<FragmentId, Fragment>,
}

impl FragmentMap {
    pub fn find_function_by_name(&self, name: &str) -> Option<FragmentId> {
        self.inner
            .values()
            .filter_map(|fragment| match &fragment.payload {
                FragmentPayload::Function { function, .. } => {
                    Some((&function.name, fragment.id()))
                }
                _ => None,
            })
            .find_map(|(n, id)| {
                if n.as_deref() == Some(name) {
                    Some(id)
                } else {
                    None
                }
            })
    }

    pub fn iter_from(&self, id: FragmentId) -> impl Iterator<Item = &Fragment> {
        let mut next = Some(id);

        iter::from_fn(move || {
            let id = next.take()?;
            let fragment = self.inner.get(&id)?;

            next = fragment.next();

            Some(fragment)
        })
    }
}
