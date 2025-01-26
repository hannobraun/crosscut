use std::collections::BTreeMap;

use crate::code::{
    syntax::SyntaxTree, Changes, FunctionInUpdate, FunctionUpdate, Hash,
};

pub fn detect_changes(
    old_code: Option<SyntaxTree>,
    new_code: &SyntaxTree,
) -> Changes {
    let old_code = old_code.unwrap_or_default();

    let mut added = BTreeMap::new();
    let mut updated = Vec::new();

    for new_function in new_code.named_functions() {
        if old_code.named_functions.values().any(|old_function| {
            Hash::new(&old_function.inner)
                == Hash::new(&new_function.fragment.inner)
        }) {
            // Function has not changed. We can forget about it.
            continue;
        }

        if let Some(old_function) =
            old_code.function_by_name(&new_function.name)
        {
            // Found a function with the same name. But it can't have the same
            // hash, or we wouldn't have made it here. Assuming the new function
            // is an updated version of the old.
            updated.push(FunctionUpdate {
                old: FunctionInUpdate {
                    location: old_function.location(),
                    function: old_function.inner.clone(),
                },
                new: FunctionInUpdate {
                    location: new_function.location(),
                    function: new_function.inner.clone(),
                },
            });

            continue;
        }

        // If we make it here, there was neither an identical function before,
        // nor one with the same name. This must mean this function is new.
        added.insert(new_function.location(), new_function.inner.clone());
    }

    Changes { added, updated }
}
