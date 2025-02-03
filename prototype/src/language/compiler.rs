use super::{
    code::{
        CodeError, Codebase, Expression, IntrinsicFunction, Location, Node,
    },
    host::Host,
};

pub fn compile(
    token: &str,
    location: Location,
    host: &Host,
    codebase: &mut Codebase,
) {
    codebase.clear_error(&location);

    let node = if token.is_empty() {
        Node::Empty
    } else {
        resolve_function(token, location, host, codebase)
    };

    codebase.replace_node(&location, node);
}

fn resolve_function(
    token: &str,
    location: Location,
    host: &Host,
    codebase: &mut Codebase,
) -> Node {
    let host_function = host.function_id_by_name(token);
    let intrinsic_function = IntrinsicFunction::resolve(token);

    match (host_function, intrinsic_function) {
        (Some(id), None) => Node::Expression {
            expression: Expression::HostFunction { id },
        },
        (None, Some(function)) => Node::Expression {
            expression: Expression::IntrinsicFunction { function },
        },
        (None, None) => {
            let candidates = Vec::new();
            emit_unresolved_identifier_error(
                token, location, candidates, codebase,
            )
        }
        (Some(id), Some(function)) => {
            let candidates = vec![
                Expression::HostFunction { id },
                Expression::IntrinsicFunction { function },
            ];
            emit_unresolved_identifier_error(
                token, location, candidates, codebase,
            )
        }
    }
}

fn emit_unresolved_identifier_error(
    token: &str,
    location: Location,
    candidates: Vec<Expression>,
    codebase: &mut Codebase,
) -> Node {
    codebase
        .insert_error(location, CodeError::UnresolvedIdentifier { candidates });

    Node::Unresolved {
        name: token.to_string(),
    }
}
