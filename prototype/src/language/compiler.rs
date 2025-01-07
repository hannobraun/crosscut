use super::code::{Code, Expression};

pub fn compile(input: String, code: &mut Code) {
    for token in input.split_whitespace() {
        let expression = match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => Expression::Identifier {
                name: token.to_string(),
            },
        };

        code.expressions.push(expression);
    }
}
