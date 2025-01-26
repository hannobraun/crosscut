use std::collections::BTreeMap;

use crosscut_runtime::InstructionAddress;

use crate::code::{syntax::FunctionLocation, DependencyCluster};

use super::{
    compile_function::{
        compile_call_to_function, compile_definition_of_local_function,
        compile_function, CallToFunction,
    },
    compile_functions::FunctionsContext,
};

pub struct ClusterContext {
    /// # Recursive calls within the cluster that need to be replaced
    ///
    /// When a recursive call is encountered, not all branches of the callee
    /// (which might be the calling function itself, or another function in the
    /// same cluster) might be compiled yet. But they're needed to compile the
    /// call.
    ///
    /// So instead of compiling the call right then and there, a placeholder
    /// instruction is emitted. An entry is also added to this map, so the
    /// placeholder instruction can be replaced with the real call, once all
    /// functions have been compiled.
    pub recursive_calls_by_callee:
        BTreeMap<FunctionLocation, Vec<CallToFunction>>,

    /// # Recursive local functions within the cluster that need to be replaced
    ///
    /// When a recursive local function is encountered, it might not have been
    /// compiled yet. To deal with this, we generate a placeholder that needs to
    /// be replaced later. This map tracks those necessary replacements.
    pub recursive_local_function_definitions_by_local_function:
        BTreeMap<FunctionLocation, InstructionAddress>,
}

pub fn compile_cluster(
    cluster: &DependencyCluster,
    functions_context: &mut FunctionsContext,
) {
    let mut context = ClusterContext {
        recursive_calls_by_callee: BTreeMap::new(),
        recursive_local_function_definitions_by_local_function: BTreeMap::new(),
    };

    for function in cluster.functions(functions_context.syntax_tree) {
        let location = function.location.clone();

        let runtime_function =
            compile_function(function, &mut context, functions_context);

        functions_context
            .compiled_functions_by_location
            .insert(location, runtime_function);
    }

    for (callee, calls) in context.recursive_calls_by_callee {
        for call in calls {
            compile_call_to_function(
                &callee,
                call,
                functions_context.compiled_functions_by_location,
                functions_context.instructions,
            );
        }
    }
    for (local_function, address) in
        context.recursive_local_function_definitions_by_local_function
    {
        compile_definition_of_local_function(
            local_function,
            address,
            functions_context,
        );
    }
}
