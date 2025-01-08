use super::{
    code::{Code, Fragment, Token},
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in tokenize(input) {
        let token = match &token {
            Token::Identifier { name } => {
                let index = code.fragments.len();

                if let Some(function) = host.function_by_name(name) {
                    code.function_calls.insert(index, function);
                }

                token
            }
            Token::LiteralNumber { .. } => token,
        };

        code.fragments.push(Fragment::UnexpectedToken { token });
    }
}

fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    input
        .split_whitespace()
        .map(|token| match token.parse::<f64>() {
            Ok(value) => Token::LiteralNumber { value },
            Err(_) => Token::Identifier {
                name: token.to_string(),
            },
        })
}
