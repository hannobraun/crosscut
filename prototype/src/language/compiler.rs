use std::collections::BTreeMap;

use super::code::{Code, Expression, FunctionType};

pub fn compile(
    input: String,
    functions: &BTreeMap<String, FunctionType>,
    code: &mut Code,
) {
    for token in input.split_whitespace() {
        let expression = match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => {
                let index = code.expressions.len();
                let name = token.to_string();

                if let Some(function) = functions.get(&name).copied() {
                    code.function_calls.insert(index, function);
                }

                Expression::Identifier { name }
            }
        };

        code.expressions.push(expression);
    }
}
