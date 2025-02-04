use super::{
    code::{
        CodeError, Codebase, Expression, IntrinsicFunction, Node, NodeKind,
        NodePath,
    },
    host::Host,
};

pub fn compile(
    token: &str,
    path: &mut NodePath,
    host: &Host,
    codebase: &mut Codebase,
) {
    let (node, maybe_error) = compile_token(token, path, host, codebase);

    *path = codebase.replace_node(path, node);
    if let Some(error) = maybe_error {
        codebase.insert_error(*path, error);
    }
}

fn compile_token(
    token: &str,
    path: &NodePath,
    host: &Host,
    codebase: &Codebase,
) -> (Node, Option<CodeError>) {
    let (kind, maybe_error) = if token.is_empty() {
        (NodeKind::Empty, None)
    } else {
        match resolve_function(token, host) {
            Ok(expression) => (NodeKind::Expression { expression }, None),
            Err(candidates) => (
                NodeKind::Unresolved {
                    name: token.to_string(),
                },
                Some(CodeError::UnresolvedIdentifier { candidates }),
            ),
        }
    };

    let node = Node {
        child: codebase.child_of(path).map(|path| *path.hash()),
        kind,
    };

    (node, maybe_error)
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
