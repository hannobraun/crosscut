use std::collections::BTreeMap;

use array_util::ArrayExt;

use crate::{
    code::{
        syntax::{
            Branch, Expression, FunctionLocation, Located, MemberLocation,
            Parameter, ParameterLocation, SyntaxTree,
        },
        types::repr::Stacks,
        Bindings, Dependencies, DependencyCluster, Environment,
        IdentifierTarget, Identifiers, Signature, Type, TypeAnnotations,
    },
    intrinsics::IntrinsicFunction,
};

use super::{
    context::InferenceContext,
    function::InferredFunction,
    signature::{self, IndirectSignature},
    stack::{LocalStack, MaybeLocalStack},
    types::{ExpectedType, InferredType, InferredTypes, Result, TypeError},
};

pub fn infer(compiler_context: CompilerContext) -> InferenceOutput {
    let mut output = InferenceOutput::default();

    for cluster in compiler_context.dependencies.clusters() {
        if let Err(TypeError {
            expected,
            actual,
            location,
        }) = infer_cluster(cluster, compiler_context, &mut output)
        {
            let actual = actual
                .map(|type_| format!("`{type_}`"))
                .unwrap_or_else(|| "nothing".to_string());

            let location = location
                .map(|location| {
                    format!(
                        "at {}\n",
                        location.display(compiler_context.syntax_tree)
                    )
                })
                .unwrap_or(String::new());

            panic!(
                "\n\
                Type error: expected {expected}, got {actual}\n\
                \n\
                {location}",
            );
        }
    }

    output
}

#[derive(Clone, Copy)]
pub struct CompilerContext<'r> {
    pub syntax_tree: &'r SyntaxTree,
    pub bindings: &'r Bindings,
    pub identifiers: &'r Identifiers,
    pub dependencies: &'r Dependencies,
    pub annotations: &'r TypeAnnotations,
}

#[derive(Default)]
pub struct InferenceOutput {
    pub functions: BTreeMap<FunctionLocation, Signature>,
    pub expressions: BTreeMap<MemberLocation, Signature>,
    pub parameters: BTreeMap<ParameterLocation, Type>,
    pub stacks: Stacks,
}

fn infer_cluster(
    cluster: &DependencyCluster,
    compiler_context: CompilerContext,
    output: &mut InferenceOutput,
) -> Result<()> {
    let mut inference_context = InferenceContext::default();

    for branch in cluster.branches(compiler_context.syntax_tree) {
        let function = (*branch.location.parent).clone();

        let environment = compiler_context.bindings.environment_of(&function);

        let mut branch = infer_branch(
            branch,
            environment,
            &mut inference_context,
            compiler_context,
            output,
        )?;

        if let Some(function) = inference_context.functions.get_mut(&function) {
            function.unify_with(&mut branch, &mut inference_context.types);
        } else {
            inference_context.functions.insert(function.clone(), branch);
        }
    }

    // We have gone through all the branches in this branches, learning
    // everything about them there is to learn. Now we can go through the
    // inferred types and convert them into concrete types where possible.

    for (location, function) in inference_context.functions {
        let Some(signature) = function.to_signature() else {
            continue;
        };
        let Some(signature) =
            signature.to_direct(&mut inference_context.types)?
        else {
            continue;
        };

        let existing = output.functions.insert(location, signature);
        assert!(
            existing.is_none(),
            "Did not expect to overwrite an existing function signature.",
        );
    }
    for (location, index) in inference_context.bindings {
        let InferredType::Direct(type_) =
            inference_context.types.resolve(&index)?
        else {
            continue;
        };

        let existing = output.parameters.insert(location, type_);
        assert!(
            existing.is_none(),
            "Did not expect to overwrite an existing binding type.",
        );
    }
    for (location, signature) in inference_context.expressions {
        let Some(signature) =
            signature.to_direct(&mut inference_context.types)?
        else {
            continue;
        };

        let existing = output.expressions.insert(location, signature);
        assert!(
            existing.is_none(),
            "Did not expect to overwrite and existing expression signature.",
        );
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
fn infer_branch(
    branch: Located<&Branch>,
    environment: &Environment,
    inference_context: &mut InferenceContext,
    compiler_context: CompilerContext,
    output: &mut InferenceOutput,
) -> Result<InferredFunction> {
    let mut register_binding =
        |location: ParameterLocation,
         parameters: &BTreeMap<ParameterLocation, Type>| {
            inference_context
                .binding(&location, parameters)
                .unwrap_or_else(|| {
                    let type_ =
                        inference_context.types.push(InferredType::Unknown);
                    inference_context.bindings.insert(location, type_);
                    type_
                })
        };

    let parameters = branch
        .parameters()
        .map(|parameter| {
            let type_ = match parameter.fragment {
                Parameter::Binding { .. } => compiler_context
                    .annotations
                    .of_binding(&parameter.location)
                    .cloned(),
                Parameter::Literal { .. } => Some(Type::Number),
            };

            if let Some(type_) = type_ {
                output.parameters.insert(parameter.location.clone(), type_);
            }

            register_binding(parameter.location, &output.parameters)
        })
        .collect::<Vec<_>>();

    for binding in environment.bindings(compiler_context.syntax_tree) {
        register_binding(binding.location, &output.parameters);
    }

    let mut local_stack = MaybeLocalStack::default();
    let mut stacks = BTreeMap::new();

    for expression in branch.expressions() {
        let location = expression.location.clone();

        if let Some(stack) = local_stack.get_mut().cloned() {
            stacks.insert(location.clone(), stack);
        }

        if let Some(signature) = infer_expression(
            expression,
            inference_context,
            &mut local_stack,
            compiler_context,
            output,
        )? {
            inference_context.expressions.insert(location, signature);
        }
    }

    // Information from a later expression could have allowed us to infer the
    // type of an earlier one. So let's handle the stacks we collected _after_
    // we look at all of the expressions.
    for (location, local_stack) in stacks {
        let Some(local_stack) =
            local_stack.make_direct(&mut inference_context.types)?
        else {
            continue;
        };

        output.stacks.insert(location, local_stack);
    }

    let inputs = parameters;
    let outputs = local_stack
        .get()
        .cloned()
        .map(|local_stack| local_stack.inner);

    Ok(InferredFunction { inputs, outputs })
}

fn infer_expression(
    expression: Located<&Expression>,
    inference_context: &mut InferenceContext,
    local_stack: &mut MaybeLocalStack,
    compiler_context: CompilerContext,
    output: &InferenceOutput,
) -> Result<Option<IndirectSignature>> {
    let explicit = compiler_context
        .annotations
        .of_expression(&expression.location)
        .cloned()
        .map(|signature| {
            IndirectSignature::from_direct(
                signature,
                &mut inference_context.types,
            )
        });

    let inferred = match expression.fragment {
        Expression::Identifier { name: identifier } => {
            match compiler_context
                .identifiers
                .is_resolved(&expression.location)
            {
                Some(target) => match target {
                    IdentifierTarget::Binding(binding) => {
                        let Some(output) = inference_context
                            .binding(binding, &output.parameters)
                        else {
                            unreachable!(
                                "Identifier `{identifier}` has been resolved \
                                as binding, but it is not known in the branch. \
                                \n\
                                \n\
                                at {}\n\
                                \n\
                                binding at {}\n",
                                expression
                                    .location
                                    .display(compiler_context.syntax_tree),
                                binding.display(compiler_context.syntax_tree),
                            );
                        };
                        let signature = Signature {
                            inputs: vec![],
                            outputs: vec![output],
                        };
                        Some(signature)
                    }
                    IdentifierTarget::HostFunction(host) => {
                        let signature = IndirectSignature::from_direct(
                            host.signature.clone(),
                            &mut inference_context.types,
                        );
                        Some(signature)
                    }
                    IdentifierTarget::IntrinsicFunction(intrinsic) => {
                        local_stack
                            .get()
                            .and_then(|local_stack| {
                                infer_intrinsic(
                                    intrinsic,
                                    &expression.location,
                                    &mut inference_context.types,
                                    local_stack,
                                )
                                .transpose()
                            })
                            .transpose()?
                    }
                    IdentifierTarget::UserDefinedFunction(location) => {
                        inference_context.function(location, &output.functions)
                    }
                },
                None => None,
            }
        }
        Expression::LiteralNumber { .. } => {
            let signature = Signature {
                inputs: vec![],
                outputs: vec![Type::Number],
            };
            let signature = IndirectSignature::from_direct(
                signature,
                &mut inference_context.types,
            );
            Some(signature)
        }
        Expression::LocalFunction { .. } => {
            let location = FunctionLocation::from(expression.location.clone());
            inference_context
                .function(&location, &output.functions)
                .map(|signature| {
                    let function = inference_context.types.push(
                        InferredType::IndirectFunction {
                            signature: signature.clone(),
                        },
                    );

                    Signature {
                        inputs: vec![],
                        outputs: vec![function],
                    }
                })
        }
    };

    if let [Some(explicit), Some(inferred)] =
        [explicit.as_ref(), inferred.as_ref()].try_map_ext(|signature| {
            signature
                .and_then(|signature| {
                    signature
                        .to_direct(&mut inference_context.types)
                        .transpose()
                })
                .transpose()
        })?
    {
        panic!(
            "Type that could be inferred was also specified explicitly. This \
            is currently not allowed, as the goal is to transition away from \
            explicit type annotations completely.\n\
            \n\
            Explicit type: {explicit:?}\n\
            Inferred type: {inferred:?}\n\
            \n\
            Expression: {:#?}
            \n\
            At {}\n",
            expression.fragment,
            expression.location.display(compiler_context.syntax_tree),
        );
    }

    let signature = match (inferred, explicit) {
        (Some(inferred), Some(explicit)) => {
            signature::unify(
                [&inferred, &explicit],
                &mut inference_context.types,
            );

            Some(inferred)
        }
        (inferred, explicit) => inferred.or(explicit),
    };

    if let Some(signature) = signature {
        if let Some(local_stack) = local_stack.get_mut() {
            for input in signature.inputs.iter().rev() {
                match local_stack.inner.pop() {
                    Some(operand) => {
                        inference_context.types.unify([&operand, input]);
                    }
                    None => {
                        let input = inference_context.types.resolve(input)?;

                        return Err(TypeError {
                            expected: input.into_expected_type(
                                &mut inference_context.types,
                            )?,
                            actual: None,
                            location: Some(expression.location),
                        });
                    }
                }
            }
            for output in signature.outputs.iter() {
                local_stack.inner.push(*output);
            }
        }

        return Ok(Some(signature));
    } else {
        local_stack.invalidate();
    }

    Ok(None)
}

fn infer_intrinsic(
    intrinsic: &IntrinsicFunction,
    location: &MemberLocation,
    types: &mut InferredTypes,
    local_stack: &LocalStack,
) -> Result<Option<IndirectSignature>> {
    let signature = match intrinsic {
        IntrinsicFunction::Copy => {
            let type_ = types.push(InferredType::Unknown);

            Some(Signature {
                inputs: vec![type_],
                outputs: vec![type_, type_],
            })
        }
        IntrinsicFunction::Drop => {
            let input = types.push(InferredType::Unknown);

            Some(Signature {
                inputs: vec![input],
                outputs: vec![],
            })
        }
        IntrinsicFunction::Eval => {
            let top_operand = local_stack
                .inner
                .last()
                .map(|index| types.resolve(index))
                .transpose()?;

            let signature = match top_operand {
                Some(InferredType::IndirectFunction { signature }) => {
                    Some(signature)
                }
                Some(InferredType::Direct(Type::Function { signature })) => {
                    Some(IndirectSignature::from_direct(signature, types))
                }
                Some(InferredType::Direct(actual)) => {
                    return Err(TypeError {
                        expected: ExpectedType::Function,
                        actual: Some(actual.clone()),
                        location: Some(location.clone()),
                    });
                }
                Some(InferredType::Unknown { .. }) => None,
                None => {
                    return Err(TypeError {
                        expected: ExpectedType::Function,
                        actual: None,
                        location: Some(location.clone()),
                    });
                }
            };

            signature.map(|signature| {
                let outputs = signature.outputs.clone();
                let inputs = signature
                    .inputs
                    .clone()
                    .into_iter()
                    .chain([types
                        .push(InferredType::IndirectFunction { signature })])
                    .collect();

                IndirectSignature { inputs, outputs }
            })
        }
        intrinsic => {
            let Some(signature) = intrinsic.signature() else {
                unreachable!(
                    "All intrinsics that do not provide a signature are \
                    handled above.",
                );
            };

            Some(IndirectSignature::from_direct(signature, types))
        }
    };

    Ok(signature)
}
