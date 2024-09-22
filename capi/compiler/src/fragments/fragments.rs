use std::{
    collections::BTreeMap,
    iter,
    ops::{Deref, DerefMut},
};

use super::{Branch, Fragment, FragmentKind, Function, Hash};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Fragments {
    /// The root fragment that indirectly points to all other fragments
    pub root: Hash<Fragment>,

    pub inner: FragmentMap,
}

impl Deref for Fragments {
    type Target = FragmentMap;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Fragments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct FragmentMap {
    inner: BTreeMap<Hash<Fragment>, Fragment>,
}

impl FragmentMap {
    pub fn insert(&mut self, fragment: Fragment) {
        self.inner.insert(fragment.hash(), fragment);
    }

    pub fn remove(&mut self, hash: &Hash<Fragment>) -> Option<Fragment> {
        self.inner.remove(hash)
    }

    pub fn get(&self, hash: &Hash<Fragment>) -> Option<&Fragment> {
        self.inner.get(hash)
    }

    pub fn find_function_by_name(&self, name: &str) -> Option<FoundFunction> {
        self.inner
            .values()
            .filter_map(|fragment| match &fragment.kind {
                FragmentKind::Function { function } => {
                    Some((fragment.hash(), function))
                }
                _ => None,
            })
            .find_map(|(id, function)| {
                if function.name.as_deref() == Some(name) {
                    Some(FoundFunction { hash: id, function })
                } else {
                    None
                }
            })
    }

    /// Find the named function that contains the provided fragment
    ///
    /// Any fragment that is syntactically a part of the named function will do.
    /// This specifically includes fragments within anonymous functions that are
    /// defined in the named function.
    ///
    /// Returns the found function, as well as the branch within which the
    /// fragment was found.
    pub fn find_named_function_by_fragment_in_body(
        &self,
        fragment_in_body: &Hash<Fragment>,
    ) -> Option<(FoundFunction, &Branch)> {
        let mut current_fragment = *fragment_in_body;

        loop {
            let previous = self.inner.values().find(|fragment| {
                fragment.location.next == Some(current_fragment)
            });

            if let Some(previous) = previous {
                // There's a previous fragment. Continue the search there.
                current_fragment = previous.hash();
                continue;
            }

            // If there's no previous fragment, this might be the first fragment
            // in a branch of a function.
            let function = self
                .inner
                .values()
                .filter_map(|fragment| match &fragment.kind {
                    FragmentKind::Function { function } => {
                        Some((fragment.hash(), function))
                    }
                    _ => None,
                })
                .find_map(|(hash, function)| {
                    let branch = function
                        .branches
                        .iter()
                        .find(|branch| branch.start == current_fragment)?;
                    Some((hash, function, branch))
                });

            if let Some((hash, function, branch)) = function {
                // We have found a function!

                if function.name.is_some() {
                    // It's a named function! Exactly what we've been looking
                    // for.
                    return Some((FoundFunction { hash, function }, branch));
                } else {
                    // An anonymous function. Let's continue our search in the
                    // context where it was defined.
                    current_fragment = hash;
                    continue;
                }
            }

            // We haven't found anything. Not even a new fragment to look at.
            // We're done here.
            break None;
        }
    }

    pub fn iter_from(
        &self,
        start: Hash<Fragment>,
    ) -> impl Iterator<Item = &Fragment> {
        let mut next = Some(start);

        iter::from_fn(move || {
            let id = next.take()?;
            let fragment = self.inner.get(&id)?;

            next = fragment.location.next;

            Some(fragment)
        })
    }
}

/// # Return type of several methods that search for functions
///
/// This type bundles the found function and its ID. It [`Deref`]s to
/// `Function`.
#[derive(Debug)]
pub struct FoundFunction<'r> {
    pub hash: Hash<Fragment>,
    pub function: &'r Function,
}

impl Deref for FoundFunction<'_> {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        self.function
    }
}
