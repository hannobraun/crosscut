use super::{
    code::{Code, Token},
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in input.split_whitespace() {
        let expression = match token.parse::<f64>() {
            Ok(value) => Token::LiteralNumber { value },
            Err(_) => {
                let index = code.expressions.len();
                let name = token.to_string();

                if let Some(function) = host.function_by_name(&name) {
                    code.function_calls.insert(index, function);
                }

                Token::Identifier { name }
            }
        };

        code.expressions.push(expression);
    }
}
