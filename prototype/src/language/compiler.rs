use super::{
    code::{
        CodeError, Codebase, Expression, IntrinsicFunction, Node, NodeKind,
        NodePath,
    },
    host::Host,
};

pub fn compile(
    token: &str,
    path: NodePath,
    host: &Host,
    codebase: &mut Codebase,
) {
    codebase.clear_error(&path);

    let node = compile_token(token, path, host, codebase);

    codebase.replace_node(&path, node);
}

fn compile_token(
    token: &str,
    path: NodePath,
    host: &Host,
    codebase: &mut Codebase,
) -> Node {
    let kind = if token.is_empty() {
        NodeKind::Empty
    } else {
        match resolve_function(token, host) {
            Ok(expression) => NodeKind::Expression { expression },
            Err(candidates) => {
                codebase.insert_error(
                    path,
                    CodeError::UnresolvedIdentifier { candidates },
                );

                NodeKind::Unresolved {
                    name: token.to_string(),
                }
            }
        }
    };

    Node {
        // This is placeholder code, while support for syntax nodes having
        // inputs is still being added.
        child: None,
        kind,
    }
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
