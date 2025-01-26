use std::{collections::VecDeque, fmt};

use anyhow::anyhow;
use crosscut_compiler::{
    code::{
        syntax::{FunctionLocation, MemberLocation, NamedFunction},
        Index,
    },
    CompilerOutput,
};
use crosscut_protocol::host_state::HostState;
use crosscut_runtime::{Effect, InstructionAddress};

use super::{
    Breakpoints, DebugBranch, DebugFunction, DebugMember, DebugNamedFunction,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveFunctions {
    Entries { entries: ActiveFunctionsEntries },
    Message { message: ActiveFunctionsMessage },
}

impl ActiveFunctions {
    pub fn new(
        code: Option<&CompilerOutput>,
        breakpoints: &Breakpoints,
        state: Option<&HostState>,
    ) -> Self {
        let Some(code) = code else {
            return Self::Message {
                message: ActiveFunctionsMessage::NoServer,
            };
        };
        let (effects, active_instructions) = match state {
            Some(state) => match state {
                HostState::Running => {
                    return Self::Message {
                        message: ActiveFunctionsMessage::ProcessRunning,
                    };
                }
                HostState::Finished => {
                    return Self::Message {
                        message: ActiveFunctionsMessage::ProcessFinished,
                    };
                }
                HostState::Stopped {
                    effect: effects,
                    active_instructions,
                    current_operands: _,
                } => (effects, active_instructions),
            },
            None => {
                return Self::Message {
                    message: ActiveFunctionsMessage::NoProcess,
                };
            }
        };

        let mut active_instructions: VecDeque<InstructionAddress> =
            active_instructions.clone().into();

        let mut entries = VecDeque::new();
        let mut expected_next_function = Some("main".to_string());

        if let Some(outer) = active_instructions.front() {
            let (outer, _) = instruction_to_named_function(outer, code);
            if Some(outer.name) != expected_next_function {
                expected_next_function = reconstruct_function(
                    "main",
                    &mut entries,
                    code,
                    breakpoints,
                    effects.as_ref(),
                );
            }
        }

        while let Some(address) = active_instructions.pop_front() {
            let (named_function, function_index_in_root_context) =
                instruction_to_named_function(&address, code);
            let active_expression =
                code.source_map.instruction_to_expression(&address);

            if let Some(expected_name) = &expected_next_function {
                if expected_name != &named_function.name {
                    reconstruct_function(
                        expected_name,
                        &mut entries,
                        code,
                        breakpoints,
                        effects.as_ref(),
                    );
                }
            } else {
                entries.push_front(ActiveFunctionsEntry::Gap);
            }

            expected_next_function =
                active_expression.and_then(|active_expression| {
                    function_call_to_function_name(active_expression, code)
                });

            let cluster = code
                .dependencies
                .find_cluster_by_named_function(&function_index_in_root_context)
                .expect("All named functions must be part of a cluster.");
            entries.push_front(ActiveFunctionsEntry::Function(
                DebugNamedFunction {
                    name: named_function.name,
                    inner: DebugFunction::new(
                        named_function.inner,
                        FunctionLocation::Named {
                            index: function_index_in_root_context,
                        },
                        active_expression,
                        active_instructions.is_empty(),
                        cluster,
                        &code.functions,
                        &code.function_calls,
                        &code.types,
                        &code.source_map,
                        breakpoints,
                        effects.as_ref(),
                    ),
                },
            ));
        }

        Self::Entries {
            entries: ActiveFunctionsEntries {
                inner: entries.into(),
            },
        }
    }

    pub fn entries(&self) -> anyhow::Result<&ActiveFunctionsEntries> {
        let ActiveFunctions::Entries { entries } = &self else {
            return Err(anyhow!(
                "Active function entries not available. Available state:\n\
                {self:#?}"
            ));
        };

        Ok(entries)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveFunctionsEntries {
    pub inner: Vec<ActiveFunctionsEntry>,
}

impl ActiveFunctionsEntries {
    pub fn leaf(&self) -> &ActiveFunctionsEntry {
        self.inner.first().expect(
            "Empty active function entries should never get constructed. At \
            the very least, the leaf function should be present. If that is \
            not `main`, the `main` function should be present (possibly \
            reconstructed) too.",
        )
    }

    pub fn find_next_expression_or_next_after_caller(
        &self,
        branch: &DebugBranch,
        expression: &MemberLocation,
    ) -> anyhow::Result<Option<DebugMember>> {
        if let Some(after) = branch.expression_after(expression)? {
            return Ok(Some(after.clone()));
        }

        self.find_next_expression_after_caller(expression)
    }

    pub fn find_next_expression_after_caller(
        &self,
        expression: &MemberLocation,
    ) -> anyhow::Result<Option<DebugMember>> {
        let caller_branch = self
            .inner
            .iter()
            .filter_map(|entry| match entry {
                ActiveFunctionsEntry::Function(function) => Some(function),
                ActiveFunctionsEntry::Gap => None,
            })
            .filter_map(|function| match function.inner.active_branch() {
                Ok(branch) => Some(branch),
                Err(_) => None,
            })
            .find(|branch| {
                !branch.body.iter().any(|f| f.data.location == *expression)
            });

        let Some(caller_branch) = caller_branch else {
            return Ok(None);
        };

        let caller = caller_branch.active_expression()?;

        self.find_next_expression_or_next_after_caller(
            caller_branch,
            &caller.data.location,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveFunctionsEntry {
    Function(DebugNamedFunction),
    Gap,
}

impl ActiveFunctionsEntry {
    pub fn function(&self) -> anyhow::Result<&DebugFunction> {
        let Self::Function(function) = self else {
            return Err(anyhow!(
                "Expected active functions entry to be function. Got \
                instead:\n\
                {self:#?}"
            ));
        };

        Ok(&function.inner)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveFunctionsMessage {
    NoServer,
    NoProcess,
    ProcessRunning,
    ProcessFinished,
}

impl fmt::Display for ActiveFunctionsMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoServer => {
                write!(f, "No connection to server.")?;
            }
            Self::NoProcess => {
                write!(f, "No connection to process.")?;
            }
            Self::ProcessRunning => {
                write!(f, "Process is running.")?;
            }
            Self::ProcessFinished => {
                write!(f, "Process is finished.")?;
            }
        }

        Ok(())
    }
}

fn instruction_to_named_function(
    address: &InstructionAddress,
    code: &CompilerOutput,
) -> (NamedFunction, Index<NamedFunction>) {
    let location = code.source_map.instruction_to_function(address).expect(
        "Expecting instructions on call stack to all map to a function.",
    );

    let mut current_location = location.clone();

    loop {
        match current_location {
            FunctionLocation::Named { index } => {
                let function = code
                    .syntax_tree
                    .named_functions
                    .get(&index)
                    .expect(
                        "Function location in source map should refer to \
                        function.",
                    )
                    .clone();

                return (function, index);
            }
            FunctionLocation::Local { location } => {
                current_location = *location.parent.parent;
            }
        }
    }
}

fn reconstruct_function(
    name: &str,
    entries: &mut VecDeque<ActiveFunctionsEntry>,
    code: &CompilerOutput,
    breakpoints: &Breakpoints,
    effect: Option<&Effect>,
) -> Option<String> {
    let Some(function) = code.syntax_tree.function_by_name(name) else {
        panic!("Expecting function `{name}` to exist.");
    };

    let tail_call = if let Some(branch) =
        function.into_located_function().find_single_branch()
    {
        let mut tail_call = None;

        for expression in branch.expressions() {
            tail_call = Some(expression.location);
        }

        tail_call
    } else {
        None
    };

    let expected_next_function = tail_call
        .as_ref()
        .and_then(|tail_call| function_call_to_function_name(tail_call, code));

    let cluster = code
        .dependencies
        .find_cluster_by_named_function(&function.index())
        .expect("All functions must be part of a cluster.");
    entries.push_front(ActiveFunctionsEntry::Function(DebugNamedFunction {
        name: function.name.clone(),
        inner: DebugFunction::new(
            function.inner.clone(),
            function.location(),
            tail_call.as_ref(),
            false,
            cluster,
            &code.functions,
            &code.function_calls,
            &code.types,
            &code.source_map,
            breakpoints,
            effect,
        ),
    }));

    expected_next_function
}

fn function_call_to_function_name(
    function_call: &MemberLocation,
    code: &CompilerOutput,
) -> Option<String> {
    let function_location = code
        .function_calls
        .is_call_to_user_defined_function(function_call)?;
    let function = code
        .syntax_tree
        .find_top_level_parent_function(function_location)?;

    Some(function.name.clone())
}
