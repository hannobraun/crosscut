use super::{
    code::{
        CodeError, Codebase, Expression, IntrinsicFunction, Location, Node,
        NodeKind,
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

    let kind = if token.is_empty() {
        NodeKind::Empty
    } else {
        match resolve_function(token, host) {
            Ok(expression) => NodeKind::Expression { expression },
            Err(candidates) => {
                codebase.insert_error(
                    location,
                    CodeError::UnresolvedIdentifier { candidates },
                );

                NodeKind::Unresolved {
                    name: token.to_string(),
                }
            }
        }
    };

    let node = Node { kind };

    codebase.replace_node(&location, node);
}

fn resolve_function(
    name: &str,
    host: &Host,
) -> Result<Expression, Vec<Expression>> {
    let host_function = host.function_id_by_name(name);
    let intrinsic_function = IntrinsicFunction::resolve(name);

    match (host_function, intrinsic_function) {
        (Some(id), None) => Ok(Expression::HostFunction { id }),
        (None, Some(function)) => {
            Ok(Expression::IntrinsicFunction { function })
        }
        (None, None) => {
            let candidates = Vec::new();
            Err(candidates)
        }
        (Some(id), Some(function)) => {
            let candidates = vec![
                Expression::HostFunction { id },
                Expression::IntrinsicFunction { function },
            ];
            Err(candidates)
        }
    }
}
