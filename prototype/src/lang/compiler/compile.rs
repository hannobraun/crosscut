use std::num::IntErrorKind;

use crate::lang::{
    code::{
        Body, CodeError, Codebase, Expression, FunctionCallTarget, Literal,
        Location, Node, NodeError, NodeKind,
    },
    host::Host,
};

pub fn compile_and_replace(
    token: &str,
    to_replace: &Location,
    host: &Host,
    code: &mut Codebase,
) -> Location {
    let location_of_compiled_fragment = code.replace(
        to_replace,
        Node {
            kind: parse_token(token, host),
            body: Body::default(),
        },
    );

    handle_errors(&location_of_compiled_fragment, code);

    location_of_compiled_fragment
}

fn parse_token(token: &str, host: &Host) -> NodeKind {
    assert!(
        !token.chars().any(|ch| ch.is_whitespace()),
        "Expecting tokens to not contain any whitespace.",
    );

    match token.parse::<u32>() {
        Ok(value) => NodeKind::Expression {
            expression: Expression::Literal {
                literal: Literal::Integer { value },
            },
        },
        Err(err) => match err.kind() {
            IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                let value = token.to_string();
                NodeKind::Error {
                    err: NodeError::IntegerOverflow { value },
                }
            }
            _ => {
                let name = token.to_string();

                let intrinsic_function =
                    if name == "identity" { Some(()) } else { None };
                let host_function = host.functions_by_name.get(&name).copied();

                match (intrinsic_function, host_function) {
                    (Some(()), None) => NodeKind::Expression {
                        expression: Expression::FunctionCall {
                            target: FunctionCallTarget::IntrinsicFunction,
                        },
                    },
                    (None, Some(id)) => NodeKind::Expression {
                        expression: Expression::FunctionCall {
                            target: FunctionCallTarget::HostFunction { id },
                        },
                    },
                    (None, None) => NodeKind::Error {
                        err: NodeError::UnresolvedIdentifier { name },
                    },
                    _ => NodeKind::Error {
                        err: NodeError::MultiResolvedIdentifier { name },
                    },
                }
            }
        },
    }
}

fn handle_errors(location: &Location, code: &mut Codebase) {
    let node = code.nodes().get(location.target());

    match &node.kind {
        NodeKind::Expression {
            expression: Expression::FunctionCall { .. },
        } => {
            if node.body.is_empty() {
                code.errors
                    .insert(*location.target(), CodeError::MissingArgument);
            }
        }
        NodeKind::Error { err } => {
            let err = match err {
                NodeError::IntegerOverflow { .. } => CodeError::IntegerOverflow,
                NodeError::MultiResolvedIdentifier { .. } => {
                    CodeError::MultiResolvedIdentifier
                }
                NodeError::UnresolvedIdentifier { .. } => {
                    CodeError::UnresolvedIdentifier
                }
            };

            code.errors.insert(*location.target(), err);
        }
        _ => {}
    }

    if let Some(parent) = location.parent() {
        let parent_already_had_an_expression = code
            .nodes()
            .get(parent)
            .body
            .expressions(code.nodes())
            .count()
            > 1;

        if parent_already_had_an_expression {
            code.errors
                .insert(*location.target(), CodeError::UnexpectedToken);
        }
    }
}
