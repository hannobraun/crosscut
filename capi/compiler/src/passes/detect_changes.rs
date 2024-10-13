use std::collections::BTreeMap;

use crate::{
    fragments::{
        Changes, Fragments, FunctionInUpdate, FunctionUpdate, NamedFunctions,
    },
    hash::Hash,
};

pub fn detect_changes(old: Option<NamedFunctions>, new: &Fragments) -> Changes {
    let mut old = old
        .map(|old_functions| old_functions.inner)
        .unwrap_or_default();
    let mut new = new.named_functions.inner.clone();

    let mut added = BTreeMap::new();
    let mut updated = Vec::new();

    while let Some((new_index, new_function)) = new.pop_first() {
        let same_hash = old.iter().find_map(|(&index, old_function)| {
            if Hash::new(old_function) == Hash::new(&new_function) {
                Some(index)
            } else {
                None
            }
        });
        if same_hash.is_some() {
            // Function has not changed. We can forget about it.
            continue;
        }

        let same_name = old.iter().find_map(|(&index, old_function)| {
            assert!(
                old_function.name.is_some(),
                "Named function should have a name."
            );
            assert!(
                new_function.name.is_some(),
                "Named function should have a name."
            );

            if old_function.name == new_function.name {
                Some(index)
            } else {
                None
            }
        });
        if let Some(old_index) = same_name {
            // Found a function with the same name. But it can't have the same
            // hash, or we wouldn't have made it here. Assuming the new function
            // is an updated version of the old.
            let old_function = old.remove(&old_index).expect(
                "Just found index in map; expecting it to still be there.",
            );
            updated.push(FunctionUpdate {
                old: FunctionInUpdate {
                    index: old_index,
                    function: old_function,
                },
                new: FunctionInUpdate {
                    index: new_index,
                    function: new_function,
                },
            });

            continue;
        }

        // If we make it here, there was neither an identical function before,
        // nor one with the same name. This must mean this function is new.
        added.insert(new_index, new_function);
    }

    Changes { added, updated }
}
