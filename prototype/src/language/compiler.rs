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
        match resolve_function(token, location, host, codebase) {
            Ok(expression) => Node::Expression { expression },
            Err(node) => node,
        }
    };

    codebase.replace_node(&location, node);
}

fn resolve_function(
    name: &str,
    location: Location,
    host: &Host,
    codebase: &mut Codebase,
) -> Result<Expression, Node> {
    let host_function = host.function_id_by_name(name);
    let intrinsic_function = IntrinsicFunction::resolve(name);

    match (host_function, intrinsic_function) {
        (Some(id), None) => Ok(Expression::HostFunction { id }),
        (None, Some(function)) => {
            Ok(Expression::IntrinsicFunction { function })
        }
        (None, None) => {
            let candidates = Vec::new();
            Err(emit_unresolved_identifier_error(
                name, location, candidates, codebase,
            ))
        }
        (Some(id), Some(function)) => {
            let candidates = vec![
                Expression::HostFunction { id },
                Expression::IntrinsicFunction { function },
            ];
            Err(emit_unresolved_identifier_error(
                name, location, candidates, codebase,
            ))
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
