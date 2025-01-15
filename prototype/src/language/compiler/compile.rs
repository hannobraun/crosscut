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

    let kind = match parse_token(token, &location, code, host) {
        Ok(expression) => FragmentKind::Expression { expression },
        Err(err) => FragmentKind::Error { err },
    };
    let fragment = Fragment {
        kind,
        body: Body::default(),
    };

    let maybe_error = check_for_error(&fragment);

    let id = code.append_to_body_at(location, fragment);

    if let Some(err) = maybe_error {
        code.errors.insert(id, err);
    }
}

fn parse_token(
    token: &str,
    append_to: &Location,
    code: &Code,
    host: &Host,
) -> Result<Expression, FragmentError> {
    assert!(
        !token.chars().any(|ch| ch.is_whitespace()),
        "Expecting tokens to not contain any whitespace.",
    );

    let can_append_expression = code
        .fragments()
        .get(append_to.target())
        .body
        .expression(code.fragments())
        .is_none();

    if !can_append_expression {
        return Err(FragmentError::UnexpectedToken {
            token: token.to_string(),
        });
    }

    match token.parse::<u32>() {
        Ok(value) => Ok(Expression::Literal {
            literal: Literal::Integer { value },
        }),
        Err(err) => match err.kind() {
            IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                let value = token.to_string();
                Err(FragmentError::IntegerOverflow { value })
            }
            _ => {
                let name = token.to_string();

                if let Some(id) = host.functions_by_name.get(&name).copied() {
                    Ok(Expression::FunctionCall { target: id })
                } else {
                    Err(FragmentError::UnresolvedIdentifier { name })
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
                FragmentError::UnexpectedToken { .. } => {
                    CodeError::UnexpectedToken
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
