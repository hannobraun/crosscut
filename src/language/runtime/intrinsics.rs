use crate::language::code::{Codebase, IntrinsicFunction, Type};

use super::{Effect, Evaluator, Value};

pub fn apply_intrinsic_function(
    intrinsic: &IntrinsicFunction,
    input: Value,
    evaluator: &mut Evaluator,
    codebase: &Codebase,
) {
    match intrinsic {
        IntrinsicFunction::Drop => {
            evaluator.exit_from_provided_function(Value::Nothing);
        }
        IntrinsicFunction::Eval => match input {
            Value::Function { body } => {
                evaluator.apply_function_from_current_node(
                    body,
                    // Right now, the `eval` function doesn't support
                    // passing an argument to the function it
                    Value::Nothing,
                    codebase,
                );
            }
            input => {
                evaluator.trigger_effect(Effect::UnexpectedInput {
                    expected: Type::Function,
                    actual: input,
                });
            }
        },
        IntrinsicFunction::Identity => {
            evaluator.exit_from_provided_function(input);
        }
    }
}
