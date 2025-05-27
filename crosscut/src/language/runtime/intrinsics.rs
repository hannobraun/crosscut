use itertools::Itertools;

use crate::language::code::Type;

use super::{Effect, Value};

pub fn apply_intrinsic_function(
    name: &str,
    input: &Value,
) -> Option<Result<Value, Effect>> {
    match name {
        "+" => {
            if let Value::Tuple { values } = &input {
                if let Some([a, b]) = values.iter().collect_array() {
                    if let [
                        Value::Integer { value: a },
                        Value::Integer { value: b },
                    ] = [a, b]
                    {
                        return Some(Ok(Value::Integer { value: *a + *b }));
                    }
                }
            }

            Some(Err(Effect::UnexpectedInput {
                expected: Type::Tuple {
                    values: vec![Type::Integer, Type::Integer],
                },
                actual: input.clone(),
            }))
        }
        "drop" => Some(Ok(Value::nothing())),
        "identity" => Some(Ok(input.clone())),
        _ => None,
    }
}
