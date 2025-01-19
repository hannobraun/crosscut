use std::num::IntErrorKind;

use crate::lang::{
    code::{
        Body, Code, CodeError, Expression, Fragment, FragmentError,
        FragmentKind, FunctionCallTarget, Literal, Location,
    },
    host::Host,
};

pub fn compile_and_replace(
    token: &str,
    to_replace: &Location,
    host: &Host,
    code: &mut Code,
) -> Location {
    let location_of_compiled_fragment = code.replace(
        to_replace,
        Fragment {
            kind: parse_token(token, host),
            body: Body::default(),
        },
    );

    handle_errors(&location_of_compiled_fragment, code);

    location_of_compiled_fragment
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

                if name == "identity" {
                    FragmentKind::Expression {
                        expression: Expression::FunctionCall {
                            target: FunctionCallTarget::IntrinsicFunction,
                        },
                    }
                } else if let Some(id) =
                    host.functions_by_name.get(&name).copied()
                {
                    FragmentKind::Expression {
                        expression: Expression::FunctionCall {
                            target: FunctionCallTarget::HostFunction { id },
                        },
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

    match &fragment.kind {
        FragmentKind::Expression {
            expression: Expression::FunctionCall { .. },
        } => {
            if fragment.body.is_empty() {
                code.errors
                    .insert(*location.target(), CodeError::MissingArgument);
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

            code.errors.insert(*location.target(), err);
        }
        _ => {}
    }

    if let Some(parent) = location.parent() {
        let parent_already_had_an_expression = code
            .fragments()
            .get(parent)
            .body
            .expressions(code.fragments())
            .count()
            > 1;

        if parent_already_had_an_expression {
            code.errors
                .insert(*location.target(), CodeError::UnexpectedToken);
        }
    }
}
