use super::code::Expression;

pub fn compile(input: String) -> Vec<Expression> {
    let mut expressions = Vec::new();

    for token in input.split_whitespace() {
        let expression = match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => Expression::Identifier {
                name: token.to_string(),
            },
        };

        expressions.push(expression);
    }

    expressions
}
