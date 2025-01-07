use super::code::{Code, Expression};

pub fn compile(input: String, code: &mut Code) {
    for token in input.split_whitespace() {
        let expression = match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => {
                let name = token.to_string();
                Expression::Identifier { name }
            }
        };

        code.expressions.push(expression);
    }
}
