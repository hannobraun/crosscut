use std::collections::{BTreeMap, VecDeque};

use capi_runtime::{Effect, Instruction, InstructionAddress, Instructions};

use crate::{
    code::{
        Branch, BranchLocation, CallGraph, Changes, Cluster, Fragment,
        FragmentLocation, Function, FunctionInUpdate,
        FunctionIndexInRootContext, FunctionLocation, FunctionUpdate,
        NamedFunctions, Pattern,
    },
    hash::Hash,
    intrinsics::IntrinsicFunction,
    source_map::{Mapping, SourceMap},
};

pub fn generate_instructions(
    named_functions: &NamedFunctions,
    call_graph: &CallGraph,
    changes: &Changes,
    instructions: &mut Instructions,
    calls_by_function: &mut BTreeMap<Hash<Function>, Vec<InstructionAddress>>,
    source_map: &mut SourceMap,
) {
    // The placeholder call into `main` is created unconditionally, regardless
    // of whether this is a fresh build and we actually need to do that, or if
    // we already have an active runtime and are just compiling changes.
    //
    // I don't think this has any adverse effects, except creating junk
    // instructions that increase the code size. And I don't want to fix that,
    // until we have infrastructure in place that would measure the code size
    // and actually show the impact of those changes.
    //
    // Otherwise, we'll just complicate the code with unclear benefit, and no
    // means to track whether simplifications are beneficial or not.
    let call_to_main = create_placeholder_for_call_to_main(instructions);

    let named_functions_to_compile = gather_named_functions_to_compile(changes);
    let queue_of_functions_to_compile = seed_queue_of_functions_to_compile(
        named_functions_to_compile,
        call_graph,
    );

    let mut output = CompileFunctions {
        named_functions,
        instructions,
        source_map,
        calls_by_function,
        queue_of_functions_to_compile,
        placeholders: BTreeMap::new(),
    };
    let mut functions = BTreeMap::default();

    while let Some(function_to_compile) =
        output.queue_of_functions_to_compile.pop_front()
    {
        compile_function(function_to_compile, &mut output, &mut functions);
    }

    if let Some(function) = named_functions.find_by_name("main") {
        output
            .placeholders
            .entry(Hash::new(&function))
            .or_default()
            .push(CallToFunction {
                address: call_to_main,
                is_tail_call: true,
            });
    }

    for (hash, calls) in &output.placeholders {
        for call in calls {
            compile_call_to_function(
                hash,
                call,
                &mut functions,
                output.instructions,
            );
        }
    }

    for update in &changes.updated {
        let old_hash = Hash::new(&update.old.function);
        let new_hash = Hash::new(&update.new.function);

        for calling_address in output
            .calls_by_function
            .remove(&old_hash)
            .unwrap_or_default()
        {
            let calling_instruction = output
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

            let function = functions.get(&new_hash).expect(
                "New function referenced in update should have been compiled; \
                is expected to exist.",
            );
            let function = capi_runtime::Function {
                branches: function
                    .iter()
                    .map(|(parameters, address)| {
                        let parameters = parameters
                            .iter()
                            .cloned()
                            .map(|pattern| match pattern {
                                Pattern::Identifier { name } => {
                                    capi_runtime::Pattern::Identifier { name }
                                }
                                Pattern::Literal { value } => {
                                    capi_runtime::Pattern::Literal { value }
                                }
                            })
                            .collect();

                        capi_runtime::Branch {
                            parameters,
                            start: *address,
                        }
                    })
                    .collect(),
                environment: BTreeMap::new(),
            };

            output.instructions.replace(
                &calling_address,
                Instruction::CallFunction {
                    function,
                    is_tail_call: *is_tail_call,
                },
            );
        }
    }
}

fn create_placeholder_for_call_to_main(
    instructions: &mut Instructions,
) -> InstructionAddress {
    // If there's no `main` function, this instruction won't get replaced later.
    // That would be a result of invalid code (valid code would provide a `main`
    // function), so an instruction generating the `BuildError` effect is an
    // appropriate placeholder.
    instructions.push(Instruction::TriggerEffect {
        effect: Effect::BuildError,
    })
}

fn gather_named_functions_to_compile(
    changes: &Changes,
) -> BTreeMap<&FunctionIndexInRootContext, &Function> {
    changes
        .added
        .iter()
        .chain(changes.updated.iter().map(
            |FunctionUpdate {
                 new: FunctionInUpdate { index, function },
                 ..
             }| (index, function),
        ))
        .collect::<BTreeMap<_, _>>()
}

fn seed_queue_of_functions_to_compile(
    mut named_functions_to_compile: BTreeMap<
        &FunctionIndexInRootContext,
        &Function,
    >,
    call_graph: &CallGraph,
) -> VecDeque<FunctionToCompile> {
    call_graph
        .functions_from_leaves()
        .filter_map(|(&index, cluster)| {
            let function = named_functions_to_compile.remove(&index)?;
            Some(FunctionToCompile {
                function: function.clone(),
                location: FunctionLocation::NamedFunction { index },
                cluster: cluster.clone(),
                address_of_instruction_to_make_anon_function: None,
            })
        })
        .collect::<VecDeque<_>>()
}

struct CompileFunctions<'r> {
    named_functions: &'r NamedFunctions,
    instructions: &'r mut Instructions,
    source_map: &'r mut SourceMap,
    calls_by_function:
        &'r mut BTreeMap<Hash<Function>, Vec<InstructionAddress>>,
    queue_of_functions_to_compile: VecDeque<FunctionToCompile>,
    placeholders: BTreeMap<Hash<Function>, Vec<CallToFunction>>,
}

fn compile_function(
    function_to_compile: FunctionToCompile,
    output: &mut CompileFunctions,
    functions: &mut BTreeMap<
        Hash<Function>,
        Vec<(Vec<Pattern>, InstructionAddress)>,
    >,
) {
    let FunctionToCompile {
        function,
        location,
        cluster,
        address_of_instruction_to_make_anon_function,
    } = function_to_compile;

    let mut branches = Vec::new();
    let mut instruction_range = None;

    for (&index, branch) in function.branches.iter() {
        let parameters = branch.parameters.iter().filter_map(|pattern| {
            match pattern {
                Pattern::Identifier { name } => Some(name),
                Pattern::Literal { .. } => {
                    // Literal patterns are only relevant when
                    // selecting the branch to execute. They no
                    // longer have meaning once the function
                    // actually starts executing.
                    None
                }
            }
        });
        let bindings_address = compile_binding(parameters, output.instructions);

        let [branch_address, last_address] = compile_branch(
            branch,
            BranchLocation {
                parent: Box::new(location.clone()),
                index,
            },
            &cluster,
            output,
            functions,
        );

        let first_address = bindings_address.unwrap_or(branch_address);
        functions
            .entry(Hash::new(&function))
            .or_default()
            .push((branch.parameters.clone(), first_address));

        branches.push(capi_runtime::Branch {
            parameters: branch
                .parameters
                .iter()
                .cloned()
                .map(|pattern| match pattern {
                    Pattern::Identifier { name } => {
                        capi_runtime::Pattern::Identifier { name }
                    }
                    Pattern::Literal { value } => {
                        capi_runtime::Pattern::Literal { value }
                    }
                })
                .collect(),
            start: first_address,
        });

        instruction_range = {
            let [first_in_function, _last_in_function] =
                instruction_range.unwrap_or([first_address, last_address]);

            Some([first_in_function, last_address])
        };
    }

    if let Some(instruction_range) = instruction_range {
        output
            .source_map
            .map_function_to_instructions(location, instruction_range);
    }

    if let Some(address) = address_of_instruction_to_make_anon_function {
        output.instructions.replace(
            &address,
            Instruction::MakeAnonymousFunction {
                branches,
                environment: function.environment,
            },
        );
    } else {
        assert!(
            function.environment.is_empty(),
            "We were not provided an address where to put a \"make anonymous \
            function\" instruction, and yet the function has an environment. \
            This is a bug.",
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_branch(
    branch: &Branch,
    location: BranchLocation,
    cluster: &Cluster,
    output: &mut CompileFunctions,
    functions: &mut BTreeMap<
        Hash<Function>,
        Vec<(Vec<Pattern>, InstructionAddress)>,
    >,
) -> [InstructionAddress; 2] {
    let mut first_instruction = None;

    for (&index, fragment) in &branch.body {
        let addr = compile_fragment(
            fragment,
            FragmentLocation {
                parent: Box::new(location.clone()),
                index,
            },
            cluster,
            output,
            functions,
        );
        first_instruction = first_instruction.or(addr);
    }

    // Unconditionally generating a return instruction, like we do here, is
    // redundant. If the previous fragment was a tail call, it didn't create a
    // new stack frame.
    //
    // In this case, the return instruction at the end of the called function
    // returns to the current function's caller, and we never get to the return
    // we generated here. It's just a junk instruction that has no effect,
    // except to make the code bigger.
    //
    // I don't think it's worth fixing right now, for the following reasons:
    //
    // - Tail call elimination still partially happens at runtime. The
    //   plan is to move it to compile-time completely. Adding other
    //   optimizations (like omitting this return instruction) will make
    //   this transition more complicated, for little gain in the
    //   meantime.
    // - There's no infrastructure in place to measure the impact of
    //   compiler optimizations. I'd rather have that, instead of making
    //   this change blindly. It will probably make the code more
    //   complicated, so it needs to be justified.
    let last_instruction =
        generate_instruction(Instruction::Return, output.instructions, None);

    let first_instruction = first_instruction.unwrap_or(last_instruction);

    [first_instruction, last_instruction]
}

#[allow(clippy::too_many_arguments)]
fn compile_fragment(
    fragment: &Fragment,
    location: FragmentLocation,
    cluster: &Cluster,
    output: &mut CompileFunctions,
    functions: &mut BTreeMap<
        Hash<Function>,
        Vec<(Vec<Pattern>, InstructionAddress)>,
    >,
) -> Option<InstructionAddress> {
    match &fragment {
        Fragment::CallToUserDefinedFunction {
            hash, is_tail_call, ..
        } => {
            // We know that this expression refers to a user-defined function,
            // but we might not have compiled that function yet.
            //
            // For now, just generate a placeholder that we can replace with the
            // call later.
            let address = generate_instruction(
                Instruction::TriggerEffect {
                    effect: Effect::CompilerBug,
                },
                output.instructions,
                Some(
                    &mut output
                        .source_map
                        .map_fragment_to_instructions(location),
                ),
            );

            compile_call_to_function(
                hash,
                &CallToFunction {
                    address,
                    is_tail_call: *is_tail_call,
                },
                functions,
                output.instructions,
            );

            // We also need to do some bookkeeping, so we can update the call,
            // in case the called function is updated.
            output
                .calls_by_function
                .entry(*hash)
                .or_default()
                .push(address);

            Some(address)
        }
        Fragment::CallToUserDefinedFunctionRecursive {
            index,
            is_tail_call,
        } => {
            let function_index_in_root_context = cluster.functions[index];
            let called_function = output
                .named_functions
                .get(&function_index_in_root_context)
                .expect("Function referred to from cluster must exist.");
            let hash = Hash::new(called_function);

            // We know that this expression refers to a user-defined function,
            // but we might not have compiled that function yet.
            //
            // For now, just generate a placeholder that we can replace with the
            // call later.
            let address = generate_instruction(
                Instruction::TriggerEffect {
                    effect: Effect::CompilerBug,
                },
                output.instructions,
                Some(
                    &mut output
                        .source_map
                        .map_fragment_to_instructions(location),
                ),
            );

            // We can't leave it at that, however. We need to make sure this
            // placeholder actually gets replaced later, and we're doing that by
            // adding it to this list.
            output
                .placeholders
                .entry(hash)
                .or_default()
                .push(CallToFunction {
                    address,
                    is_tail_call: *is_tail_call,
                });

            // We also need to do some bookkeeping, so we can update the call,
            // in case the called function is updated.
            output
                .calls_by_function
                .entry(hash)
                .or_default()
                .push(address);

            Some(address)
        }
        Fragment::CallToHostFunction { effect_number } => {
            let mut mapping =
                output.source_map.map_fragment_to_instructions(location);

            let address = generate_instruction(
                Instruction::Push {
                    value: (*effect_number).into(),
                },
                output.instructions,
                Some(&mut mapping),
            );
            generate_instruction(
                Instruction::TriggerEffect {
                    effect: Effect::Host,
                },
                output.instructions,
                Some(&mut mapping),
            );
            Some(address)
        }
        Fragment::CallToIntrinsicFunction {
            intrinsic,
            is_tail_call,
        } => {
            let instruction =
                intrinsic_to_instruction(intrinsic, *is_tail_call);

            Some(generate_instruction(
                instruction,
                output.instructions,
                Some(
                    &mut output
                        .source_map
                        .map_fragment_to_instructions(location),
                ),
            ))
        }
        Fragment::Comment { .. } => None,
        Fragment::Function { function } => {
            assert!(
                function.name.is_none(),
                "An anonymous function should not have a name."
            );

            // We have encountered an anonymous function. We need to emit an
            // instruction that allocates it, and takes care of its environment.
            //
            // But we haven't compiled the anonymous function yet, and we can't
            // do that right now. If we did, we would be emitting its
            // instructions in the middle of whatever function (anonymous or
            // named) that we're currently compiling.
            //
            // The result of that would be, that every anonymous function would
            // be executed right where it's defined, which would defeat the
            // purpose of having them in the first place.
            //
            // But we still somehow need to emit that instruction to allocate
            // the anonymous function and take care of its environment. We'll do
            // that later, after we've actually compiled the anonymous function.
            //
            // For now, we'll just emit a placeholder that can be replaced with
            // the real instruction then.
            let address_of_instruction_to_make_anon_function =
                Some(generate_instruction(
                    Instruction::TriggerEffect {
                        effect: Effect::CompilerBug,
                    },
                    output.instructions,
                    Some(
                        &mut output
                            .source_map
                            .map_fragment_to_instructions(location.clone()),
                    ),
                ));

            // We've done what we could. Let's arrange for the anonymous
            // function to be compiled, and the placeholder instruction to be
            // replaced, at a later time.
            output.queue_of_functions_to_compile.push_front(
                FunctionToCompile {
                    function: function.clone(),
                    location: FunctionLocation::AnonymousFunction { location },
                    cluster: cluster.clone(),
                    address_of_instruction_to_make_anon_function,
                },
            );

            address_of_instruction_to_make_anon_function
        }
        Fragment::ResolvedBinding { name } => Some(generate_instruction(
            Instruction::BindingEvaluate { name: name.clone() },
            output.instructions,
            Some(&mut output.source_map.map_fragment_to_instructions(location)),
        )),
        Fragment::UnresolvedIdentifier { .. } => Some(generate_instruction(
            Instruction::TriggerEffect {
                effect: Effect::BuildError,
            },
            output.instructions,
            Some(&mut output.source_map.map_fragment_to_instructions(location)),
        )),
        Fragment::Value(value) => Some(generate_instruction(
            Instruction::Push { value: *value },
            output.instructions,
            Some(&mut output.source_map.map_fragment_to_instructions(location)),
        )),
    }
}

fn compile_call_to_function(
    hash: &Hash<Function>,
    call: &CallToFunction,
    functions: &mut BTreeMap<
        Hash<Function>,
        Vec<(Vec<Pattern>, InstructionAddress)>,
    >,
    instructions: &mut Instructions,
) {
    let Some(function) = functions.get(hash) else {
        // This won't happen for any regular function, because we only create
        // placeholders for functions that we actually encounter. But it can
        // happen for the `main` function, since we create a placeholder for
        // that unconditionally.
        //
        // If that happens, let's just leave the placeholder panic. It's not
        // great, as it doesn't provide any context to the user. But while we
        // don't have any way to make panics more descriptive, it'll have to do.
        return;
    };

    let function = capi_runtime::Function {
        branches: function
            .iter()
            .map(|(parameters, address)| {
                let parameters = parameters
                    .iter()
                    .cloned()
                    .map(|pattern| match pattern {
                        Pattern::Identifier { name } => {
                            capi_runtime::Pattern::Identifier { name }
                        }
                        Pattern::Literal { value } => {
                            capi_runtime::Pattern::Literal { value }
                        }
                    })
                    .collect();

                capi_runtime::Branch {
                    parameters,
                    start: *address,
                }
            })
            .collect(),
        environment: BTreeMap::new(),
    };

    instructions.replace(
        &call.address,
        Instruction::CallFunction {
            function: function.clone(),
            is_tail_call: call.is_tail_call,
        },
    );
}

fn intrinsic_to_instruction(
    intrinsic: &IntrinsicFunction,
    is_tail_call: bool,
) -> Instruction {
    match intrinsic {
        IntrinsicFunction::AddS8 => Instruction::AddS8,
        IntrinsicFunction::AddS32 => Instruction::AddS32,
        IntrinsicFunction::AddU8 => Instruction::AddU8,
        IntrinsicFunction::AddU8Wrap => Instruction::AddU8Wrap,
        IntrinsicFunction::And => Instruction::LogicalAnd,
        IntrinsicFunction::Brk => Instruction::TriggerEffect {
            effect: Effect::Breakpoint,
        },
        IntrinsicFunction::Copy => Instruction::Copy,
        IntrinsicFunction::DivS32 => Instruction::DivS32,
        IntrinsicFunction::DivU8 => Instruction::DivU8,
        IntrinsicFunction::Drop => Instruction::Drop,
        IntrinsicFunction::Eq => Instruction::Eq,
        IntrinsicFunction::Eval => Instruction::Eval { is_tail_call },
        IntrinsicFunction::GreaterS8 => Instruction::GreaterS8,
        IntrinsicFunction::GreaterS32 => Instruction::GreaterS32,
        IntrinsicFunction::GreaterU8 => Instruction::GreaterU8,
        IntrinsicFunction::MulS32 => Instruction::MulS32,
        IntrinsicFunction::MulU8Wrap => Instruction::MulU8Wrap,
        IntrinsicFunction::NegS32 => Instruction::NegS32,
        IntrinsicFunction::Nop => Instruction::Nop,
        IntrinsicFunction::Not => Instruction::LogicalNot,
        IntrinsicFunction::RemainderS32 => Instruction::RemainderS32,
        IntrinsicFunction::S32ToS8 => Instruction::ConvertS32ToS8,
        IntrinsicFunction::SubS32 => Instruction::SubS32,
        IntrinsicFunction::SubU8 => Instruction::SubU8,
        IntrinsicFunction::SubU8Wrap => Instruction::SubU8Wrap,
    }
}

fn compile_binding<'r, N>(
    names: N,
    instructions: &mut Instructions,
) -> Option<InstructionAddress>
where
    N: IntoIterator<Item = &'r String>,
    N::IntoIter: DoubleEndedIterator,
{
    let mut first_address = None;

    for name in names.into_iter().rev() {
        let address = generate_instruction(
            Instruction::Bind { name: name.clone() },
            instructions,
            None,
        );
        first_address = first_address.or(Some(address));
    }

    first_address
}

fn generate_instruction(
    instruction: Instruction,
    instructions: &mut Instructions,
    mapping: Option<&mut Mapping<'_>>,
) -> InstructionAddress {
    let addr = instructions.push(instruction);
    if let Some(mapping) = mapping {
        mapping.append_instruction(addr);
    }
    addr
}

pub struct CallToFunction {
    pub address: InstructionAddress,
    pub is_tail_call: bool,
}

struct FunctionToCompile {
    function: Function,
    location: FunctionLocation,
    cluster: Cluster,
    address_of_instruction_to_make_anon_function: Option<InstructionAddress>,
}
