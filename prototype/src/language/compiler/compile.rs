use crate::language::code::{Code, Expression, Fragment, Token};

pub fn compile(input: &str, code: &mut Code) {
    for token in tokenize(input) {
        let fragment = match token {
            Token::Identifier { name } => Fragment::UnexpectedToken {
                token: Token::Identifier { name },
            },
            Token::LiteralNumber { value } => {
                if code.is_complete() {
                    let index = code.fragments.len();
                    code.errors.insert(index);

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
