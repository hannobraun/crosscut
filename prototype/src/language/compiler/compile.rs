use crate::language::{
    code::{Code, Expression, Fragment, Token},
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in tokenize(input) {
        let fragment = match token {
            Token::Identifier { name } => {
                if let Some(id) = host.functions_by_name.get(&name).copied() {
                    Fragment::Expression {
                        expression: Expression::FunctionCall {
                            target: id,
                            argument: code.push(Fragment::MissingArgument),
                        },
                    }
                } else {
                    Fragment::UnexpectedToken {
                        token: Token::Identifier { name },
                    }
                }
            }
            Token::LiteralNumber { value } => {
                if code.is_complete() {
                    Fragment::UnexpectedToken {
                        token: Token::LiteralNumber { value },
                    }
                } else {
                    Fragment::Expression {
                        expression: Expression::LiteralValue { value },
                    }
                }
            }
        };

        let is_error = matches!(fragment, Fragment::UnexpectedToken { .. });

        let hash = code.push(fragment);

        if is_error {
            code.errors.insert(hash);
        }
    }
}

fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    input
        .split_whitespace()
        .map(|token| match token.parse::<u32>() {
            Ok(value) => Token::LiteralNumber { value },
            Err(_) => Token::Identifier {
                name: token.to_string(),
            },
        })
}
