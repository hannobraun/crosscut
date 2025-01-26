use anyhow::anyhow;
use crosscut_compiler::{
    code::{
        syntax::{
            self, BranchLocation, FunctionLocation, Located, MemberLocation,
        },
        DependencyCluster, FunctionCalls, Functions, Signature, Types,
    },
    source_map::SourceMap,
};
use crosscut_runtime::Effect;

use super::{Breakpoints, DebugBranch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugNamedFunction {
    pub name: String,
    pub inner: DebugFunction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugFunction {
    pub branches: Vec<DebugBranch>,
    pub signature: Option<Signature>,
}

impl DebugFunction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        function: syntax::Function,
        location: FunctionLocation,
        active_expression: Option<&MemberLocation>,
        is_innermost_active_function: bool,
        cluster: &DependencyCluster,
        functions: &Functions,
        function_calls: &FunctionCalls,
        types: &Types,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effect: Option<&Effect>,
    ) -> Self {
        let branches = function
            .branches
            .into_iter()
            .map(|(index, branch)| {
                DebugBranch::new(
                    Located {
                        fragment: branch,
                        location: BranchLocation {
                            parent: Box::new(location.clone()),
                            index,
                        },
                    },
                    active_expression,
                    is_innermost_active_function,
                    cluster,
                    functions,
                    function_calls,
                    types,
                    source_map,
                    breakpoints,
                    effect,
                )
            })
            .collect();

        let signature = types.signature_of_function(&location).cloned();

        Self {
            branches,
            signature,
        }
    }

    pub fn active_branch(&self) -> anyhow::Result<&DebugBranch> {
        self.branches
            .iter()
            .find(|branch| branch.is_active)
            .ok_or_else(|| {
                anyhow!(
                    "Expected to find active branch in function, but none is."
                )
            })
    }
}
