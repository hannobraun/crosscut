use std::collections::{BTreeMap, BTreeSet};

use crosscut_runtime::{Effect, Instruction, InstructionAddress, Value};

use crate::{
    code::syntax::{
        Binding, Branch, Expression, Function, FunctionLocation, Located,
        Parameter,
    },
    intrinsics::IntrinsicFunction,
    source_map::Mapping,
    Instructions,
};

use super::{
    compile_cluster::ClusterContext, compile_functions::FunctionsContext,
};

struct FunctionContext<'r> {
    location: &'r FunctionLocation,
}

pub fn compile_function(
    function: Located<&Function>,
    cluster_context: &mut ClusterContext,
    functions_context: &mut FunctionsContext,
) -> crosscut_runtime::Function {
    let mut context = FunctionContext {
        location: &function.location,
    };
    let mut runtime_function = crosscut_runtime::Function::default();
    let mut instruction_range = None;

    for branch in function.branches() {
        let (runtime_branch, [first_address, last_address]) = compile_branch(
            branch,
            &mut context,
            cluster_context,
            functions_context,
        );

        runtime_function.branches.push(runtime_branch);

        instruction_range = {
            let [first_in_function, _last_in_function] =
                instruction_range.unwrap_or([first_address, last_address]);

            Some([first_in_function, last_address])
        };
    }

    if let Some(instruction_range) = instruction_range {
        functions_context
            .source_map
            .map_function_to_instructions(function.location, instruction_range);
    }

    runtime_function
}

fn compile_branch(
    branch: Located<&Branch>,
    function_context: &mut FunctionContext,
    cluster_context: &mut ClusterContext,
    functions_context: &mut FunctionsContext,
) -> (crosscut_runtime::Branch, [InstructionAddress; 2]) {
    let parameters = branch.parameters.values().filter_map(|parameter| {
        match parameter {
            Parameter::Binding {
                binding: Binding { name },
                type_: _,
            } => Some(name),
            Parameter::Literal { .. } => {
                // Literal patterns are only relevant when
                // selecting the branch to execute. They no
                // longer have meaning once the function
                // actually starts executing.
                None
            }
        }
    });
    let bindings_address =
        compile_bindings(parameters, functions_context.instructions);

    let [body_address, last_address] = {
        let mut body_address = None;

        for expression in branch.expressions() {
            let bindings = branch
                .bindings()
                .map(|binding| binding.fragment.clone())
                .chain(
                    functions_context
                        .bindings
                        .environment_of(function_context.location)
                        .bindings(functions_context.syntax_tree)
                        .map(|binding| binding.fragment.clone()),
                )
                .collect();

            let addr = compile_expression(
                expression,
                bindings,
                cluster_context,
                functions_context,
            );
            body_address = body_address.or(Some(addr));
        }

        // Unconditionally generating a return instruction, like we do here, is
        // redundant. If the previous expression was a tail call, it didn't
        // create a new stack frame.
        //
        // In this case, the return instruction at the end of the called
        // function returns to the current function's caller, and we never get
        // to the return we generated here. It's just a junk instruction that
        // has no effect, except to make the code bigger.
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
        let last_instruction = emit_instruction(
            Instruction::Return,
            functions_context.instructions,
            None,
        );

        let first_instruction = body_address.unwrap_or(last_instruction);

        [first_instruction, last_instruction]
    };

    let first_address = bindings_address.unwrap_or(body_address);

    let branch = crosscut_runtime::Branch {
        parameters: branch
            .parameters
            .values()
            .cloned()
            .map(|parameter| match parameter {
                Parameter::Binding {
                    binding: Binding { name },
                    type_: _,
                } => crosscut_runtime::Pattern::Identifier { name },
                Parameter::Literal { value } => {
                    crosscut_runtime::Pattern::Literal { value }
                }
            })
            .collect(),
        start: first_address,
    };

    (branch, [first_address, last_address])
}

fn compile_bindings<'r, N>(
    names: N,
    instructions: &mut Instructions,
) -> Option<InstructionAddress>
where
    N: IntoIterator<Item = &'r String>,
    N::IntoIter: DoubleEndedIterator,
{
    let mut first_address = None;

    for name in names.into_iter().rev() {
        let address = emit_instruction(
            Instruction::Bind { name: name.clone() },
            instructions,
            None,
        );
        first_address = first_address.or(Some(address));
    }

    first_address
}

fn compile_expression(
    expression: Located<&Expression>,
    _: BTreeSet<Binding>,
    cluster_context: &mut ClusterContext,
    functions_context: &mut FunctionsContext,
) -> InstructionAddress {
    let is_tail_expression = functions_context
        .tail_expressions
        .is_tail_expression(&expression.location);

    let mut mapping = functions_context
        .source_map
        .map_expression_to_instructions(expression.location.clone());

    match expression.fragment {
        Expression::Identifier { name } => {
            if functions_context
                .bindings
                .is_binding(&expression.location)
                .is_some()
            {
                emit_instruction(
                    Instruction::BindingEvaluate { name: name.clone() },
                    functions_context.instructions,
                    Some(&mut mapping),
                )
            } else if let Some(function) = functions_context
                .function_calls
                .is_call_to_intrinsic_function(&expression.location)
            {
                compile_intrinsic(
                    function,
                    is_tail_expression,
                    functions_context.instructions,
                    &mut mapping,
                )
            } else if let Some(function) = functions_context
                .function_calls
                .is_call_to_host_function(&expression.location)
            {
                let address = emit_instruction(
                    Instruction::Push {
                        value: function.number.into(),
                    },
                    functions_context.instructions,
                    Some(&mut mapping),
                );
                emit_instruction(
                    Instruction::TriggerEffect {
                        effect: Effect::Host,
                    },
                    functions_context.instructions,
                    Some(&mut mapping),
                );
                address
            } else if let Some(callee_location) = functions_context
                .function_calls
                .is_call_to_user_defined_function(&expression.location)
            {
                let callee = functions_context
                    .functions
                    .by_location(callee_location)
                    .expect("Function referred to from cluster must exist.");

                let address = if functions_context
                    .recursion
                    .is_recursive_expression(&expression.location)
                {
                    // For recursive calls, we can't generally assume that the
                    // called function has been compiled yet. It's a recursive
                    // call, after all!
                    //
                    // Let's emit a placeholder instruction and arrange for that
                    // to be replaced later, once all of the functions in the
                    // cluster have been compiled.
                    let address = emit_instruction(
                        Instruction::TriggerEffect {
                            effect: Effect::CompilerBug,
                        },
                        functions_context.instructions,
                        Some(&mut mapping),
                    );
                    cluster_context
                        .recursive_calls_by_callee
                        .entry(callee.location)
                        .or_default()
                        .push(CallToFunction {
                            address,
                            is_tail_call: is_tail_expression,
                        });

                    address
                } else {
                    let Some(function) = functions_context
                        .compiled_functions_by_location
                        .get(callee_location)
                    else {
                        let function = functions_context
                            .syntax_tree
                            .named_functions()
                            .find(|function| {
                                function.location() == *callee_location
                            });

                        unreachable!(
                            "Compiling call to this user-defined function: \
                            `{}` \
                            Expecting functions to be compiled before any \
                            non-recursive calls to them, but can't find the \
                            compiled version of this one.\n\
                            \n\
                            Function:\n\
                            {function:#?}",
                            callee_location
                                .display(functions_context.syntax_tree,),
                        )
                    };

                    emit_instruction(
                        Instruction::CallFunction {
                            callee: function.clone(),
                            is_tail_call: is_tail_expression,
                        },
                        functions_context.instructions,
                        Some(&mut mapping),
                    )
                };

                // For now, we're done with this call. But the function we're
                // calling might get updated in the future. When that happens,
                // the compiler wants to know about all calls to the function,
                // to update them.
                //
                // Let's make sure that information is going to be available.
                functions_context
                    .call_instructions_by_callee
                    .inner
                    .entry(callee_location.clone())
                    .or_default()
                    .push(address);

                address
            } else {
                emit_instruction(
                    Instruction::TriggerEffect {
                        effect: Effect::BuildError,
                    },
                    functions_context.instructions,
                    Some(&mut mapping),
                )
            }
        }
        Expression::LiteralNumber { value } => emit_instruction(
            Instruction::Push { value: *value },
            functions_context.instructions,
            Some(&mut mapping),
        ),
        Expression::LocalFunction { function: _ } => {
            if functions_context
                .recursion
                .is_recursive_expression(&expression.location)
            {
                // For recursive local functions, we can't generally assume that
                // the local function has been compiled yet.
                //
                // Let's emit a placeholder instruction and arrange for that
                // to be replaced later, once all of the functions in the
                // cluster have been compiled.
                let address = emit_instruction(
                    Instruction::TriggerEffect {
                        effect: Effect::CompilerBug,
                    },
                    functions_context.instructions,
                    Some(&mut mapping),
                );
                cluster_context
                    .recursive_local_function_definitions_by_local_function
                    .insert(expression.location.into(), address);

                address
            } else {
                let location = FunctionLocation::from(expression.location);

                let Some(runtime_function) = functions_context
                    .compiled_functions_by_location
                    .get(&location)
                else {
                    unreachable!(
                        "Replacing instructions that define local functions \
                        _after_ all functions have been compiled. Yet can't \
                        find the local function.",
                    )
                };

                let environment = functions_context
                    .bindings
                    .environment_of(&location)
                    .bindings(functions_context.syntax_tree)
                    .map(|binding| binding.name.clone())
                    .collect();

                emit_instruction(
                    Instruction::MakeAnonymousFunction {
                        branches: runtime_function.branches.clone(),
                        environment,
                    },
                    functions_context.instructions,
                    Some(&mut mapping),
                )
            }
        }
    }
}

pub fn compile_call_to_function(
    callee: &FunctionLocation,
    call: CallToFunction,
    functions: &mut BTreeMap<FunctionLocation, crosscut_runtime::Function>,
    instructions: &mut Instructions,
) {
    let callee = functions.get(callee).expect(
        "Attempting to compile call to function. Expecting that function to \
        have been compiled already.",
    );

    instructions.replace(
        &call.address,
        Instruction::CallFunction {
            callee: callee.clone(),
            is_tail_call: call.is_tail_call,
        },
    );
}

pub fn compile_definition_of_local_function(
    local_function: FunctionLocation,
    address: InstructionAddress,
    functions_context: &mut FunctionsContext,
) {
    let Some(runtime_function) = functions_context
        .compiled_functions_by_location
        .get(&local_function)
    else {
        unreachable!(
            "Replacing instructions that define local functions _after_ all \
            functions have been compiled. Yet can't find the local function.",
        )
    };

    let environment = functions_context
        .bindings
        .environment_of(&local_function)
        .bindings(functions_context.syntax_tree)
        .map(|binding| binding.name.clone())
        .collect();

    functions_context.instructions.replace(
        &address,
        Instruction::MakeAnonymousFunction {
            branches: runtime_function.branches.clone(),
            environment,
        },
    );
}

fn compile_intrinsic(
    intrinsic: &IntrinsicFunction,
    is_tail_call: bool,
    instructions: &mut Instructions,
    mapping: &mut Mapping,
) -> InstructionAddress {
    let instruction = match intrinsic {
        IntrinsicFunction::AddS8 => Instruction::AddS8,
        IntrinsicFunction::AddS32 => Instruction::AddS32,
        IntrinsicFunction::AddU8 => Instruction::AddU8,
        IntrinsicFunction::AddU8Wrap => Instruction::AddU8Wrap,
        IntrinsicFunction::And => Instruction::LogicalAnd,
        IntrinsicFunction::Brk => Instruction::TriggerEffect {
            effect: Effect::Breakpoint,
        },
        IntrinsicFunction::Copy => {
            let offset_from_top = Value::from(0);

            let address = emit_instruction(
                Instruction::Push {
                    value: offset_from_top,
                },
                instructions,
                Some(mapping),
            );
            emit_instruction(Instruction::Copy, instructions, Some(mapping));

            return address;
        }
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
    };

    emit_instruction(instruction, instructions, Some(mapping))
}

fn emit_instruction(
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
