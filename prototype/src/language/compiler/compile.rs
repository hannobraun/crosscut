use std::num::IntErrorKind;

use crate::language::{
    code::{
        Body, Code, CodeError, Expression, Fragment, FragmentError,
        FragmentKind, FragmentPath, Literal, Token,
    },
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in tokenize(input) {
        let append_to = code.find_innermost_fragment_with_valid_body();

        let kind = match parse_token(token, &append_to, code, host) {
            Ok(expression) => FragmentKind::Expression { expression },
            Err(err) => FragmentKind::Error { err },
        };
        let fragment = Fragment {
            kind,
            body: Body::default(),
        };

        let maybe_error = check_for_error(&fragment);

        let id = code.append(fragment, append_to);

        if let Some(err) = maybe_error {
            code.errors.insert(id, err);
        }
    }
}

fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    input
        .split_whitespace()
        .map(|token| match token.parse::<u32>() {
            Ok(value) => Token::Literal {
                literal: Literal::Integer { value },
            },
            Err(err) => match err.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    Token::OverflowedInteger {
                        value: token.to_string(),
                    }
                }
                _ => Token::Identifier {
                    name: token.to_string(),
                },
            },
        })
}

fn parse_token(
    token: Token,
    append_to: &FragmentPath,
    code: &Code,
    host: &Host,
) -> Result<Expression, FragmentError> {
    match token {
        Token::Identifier { name } => {
            if let Some(id) = host.functions_by_name.get(&name).copied() {
                Ok(Expression::FunctionCall { target: id })
            } else {
                Err(FragmentError::UnresolvedIdentifier { name })
            }
        }
        Token::Literal {
            literal: Literal::Integer { value },
        } => {
            let can_append_expression = code
                .fragments()
                .get(append_to.id())
                .body
                .expression(code.fragments())
                .is_none();

            if can_append_expression {
                Ok(Expression::Literal {
                    literal: Literal::Integer { value },
                })
            } else {
                Err(FragmentError::UnexpectedToken {
                    token: Token::Literal {
                        literal: Literal::Integer { value },
                    },
                })
            }
        }
        Token::OverflowedInteger { value } => {
            Err(FragmentError::IntegerOverflow { value })
        }
    }
}

fn check_for_error(fragment: &Fragment) -> Option<CodeError> {
    match &fragment.kind {
        FragmentKind::Expression {
            expression: Expression::FunctionCall { .. },
        } => {
            if fragment.body.is_empty() {
                return Some(CodeError::MissingArgument);
            }
        }
        FragmentKind::Error { err } => match err {
            FragmentError::IntegerOverflow { .. } => {
                return Some(CodeError::IntegerOverflow);
            }
            FragmentError::UnexpectedToken { .. } => {
                return Some(CodeError::UnexpectedToken);
            }
            FragmentError::UnresolvedIdentifier { .. } => {
                return Some(CodeError::UnresolvedIdentifier);
            }
        },
        _ => {}
    }

    None
}
