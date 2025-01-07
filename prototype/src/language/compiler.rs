use super::code::Expression;

pub fn compile(input: String) -> Vec<Expression> {
    input
        .split_whitespace()
        .map(|token| match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => Expression::Identifier {
                name: token.to_string(),
            },
        })
        .collect()
}
