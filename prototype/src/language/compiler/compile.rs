use crate::language::{
    code::{Body, Code, Expression, Fragment, FragmentKind, Token},
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in tokenize(input) {
        let append_to = code.find_innermost_fragment_with_valid_body();

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
                let can_append_expression = if let Some(id) = append_to.id() {
                    code.fragments()
                        .get(id)
                        .body
                        .expression(code.fragments())
                        .is_none()
                } else {
                    code.root.expression(code.fragments()).is_none()
                };

                if can_append_expression {
                    FragmentKind::Expression {
                        expression: Expression::LiteralValue { value },
                    }
                } else {
                    FragmentKind::UnexpectedToken {
                        token: Token::LiteralNumber { value },
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
            append_to,
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
