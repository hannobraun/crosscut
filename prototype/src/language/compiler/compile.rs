use std::num::IntErrorKind;

use crate::language::{
    code::{
        Body, Code, CodeError, Expression, Fragment, FragmentError,
        FragmentKind, Literal, Location,
    },
    host::Host,
};

pub fn compile(token: &str, host: &Host, code: &mut Code) {
    let location = code.find_innermost_fragment_with_valid_body();

    let location_already_had_an_expression = code
        .fragments()
        .get(location.target())
        .body
        .expression(code.fragments())
        .is_some();

    let to_replace = code.append_to(
        &location,
        Fragment {
            kind: FragmentKind::Empty,
            body: Body::default(),
        },
    );

    let fragment = Fragment {
        kind: parse_token(token, host),
        body: Body::default(),
    };

    let location_of_compiled_fragment = code.replace(&to_replace, fragment);

    handle_errors(&location_of_compiled_fragment, code);
    if location_already_had_an_expression {
        code.errors.insert(
            *location_of_compiled_fragment.target(),
            CodeError::UnexpectedToken,
        );
    }
}

fn parse_token(token: &str, host: &Host) -> FragmentKind {
    assert!(
        !token.chars().any(|ch| ch.is_whitespace()),
        "Expecting tokens to not contain any whitespace.",
    );

    match token.parse::<u32>() {
        Ok(value) => FragmentKind::Expression {
            expression: Expression::Literal {
                literal: Literal::Integer { value },
            },
        },
        Err(err) => match err.kind() {
            IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                let value = token.to_string();
                FragmentKind::Error {
                    err: FragmentError::IntegerOverflow { value },
                }
            }
            _ => {
                let name = token.to_string();

                if let Some(id) = host.functions_by_name.get(&name).copied() {
                    FragmentKind::Expression {
                        expression: Expression::FunctionCall { target: id },
                    }
                } else {
                    FragmentKind::Error {
                        err: FragmentError::UnresolvedIdentifier { name },
                    }
                }
            }
        },
    }
}

fn handle_errors(location: &Location, code: &mut Code) {
    let fragment = code.fragments().get(location.target());

    let maybe_error = match &fragment.kind {
        FragmentKind::Expression {
            expression: Expression::FunctionCall { .. },
        } => {
            if fragment.body.is_empty() {
                Some(CodeError::MissingArgument)
            } else {
                None
            }
        }
        FragmentKind::Error { err } => {
            let err = match err {
                FragmentError::IntegerOverflow { .. } => {
                    CodeError::IntegerOverflow
                }
                FragmentError::UnresolvedIdentifier { .. } => {
                    CodeError::UnresolvedIdentifier
                }
            };

            Some(err)
        }
        _ => None,
    };

    if let Some(err) = maybe_error {
        code.errors.insert(*location.target(), err);
    }
}
