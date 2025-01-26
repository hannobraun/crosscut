use std::collections::BTreeMap;

use crosscut_runtime::Instruction;

use crate::{
    code::{
        syntax::{FunctionLocation, SyntaxTree},
        Bindings, Changes, Dependencies, FunctionCalls, Functions, Recursion,
        TailExpressions, Types,
    },
    compiler::CallInstructionsByCallee,
    source_map::SourceMap,
    Instructions,
};

use super::compile_cluster::compile_cluster;

pub struct FunctionsContext<'r> {
    pub syntax_tree: &'r SyntaxTree,
    pub functions: &'r Functions,
    pub bindings: &'r Bindings,
    pub function_calls: &'r FunctionCalls,
    pub tail_expressions: &'r TailExpressions,
    pub recursion: &'r Recursion,
    pub instructions: &'r mut Instructions,
    pub source_map: &'r mut SourceMap,
    pub call_instructions_by_callee: &'r mut CallInstructionsByCallee,
    pub compiled_functions_by_location:
        &'r mut BTreeMap<FunctionLocation, crosscut_runtime::Function>,
}

#[allow(clippy::too_many_arguments)]
pub fn compile_functions(
    syntax_tree: &SyntaxTree,
    functions: &Functions,
    changes: &Changes,
    dependencies: &Dependencies,
    bindings: &Bindings,
    function_calls: &FunctionCalls,
    tail_expressions: &TailExpressions,
    _: &Types,
    recursion: &Recursion,
    instructions: &mut Instructions,
    source_map: &mut SourceMap,
    call_instructions_by_callee: &mut CallInstructionsByCallee,
    compiled_functions_by_location: &mut BTreeMap<
        FunctionLocation,
        crosscut_runtime::Function,
    >,
) {
    let mut context = FunctionsContext {
        syntax_tree,
        functions,
        bindings,
        function_calls,
        tail_expressions,
        recursion,
        instructions,
        source_map,
        call_instructions_by_callee,
        compiled_functions_by_location,
    };

    for cluster in dependencies.clusters() {
        compile_cluster(cluster, &mut context);
    }

    for update in &changes.updated {
        for calling_address in context
            .call_instructions_by_callee
            .inner
            .remove(&update.old.location)
            .unwrap_or_default()
        {
            let calling_instruction = context
                .instructions
                .get(&calling_address)
                .expect("Instruction referenced from source map must exist.");
            let Instruction::CallFunction { is_tail_call, .. } =
                calling_instruction
            else {
                panic!(
                    "Calling instruction referenced from source map is not a \
                    function call."
                );
            };

            let function = context
                .compiled_functions_by_location
                .get(&update.new.location)
                .expect(
                    "New function referenced in update should have been \
                    compiled; is expected to exist.",
                );

            context.instructions.replace(
                &calling_address,
                Instruction::CallFunction {
                    callee: function.clone(),
                    is_tail_call: *is_tail_call,
                },
            );
        }
    }
}
