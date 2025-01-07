use super::{
    code::{Code, Expression},
    host::Host,
};

pub fn compile(input: String, host: &Host, code: &mut Code) {
    for token in input.split_whitespace() {
        let expression = match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => {
                let index = code.expressions.len();
                let name = token.to_string();

                if let Some(function) = host.functions.get(&name).copied() {
                    code.function_calls.insert(index, function);
                }

                Expression::Identifier { name }
            }
        };

        code.expressions.push(expression);
    }
}
