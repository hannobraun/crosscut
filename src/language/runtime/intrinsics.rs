use itertools::Itertools;

use crate::language::code::{IntrinsicFunction, Type};

use super::{Effect, Evaluator, Value};

pub fn apply_intrinsic_function(
    intrinsic: &IntrinsicFunction,
    input: Value,
    _: &mut Evaluator,
) -> Result<Value, Effect> {
    match intrinsic {
        IntrinsicFunction::Add => {
            if let Value::Tuple { values } = &input {
                if let Some([a, b]) = values.iter().collect_array() {
                    if let [
                        Value::Integer { value: a },
                        Value::Integer { value: b },
                    ] = [a, b]
                    {
                        return Ok(Value::Integer { value: *a + *b });
                    }
                }
            }

            Err(Effect::UnexpectedInput {
                expected: Type::Tuple {
                    values: vec![Type::Integer, Type::Integer],
                },
                actual: input,
            })
        }
        IntrinsicFunction::Drop => Ok(Value::nothing()),
        IntrinsicFunction::Identity => Ok(input),
    }
}
