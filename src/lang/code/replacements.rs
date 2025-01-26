use std::collections::{BTreeMap, BTreeSet};

use super::NodeId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Replacements {
    inner: BTreeMap<NodeId, NodeId>,
}

impl Replacements {
    pub fn insert_original_and_replacement(
        &mut self,
        original: NodeId,
        replacement: NodeId,
    ) {
        // This check is not sufficient, as it only catches a special case of a
        // larger problem: It is quite easy to construct replacement loops,
        // which could then result in the panic below (in `latest_version_of`)
        // triggering.
        //
        // Here's an example scenario for how to construct such a loop:
        //
        // 1. Create a new node and type '2'. We record `2` as replacing the
        //    empty fragment.
        // 2. Continue editing the node and type '5'. We record `25` as
        //    replacing `2`.
        // 3. Continue editing the node by pressing backspace. We record `2` as
        //    replacing `25`, closing the loop.
        //
        // But while it's easy to construct a loop like this, it is not
        // straight-forward, maybe impossible, to then trigger the panic below.
        // Because `latest_version_of` will only get called, if the interpreter
        // is running while any of that happens. And due to the limited nature
        // of the current setup, that's generally not how it happens.
        if original == replacement {
            // It seems like this check should maybe be an assertion instead,
            // but it's actually quite easy to get into a situation where a
            // token gets compiled unchanged, which will lead to this case. And
            // there might actually not be an easier place to catch than, than
            // here.
            return;
        }

        self.inner.insert(original, replacement);
    }

    pub fn latest_version_of(&self, id: &NodeId) -> NodeId {
        let mut already_seen = BTreeSet::new();
        let mut current_id = id;

        while let Some(replacement) = self.inner.get(current_id) {
            if already_seen.contains(replacement) {
                unreachable!(
                    "Detected endless loop while searching for latest version \
                    of {id:?}. IDs found: {already_seen:?}"
                );
            }

            already_seen.insert(id);
            current_id = replacement;
        }

        *current_id
    }
}
