use std::collections::BTreeMap;

use crate::code::syntax::{
    MemberLocation, ParameterLocation, SyntaxTree, SyntaxType,
};

use super::{Signature, Type};

pub fn resolve_type_annotations(
    syntax_tree: &SyntaxTree,
) -> (
    BTreeMap<ParameterLocation, Type>,
    BTreeMap<MemberLocation, Signature>,
) {
    let mut bindings = BTreeMap::new();
    let mut expressions = BTreeMap::new();

    for function in syntax_tree.all_functions() {
        for branch in function.branches() {
            for (binding, type_) in branch.annotated_bindings() {
                let Some(type_) = type_ else {
                    continue;
                };

                let type_ = resolve_type(type_);

                bindings.insert(binding.location, type_);
            }

            for (expression, signature) in branch.annotated_expressions() {
                let Some(signature) = signature else {
                    continue;
                };

                let signature = resolve_signature(signature);

                expressions.insert(expression.location, signature);
            }
        }
    }

    (bindings, expressions)
}

fn resolve_signature(signature: &Signature<SyntaxType>) -> Signature {
    Signature {
        inputs: resolve_types(&signature.inputs),
        outputs: resolve_types(&signature.outputs),
    }
}

fn resolve_types(types: &[SyntaxType]) -> Vec<Type> {
    types.iter().map(resolve_type).collect()
}

fn resolve_type(type_: &SyntaxType) -> Type {
    match type_ {
        SyntaxType::Function { signature } => {
            let signature = resolve_signature(signature);
            Type::Function { signature }
        }
        SyntaxType::Identifier { name } => match name.as_str() {
            "Number" => Type::Number,
            type_ => {
                panic!("Unknown type `{type_}`");
            }
        },
    }
}
