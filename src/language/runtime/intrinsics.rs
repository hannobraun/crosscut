use itertools::Itertools;

use crate::language::code::{Codebase, IntrinsicFunction, Type};

use super::{Effect, Evaluator, Value};

pub fn apply_intrinsic_function(
    intrinsic: &IntrinsicFunction,
    input: Value,
    evaluator: &mut Evaluator,
    _: &Codebase,
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
        IntrinsicFunction::Identity => {
            evaluator.exit_from_provided_function(input);
        }
    }
}
