use itertools::Itertools;

use crate::language::code::{Codebase, IntrinsicFunction, Type};

use super::{Effect, Evaluator, Value};

pub fn apply_intrinsic_function(
    intrinsic: &IntrinsicFunction,
    input: Value,
    evaluator: &mut Evaluator,
    codebase: &Codebase,
) {
    match intrinsic {
        IntrinsicFunction::Add => {
            if let Value::Tuple { values } = &input {
                if let Some([a, b]) = values.iter().collect_array() {
                    if let [
                        Value::Integer { value: a },
                        Value::Integer { value: b },
                    ] = [a, b]
                    {
                        evaluator.exit_from_provided_function(Value::Integer {
                            value: *a + *b,
                        });
                        return;
                    }
                }
            }

            evaluator.trigger_effect(Effect::UnexpectedInput {
                expected: Type::Tuple {
                    values: vec![Type::Integer, Type::Integer],
                },
                actual: input,
            });
        }
        IntrinsicFunction::Drop => {
            evaluator.exit_from_provided_function(Value::nothing());
        }
        IntrinsicFunction::Eval => match input {
            Value::Function { body } => {
                evaluator.apply_function_from_current_node(
                    body,
                    // Right now, the `eval` function doesn't support
                    // passing an argument to the function it
                    Value::nothing(),
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
