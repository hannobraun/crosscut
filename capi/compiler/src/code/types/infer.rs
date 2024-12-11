use std::fmt;

use crate::{
    code::{
        syntax::{Branch, Expression, Located, MemberLocation, SyntaxTree},
        FunctionCalls, Index, IndexMap,
    },
    intrinsics::IntrinsicFunction,
};

use super::{
    repr::{Signatures, Stack, Stacks},
    ExplicitTypes, Signature, Type,
};

pub fn infer_types(context: Context) -> InferenceOutput {
    let mut output = InferenceOutput::default();

    for function in context.syntax_tree.all_functions() {
        for branch in function.branches() {
            if let Err(TypeError {
                expected,
                actual,
                location,
            }) = infer_branch(branch, context, &mut output)
            {
                let actual = actual
                    .map(|type_| format!("`{type_}`"))
                    .unwrap_or_else(|| "nothing".to_string());

                panic!(
                    "\n\
                    Type error: expected {expected}, got {actual}\n\
                    \n\
                    at {}\n",
                    location.display(context.syntax_tree),
                );
            }
        }
    }

    output
}

#[derive(Clone, Copy)]
pub struct Context<'r> {
    pub syntax_tree: &'r SyntaxTree,
    pub explicit_types: &'r ExplicitTypes,
    pub function_calls: &'r FunctionCalls,
}

#[derive(Default)]
pub struct InferenceOutput {
    pub signatures: Signatures,
    pub stacks: Stacks,
}

fn infer_branch(
    branch: Located<&Branch>,
    context: Context,
    output: &mut InferenceOutput,
) -> Result<(), TypeError> {
    let mut local_types = IndexMap::default();
    let mut local_stack = Some(Vec::new());

    for expression in branch.expressions() {
        infer_expression(
            expression,
            &mut local_types,
            &mut local_stack,
            context,
            output,
        )?;
    }

    dbg!(local_types);

    Ok(())
}

fn infer_expression(
    expression: Located<&Expression>,
    local_types: &mut IndexMap<Type>,
    local_stack: &mut Option<Stack>,
    context: Context,
    output: &mut InferenceOutput,
) -> Result<(), TypeError> {
    let explicit = context.explicit_types.signature_of(&expression.location);

    let inferred = match expression.fragment {
        Expression::Identifier { .. } => {
            let host = context
                .function_calls
                .is_call_to_host_function(&expression.location);
            let intrinsic = context
                .function_calls
                .is_call_to_intrinsic_function(&expression.location);

            match (host, intrinsic) {
                (Some(host), None) => {
                    let signature = host.signature.clone();
                    make_indirect(signature.clone(), local_types);
                    Some(signature)
                }
                (None, Some(intrinsic)) => {
                    let signature = infer_intrinsic(
                        intrinsic,
                        &expression.location,
                        local_stack,
                    )?;

                    signature.as_ref().map(|signature| {
                        make_indirect(signature.clone(), local_types)
                    });

                    signature
                }
                (None, None) => None,
                _ => {
                    unreachable!(
                        "Single identifier resolved to multiple function calls."
                    );
                }
            }
        }
        Expression::LiteralNumber { .. } => {
            let signature = Signature {
                inputs: vec![],
                outputs: vec![Type::Number],
            };
            make_indirect(signature.clone(), local_types);
            Some(signature)
        }
        _ => None,
    };

    if let (Some(explicit), Some(inferred)) = (explicit, inferred.as_ref()) {
        panic!(
            "Type that could be inferred was also specified explicitly. This \
            is currently not allowed, as the goal is to transition away from \
            explicit type annotations completely.\n\
            \n\
            Explicit type: {explicit:?}\n\
            Inferred type: {inferred:?}\n\
            \n\
            At {}\n",
            expression.location.display(context.syntax_tree),
        );
    }

    if let Some(signature) = inferred.or(explicit.cloned()) {
        if let Some(local_stack) = local_stack {
            for input in signature.inputs.iter().rev() {
                match local_stack.pop() {
                    Some(type_) if type_ == *input => {
                        // Type checks out!
                    }
                    actual => {
                        return Err(TypeError {
                            expected: ExpectedType::Specific(input.clone()),
                            actual,
                            location: expression.location,
                        });
                    }
                }
            }
            for output in signature.outputs.iter().cloned() {
                local_stack.push(output);
            }

            output
                .stacks
                .insert(expression.location.clone(), local_stack.clone());
        }

        output.signatures.insert(expression.location, signature);
    } else {
        *local_stack = None;
    }

    Ok(())
}

fn infer_intrinsic(
    intrinsic: &IntrinsicFunction,
    location: &MemberLocation,
    local_stack: &mut Option<Stack>,
) -> Result<Option<Signature>, TypeError> {
    let signature = match intrinsic {
        IntrinsicFunction::Eval => {
            let Some(local_stack) = local_stack.as_mut() else {
                return Ok(None);
            };

            match local_stack.last() {
                Some(Type::Function { signature }) => {
                    let outputs = signature.outputs.clone();
                    let inputs = signature
                        .inputs
                        .clone()
                        .into_iter()
                        .chain([Type::Function {
                            signature: signature.clone(),
                        }])
                        .collect();

                    Some(Signature { inputs, outputs })
                }
                actual => {
                    return Err(TypeError {
                        expected: ExpectedType::Function,
                        actual: actual.cloned(),
                        location: location.clone(),
                    });
                }
            }
        }
        intrinsic => intrinsic.signature(),
    };

    Ok(signature)
}

fn make_indirect(
    signature: Signature,
    local_types: &mut IndexMap<Type>,
) -> Signature<Index<Type>> {
    let inputs = signature
        .inputs
        .into_iter()
        .map(|input| local_types.push(input))
        .collect();
    let outputs = signature
        .outputs
        .into_iter()
        .map(|output| local_types.push(output))
        .collect();

    Signature { inputs, outputs }
}

struct TypeError {
    expected: ExpectedType,
    actual: Option<Type>,
    location: MemberLocation,
}

enum ExpectedType {
    Function,
    Specific(Type),
}

impl fmt::Display for ExpectedType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Specific(type_) => write!(f, "`{type_}`"),
        }
    }
}
