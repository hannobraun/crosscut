use super::{
    code::{Code, Expression, Fragment, Token},
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    // This is a hack to get things starting to work, while I get a real
    // compiler pipeline set up.
    let mut already_compiled_expression = code
        .fragments
        .last()
        .map(|fragment| match fragment {
            Fragment::Expression { .. } => true,
            Fragment::UnexpectedToken { token } => match token {
                Token::Identifier { .. } => false,
                Token::LiteralNumber { .. } => true,
            },
        })
        .unwrap_or(false);

    for token in tokenize(input) {
        let fragment = match token {
            Token::Identifier { name } => {
                if let Some(function) = host.function_by_name(&name) {
                    let index = code.fragments.len();
                    code.function_calls.insert(index, function);
                }

                Fragment::UnexpectedToken {
                    token: Token::Identifier { name },
                }
            }
            Token::LiteralNumber { value } => {
                if already_compiled_expression {
                    Fragment::UnexpectedToken {
                        token: Token::LiteralNumber { value },
                    }
                } else {
                    already_compiled_expression = true;

                    Fragment::Expression {
                        expression: Expression::LiteralValue { value },
                    }
                }
            }
        };

        code.fragments.push(fragment);
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
