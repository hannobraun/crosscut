use std::num::IntErrorKind;

use crate::language::{
    code::{
        Body, Code, CodeError, Expression, Fragment, FragmentError,
        FragmentKind, Literal,
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

    let kind = parse_token(token, host);
    let fragment = Fragment {
        kind,
        body: Body::default(),
    };

    let maybe_error = check_for_error(&fragment);

    let location_of_compiled_fragment = code.append_to(&location, fragment);

    if location_already_had_an_expression {
        code.errors.insert(
            *location_of_compiled_fragment.target(),
            CodeError::UnexpectedToken,
        );
    }
    if let Some(err) = maybe_error {
        code.errors
            .insert(*location_of_compiled_fragment.target(), err);
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

fn check_for_error(fragment: &Fragment) -> Option<CodeError> {
    match &fragment.kind {
        FragmentKind::Expression {
            expression: Expression::FunctionCall { .. },
        } => {
            if fragment.body.is_empty() {
                return Some(CodeError::MissingArgument);
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

            return Some(err);
        }
        _ => {}
    }

    None
}
