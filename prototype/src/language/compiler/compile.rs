use crate::language::{
    code::{Body, Code, Expression, Fragment, FragmentKind, Token},
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in tokenize(input) {
        let fragment = match token {
            Token::Identifier { name } => {
                if let Some(id) = host.functions_by_name.get(&name).copied() {
                    FragmentKind::Expression {
                        expression: Expression::FunctionCall { target: id },
                    }
                } else {
                    FragmentKind::UnexpectedToken {
                        token: Token::Identifier { name },
                    }
                }
            }
            Token::LiteralNumber { value } => {
                if code.root.expression(code.fragments()).is_some() {
                    FragmentKind::UnexpectedToken {
                        token: Token::LiteralNumber { value },
                    }
                } else {
                    FragmentKind::Expression {
                        expression: Expression::LiteralValue { value },
                    }
                }
            }
        };

        let is_error = matches!(fragment, FragmentKind::UnexpectedToken { .. });

        let id = code.append(
            Fragment {
                kind: fragment,
                body: Body::default(),
            },
            code.find_innermost_fragment_with_valid_body(),
        );

        if is_error {
            code.errors.insert(id);
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
